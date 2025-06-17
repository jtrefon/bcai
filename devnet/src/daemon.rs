//! The logic for the devnet daemon process.

use crate::cli::{AccountCommands, P2pCommands, TxCommands};
use log::{error, info};
use runtime::{
    blockchain::{Blockchain, Transaction, DEV_GENESIS_PUBKEY},
    miner,
    network::NetworkCoordinator,
    wire::WireMessage,
};
use schnorrkel::{PublicKey, SecretKey};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};
use std::{
    collections::HashSet,
    error::Error,
    fs,
    process,
};
use tokio::sync::Mutex;
use tracing::{error, info};

pub const SOCKET_PATH: &str = "/tmp/bcai_devnet.sock";
pub const PID_FILE: &str = "/tmp/bcai_devnet.pid";

type Mempool = Arc<Mutex<Vec<Transaction>>>;

pub struct Daemon {
    pid: String,
    p2p_service: Arc<Mutex<NetworkCoordinator>>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Mempool,
}

impl Daemon {
    fn new() -> Self {
        let blockchain = Arc::new(Mutex::new(Blockchain::new(Default::default())));
        let mempool = Arc::new(Mutex::new(HashSet::new()));

        Self {
            pid: process::id().to_string(),
            p2p_service: Arc::new(Mutex::new(NetworkCoordinator::new(blockchain.clone(), mempool.clone()).await)),
            blockchain,
            mempool,
        }
    }

    fn handle_command(&mut self, stream: &mut UnixStream) -> Result<String, Box<dyn Error>> {
        let mut cmd_bytes = Vec::new();
        if let Err(e) = stream.read_to_end(&mut cmd_bytes).await {
            error!("Failed to read command from socket: {}", e);
            return Ok("Failed to read command".to_string());
        }

        let cmd: P2pCommands = match bincode::deserialize(&cmd_bytes) {
            Ok(cmd) => cmd,
            Err(e) => {
                let response = format!("Failed to deserialize command: {}", e);
                if let Err(write_err) = stream.write_all(response.as_bytes()).await {
                    error!("Failed to write error response to socket: {}", write_err);
                }
                return Ok(response);
            }
        };

        let response = self.handle_daemon_command(cmd, self.blockchain.clone(), self.mempool.clone(), self.p2p_service.clone())?;
        
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            error!("Failed to write response to socket: {}", e);
        }
        Ok(response)
    }

    fn validate_and_add_to_mempool(&self, tx: Transaction) -> Result<(), Box<dyn Error>> {
        let chain = self.blockchain.lock().unwrap();

        // 1. Verify signature
        if !tx.verify_signature() {
            return Err("Invalid signature".into());
        }

        // 2. Check nonce
        let expected_nonce = chain.state.get_nonce(&tx.from) + 1;
        if tx.nonce != expected_nonce {
            return Err(format!("Invalid nonce. Expected {}, got {}", expected_nonce, tx.nonce).into());
        }

        // 3. Check balance
        let balance = chain.state.get_balance(&tx.from);
        let total_cost = tx.amount + tx.fee;
        if balance < total_cost {
            return Err(format!("Insufficient funds. Have {}, need {}", balance, total_cost).into());
        }

        // Add to mempool
        self.mempool.lock().unwrap().insert(tx);

        Ok(())
    }
}

/// The main entry point for the daemon process.
pub async fn daemon_main() {
    // Write the PID to the file after the daemon has successfully started.
    if let Err(e) = fs::write(PID_FILE, process::id().to_string()) {
        error!("Failed to write PID file: {}", e);
        return;
    }
    
    let mut daemon = Daemon::new();

    // Set up the Unix socket for CLI communication
    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to socket {}: {}", SOCKET_PATH, e);
            error!("Is the daemon already running? If not, try 'rm {}'", SOCKET_PATH);
            return;
        }
    };
    info!("Daemon listening on socket: {}", SOCKET_PATH);

    // Start the network coordinator
    let coordinator_clone = daemon.p2p_service.lock().await.clone();
    tokio::spawn(async move {
        coordinator_clone.run().await;
    });

    // Main command loop: listen for commands from the CLI via the socket
    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let response = daemon.handle_command(&mut stream).await?;
            if let Err(e) = stream.write_all(response.as_bytes()).await {
                error!("Failed to write response to socket: {}", e);
            }
        }
    }
}

