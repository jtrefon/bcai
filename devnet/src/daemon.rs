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
    fs,
    process,
};

pub const SOCKET_PATH: &str = "/tmp/bcai_devnet.sock";
pub const PID_FILE: &str = "/tmp/bcai_devnet.pid";

/// The main entry point for the daemon process.
pub async fn daemon_main() {
    // Write the PID to the file after the daemon has successfully started.
    if let Err(e) = fs::write(PID_FILE, process::id().to_string()) {
        error!("Failed to write PID file: {}", e);
        return;
    }
    
    // Initialize the blockchain and mempool
    let blockchain = Arc::new(Mutex::new(Blockchain::new(Default::default())));
    let mempool = Arc::new(Mutex::new(Vec::<Transaction>::new()));

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
    let coordinator =
        Arc::new(Mutex::new(NetworkCoordinator::new(blockchain.clone(), mempool.clone()).await));

    // Run the coordinator's event loop in a separate task
    let coordinator_clone = coordinator.clone();
    tokio::spawn(async move {
        coordinator_clone.lock().await.run().await;
    });

    // Main command loop: listen for commands from the CLI via the socket
    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut cmd_bytes = Vec::new();
            if let Err(e) = stream.read_to_end(&mut cmd_bytes).await {
                error!("Failed to read command from socket: {}", e);
                continue;
            }

            let cmd: P2pCommands = match bincode::deserialize(&cmd_bytes) {
                Ok(cmd) => cmd,
                Err(e) => {
                    let response = format!("Failed to deserialize command: {}", e);
                    if let Err(write_err) = stream.write_all(response.as_bytes()).await {
                        error!("Failed to write error response to socket: {}", write_err);
                    }
                    continue;
                }
            };

            let response =
                handle_daemon_command(cmd, blockchain.clone(), mempool.clone(), coordinator.clone())
                    .await;
            
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
    mempool: Arc<Mutex<Vec<Transaction>>>,
    coordinator: Arc<Mutex<NetworkCoordinator>>,
) -> String {
    match command {
        P2pCommands::Peers => "Peers command not yet implemented.".to_string(),
        P2pCommands::Send { .. } => "Send command not yet implemented.".to_string(),
        P2pCommands::Mine => {
            info!("Received 'mine' command.");
            let miner_pubkey = DEV_GENESIS_PUBKEY.to_string();

            let new_block = match miner::mine_block(miner_pubkey, blockchain.clone(), mempool) {
                Ok(block) => block,
                Err(e) => return format!("Error creating block: {}", e),
            };

            let block_hash = new_block.hash.clone();
            info!("Successfully mined new block: {}", block_hash);

            // Add the new block to our local chain before broadcasting.
            {
                let mut bc = blockchain.lock().unwrap();
                if let Err(e) = bc.add_block(new_block.clone()) {
                    let err_msg = format!("Failed to add locally mined block: {}", e);
                    error!("{}", err_msg);
                    return err_msg;
                }
            }

            // Broadcast the new block to the network.
            let mut coord = coordinator.lock().await;
            coord.broadcast(WireMessage::Block(new_block)).await;

            format!("Mined and broadcast new block: {}", block_hash)
        }
        P2pCommands::Tx { tx_command } => match tx_command {
            TxCommands::Create {
                from_secret_key_file,
                to_pubkey,
                amount,
                nonce,
                fee,
            } => {
                let secret_key_bytes = match std::fs::read(from_secret_key_file) {
                    Ok(bytes) => bytes,
                    Err(e) => return format!("Failed to read secret key file: {}", e),
                };
                let secret_key = match SecretKey::from_bytes(&secret_key_bytes) {
                     Ok(sk) => sk,
                     Err(_) => return "Invalid secret key file format".to_string(),
                };
                let to_pk_bytes = match hex::decode(&to_pubkey) {
                    Ok(bytes) => bytes,
                    Err(_) => return "Invalid recipient public key hex".to_string(),
                };
                let to_public_key = match PublicKey::from_bytes(&to_pk_bytes) {
                    Ok(pk) => pk,
                    Err(_) => return "Invalid recipient public key".to_string(),
                };

                let tx = Transaction::new_transfer(&secret_key, to_public_key, amount, fee, nonce);
                let tx_hash = tx.hash();
                
                // Add to our own mempool and broadcast to the network
                mempool.lock().unwrap().push(tx.clone());
                coordinator.lock().await.broadcast(WireMessage::Transaction(tx)).await;

                format!("Submitted transaction {} to network.", tx_hash)
            }
        },
        P2pCommands::Account { account_command } => match account_command {
            AccountCommands::Nonce { pubkey } => {
                let bc = blockchain.lock().unwrap();
                let nonce = bc.get_nonce(&pubkey);
                format!("Nonce for {}: {}", pubkey, nonce)
            }
        },
    }
} 