//! Handles the logic for processing IPC commands received by the daemon.

use crate::cli::{AccountCommands, JobCommands, P2pCommands, TxCommands};
use runtime::{
    blockchain::{self, validation, Blockchain, Transaction},
    job::Job,
    miner,
    p2p_service::{P2PHandle, WireMessage},
};
use schnorrkel::{PublicKey, SecretKey};
use std::{
    collections::{HashSet, VecDeque},
    error::Error,
    fs,
    path::Path,
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{error, info};

type Mempool = Arc<Mutex<HashSet<Transaction>>>;
type JobQueue = Arc<Mutex<VecDeque<Job>>>;

/// Encapsulates the state and logic for handling commands.
pub struct CommandHandler {
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Mempool,
    job_queue: JobQueue,
    p2p_handle: P2PHandle,
    job_id_counter: u64,
}

impl CommandHandler {
    pub fn new(
        blockchain: Arc<Mutex<Blockchain>>,
        mempool: Mempool,
        job_queue: JobQueue,
        p2p_handle: P2PHandle,
    ) -> Self {
        Self {
            blockchain,
            mempool,
            job_queue,
            p2p_handle,
            job_id_counter: 0,
        }
    }

    /// The main entry point for processing a command.
    pub async fn handle_command(
        &mut self,
        command: P2pCommands,
    ) -> Result<String, Box<dyn Error>> {
        match command {
            P2pCommands::Info => self.info().await,
            P2pCommands::Mine => self.mine().await,
            P2pCommands::Tx { tx_command } => self.handle_tx_command(tx_command).await,
            P2pCommands::Account { account_command } => {
                self.handle_account_command(account_command).await
            }
            P2pCommands::Job { job_command } => self.handle_job_command(job_command).await,
            P2pCommands::Peers | P2pCommands::Send { .. } => {
                Ok("This command is not handled by the daemon's command handler.".to_string())
            }
            _ => Ok("Command not yet implemented or invalid.".to_string()),
        }
    }

    async fn info(&self) -> Result<String, Box<dyn Error>> {
        let bc = self.blockchain.lock().await;
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

    async fn mine(&mut self) -> Result<String, Box<dyn Error>> {
        info!("Received 'mine' command.");
        let miner_pubkey = blockchain::constants::DEV_PUBLIC_KEY.to_string();

        let new_block = match miner::mine_block(
            miner_pubkey,
            self.blockchain.clone(),
            self.mempool.clone(),
            self.job_queue.clone(),
        )
        .await
        {
            Ok(block) => block,
            Err(e) => return Ok(format!("Error creating block: {}", e)),
        };
        let block_hash = new_block.hash.clone();
        let num_txs = new_block.transactions.len();
        let total_fees: u64 = new_block.transactions.iter().map(|tx| tx.fee).sum();
        let miner_reward = blockchain::constants::BLOCK_REWARD.saturating_add(total_fees);

        let included_txs = new_block.transactions.clone();
        let block_to_broadcast = new_block.clone();

        match self.blockchain.lock().await.add_block(new_block) {
            Ok(_) => {
                info!("Successfully added locally mined block: {}", block_hash);
                self.prune_mempool(&included_txs).await;
            }
            Err(e) => {
                let err_msg = format!("Failed to add locally mined block: {}", e);
                error!("{}", err_msg);
                return Ok(err_msg);
            }
        }

        let message = WireMessage::Block(block_to_broadcast);
        let topic = libp2p::gossipsub::IdentTopic::new("bcai_global");
        self.p2p_handle
            .send_message(topic, serde_json::to_vec(&message)?)
            .await?;

        Ok(format!(
            "Success! Mined and broadcast new block #{}:\n  Hash: {}\n  Transactions: {}\n  Miner Reward: {} ({} base + {} fees)",
            self.blockchain.lock().await.blocks.len() - 1,
            block_hash,
            num_txs,
            miner_reward,
            blockchain::constants::BLOCK_REWARD,
            total_fees
        ))
    }

    async fn handle_tx_command(&mut self, tx_command: TxCommands) -> Result<String, Box<dyn Error>> {
        match tx_command {
            TxCommands::Create {
                from_secret_key_file,
                to_pubkey,
                amount,
                fee,
                nonce,
            } => {
                let secret_key = self.read_secret_key(&from_secret_key_file)?;

                let to_pk_bytes = hex::decode(&to_pubkey)
                    .map_err(|_| "Invalid recipient public key hex")?;
                let to_public_key = PublicKey::from_bytes(&to_pk_bytes)
                    .map_err(|_| "Invalid recipient public key")?;

                let tx = Transaction::new_transfer(
                    &secret_key,
                    to_public_key,
                    amount,
                    fee,
                    nonce.unwrap(),
                );

                self.validate_transaction_for_mempool(&tx).await?;

                let tx_hash = tx.hash();
                self.mempool.lock().await.insert(tx.clone());

                let message = WireMessage::Transaction(tx);
                let topic = libp2p::gossipsub::IdentTopic::new("bcai_global");
                self.p2p_handle
                    .send_message(topic, serde_json::to_vec(&message)?)
                    .await?;

                Ok(format!("Submitted transaction {} to network.", tx_hash))
            }
        }
    }

    async fn handle_account_command(
        &self,
        account_command: AccountCommands,
    ) -> Result<String, Box<dyn Error>> {
        match account_command {
            AccountCommands::Nonce { pubkey } => {
                let bc = self.blockchain.lock().await;
                let nonce = bc.state.get_nonce(&pubkey);
                Ok(format!("{}", nonce))
            }
        }
    }

    async fn handle_job_command(&mut self, job_command: JobCommands) -> Result<String, Box<dyn Error>> {
        match job_command {
            JobCommands::Submit {
                model_id,
                dataset_id,
                iterations,
            } => {
                let job_id = self.job_id_counter;
                self.job_id_counter += 1;

                let job = Job::new(job_id, model_id, dataset_id, iterations);
                self.job_queue.lock().await.push_back(job.clone());

                info!("Added new job to queue: {:?}", job);
                Ok(format!("Submitted job with ID: {}", job_id))
            }
        }
    }

    // --- Private Helper Methods ---

    async fn prune_mempool(&self, included_txs: &[Transaction]) {
        let mut mempool_guard = self.mempool.lock().await;
        let chain_guard = self.blockchain.lock().await;

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

    async fn validate_transaction_for_mempool(&self, tx: &Transaction) -> Result<(), String> {
        let chain = self.blockchain.lock().await;

        validation::validate_transaction_stateless(tx)
            .and_then(|_| validation::validate_transaction_stateful(tx, &chain.state))
            .map_err(|e| e.to_string())?;

        let mempool_guard = self.mempool.lock().await;
        if mempool_guard
            .iter()
            .any(|mempool_tx| mempool_tx.hash() == tx.hash())
        {
            return Err("Transaction already in mempool".to_string());
        }

        Ok(())
    }

    fn read_secret_key(&self, path: &Path) -> Result<SecretKey, Box<dyn Error>> {
        let key_bytes = fs::read(path)?;
        SecretKey::from_bytes(&key_bytes)
            .map_err(|e| format!("Failed to create secret key from bytes: {}", e).into())
    }
} 