/// Handles a single command received by the daemon.
async fn handle_daemon_command(
    command: P2pCommands,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Mempool,
    coordinator: Arc<Mutex<NetworkCoordinator>>,
) -> Result<String, Box<dyn Error>> {
    match command {
        P2pCommands::Info => {
            let bc = blockchain.lock().unwrap();
            if let Some(last_block) = bc.get_last_block() {
                Ok(format!(
                    "Chain Info:\n  Length: {}\n  Last Block Hash: {}",
                    bc.blocks.len(),
                    last_block.hash
                ))
            } else {
                Ok("Chain Info:\n  The blockchain is empty.".to_string())
            }
        }
        P2pCommands::Mine => {
            info!("Received 'mine' command.");
            let miner_pubkey = DEV_GENESIS_PUBKEY.to_string();

            let new_block = match miner::mine_block(miner_pubkey, blockchain.clone(), mempool.clone()) {
                Ok(block) => block,
                Err(e) => return Ok(format!("Error creating block: {}", e)),
            };
            let block_hash = new_block.hash.clone();
            let num_txs = new_block.transactions.len();
            let total_fees: u64 = new_block.transactions.iter().map(|tx| tx.fee).sum();
            let miner_reward = BLOCK_REWARD.saturating_add(total_fees);

            let included_txs = new_block.transactions.clone();

            match blockchain.lock().unwrap().add_block(new_block) {
                Ok(_) => {
                    info!("Successfully added locally mined block: {}", block_hash);
                    prune_mempool(&mempool, &blockchain, &included_txs);
                }
                Err(e) => {
                    let err_msg = format!("Failed to add locally mined block: {}", e);
                    error!("{}", err_msg);
                    return Ok(err_msg);
                }
            }
            
            // We must refetch the block to broadcast it, as it was moved into add_block
            let latest_block = blockchain.lock().unwrap().get_last_block().unwrap().clone();
            let mut coord = coordinator.lock().await;
            coord.broadcast(WireMessage::Block(latest_block)).await?;

            Ok(format!(
                "Success! Mined and broadcast new block #{}:\n  Hash: {}\n  Transactions: {}\n  Miner Reward: {} ({} base + {} fees)",
                blockchain.lock().unwrap().blocks.len() - 1,
                block_hash,
                num_txs,
                miner_reward,
                BLOCK_REWARD,
                total_fees
            ))
        }
        P2pCommands::Tx { tx_command } => match tx_command {
            TxCommands::Create {
                from_secret_key_file,
                to_pubkey,
                amount,
                fee,
                nonce,
            } => {
                let secret_key_bytes = match std::fs::read(from_secret_key_file) {
                    Ok(bytes) => bytes,
                    Err(e) => return Ok(format!("Failed to read secret key file: {}", e)),
                };
                let secret_key = match SecretKey::from_bytes(&secret_key_bytes) {
                    Ok(sk) => sk,
                    Err(_) => return Ok("Invalid secret key file format".to_string()),
                };
                let to_pk_bytes = match hex::decode(&to_pubkey) {
                    Ok(bytes) => bytes,
                    Err(_) => return Ok("Invalid recipient public key hex".to_string()),
                };
                let to_public_key = match PublicKey::from_bytes(&to_pk_bytes) {
                    Ok(pk) => pk,
                    Err(_) => return Ok("Invalid recipient public key".to_string()),
                };

                let tx = Transaction::new_transfer(&secret_key, to_public_key, amount, fee, nonce.unwrap());
                
                if let Err(e) = validate_transaction_for_mempool(&tx, &blockchain, &mempool) {
                    return Ok(format!("Transaction is invalid: {}", e));
                }

                let tx_hash = tx.hash();
                
                mempool.lock().unwrap().push(tx.clone());
                
                let coord = coordinator.lock().await;
                coord.broadcast(WireMessage::Transaction(tx)).await?;

                Ok(format!("Submitted transaction {} to network.", tx_hash))
            }
        },
        P2pCommands::Account { account_command } => match account_command {
            AccountCommands::Nonce { pubkey } => {
                let bc = blockchain.lock().unwrap();
                let nonce = bc.state.get_nonce(&pubkey);
                Ok(format!("{}", nonce))
            }
        },
        _ => Ok("Command not yet implemented or invalid.".to_string()),
    }
}

/// Removes transactions included in a new block from the mempool
/// and purges any other transactions that are now invalid.
fn prune_mempool(
    mempool: &Mempool,
    blockchain: &Arc<Mutex<Blockchain>>,
    included_txs: &[Transaction],
) {
    let mut mempool_guard = mempool.lock().unwrap();
    let chain_guard = blockchain.lock().unwrap();

    let included_hashes: HashSet<_> = included_txs.iter().map(|tx| tx.hash()).collect();
    mempool_guard.retain(|tx| !included_hashes.contains(&tx.hash()));

    mempool_guard.retain(|tx| {
        validation::validate_transaction_stateful(tx, &chain_guard.state).is_ok()
    });

    info!(
        "Mempool pruned. Included: {}. Remaining: {}.",
        included_txs.len(),
        mempool_guard.len()
    );
}

/// Validates a transaction against the current chain state and mempool.
/// Returns an error message string if invalid.
fn validate_transaction_for_mempool(
    tx: &Transaction,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Mempool,
) -> Result<(), String> {
    let chain = blockchain.lock().unwrap();

    validation::validate_transaction_stateless(tx)
        .and_then(|_| validation::validate_transaction_stateful(tx, &chain.state))
        .map_err(|e| e.to_string())?;

    let mempool_guard = mempool.lock().unwrap();
    if mempool_guard.iter().any(|mempool_tx| mempool_tx.hash() == tx.hash()) {
        return Err("Transaction already in mempool".to_string());
    }

    Ok(())
}

fn read_secret_key(path: &str) -> Result<SecretKey, Box<dyn Error>> {
    let key_bytes = fs::read(path)?;
    SecretKey::from_bytes(&key_bytes)
        .map_err(|e| format!("Failed to create secret key from bytes: {}", e).into())
} 