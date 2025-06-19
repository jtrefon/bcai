use crate::cli::{AccountCommands, JobCommands, P2pCommands, TxCommands};
use runtime::{
    blockchain::{self, validation, Blockchain, Transaction},
    job::Job,
    miner,
    p2p_service::{P2PHandle, WireMessage},
};
use std::{
    collections::{HashSet, VecDeque},
    error::Error,
    sync::Arc,
};
use tokio::sync::Mutex;

/// Shared alias for pending transactions.
pub(super) type Mempool = Arc<Mutex<HashSet<Transaction>>>;
/// Shared alias for queued compute jobs.
pub(super) type JobQueue = Arc<Mutex<VecDeque<Job>>>;

/// Central dispatcher for all CLI-originated commands.
///
/// This struct intentionally contains **no heavy business logic** – it simply
/// delegates to specialized helper modules. Each helper implements its own
/// `impl CommandHandler` block to extend this type with domain-specific
/// behavior. This keeps the core orchestrator minimal and focused on routing.
pub struct CommandHandler {
    pub(super) blockchain: Arc<Mutex<Blockchain>>, // shared chain state
    pub(super) mempool: Mempool,                   // pending transactions
    pub(super) job_queue: JobQueue,                // queued training jobs
    pub(super) p2p_handle: P2PHandle,              // network interface
    job_id_counter: u64,                           // monotonically increasing job id
}

impl CommandHandler {
    /// Construct a new command handler with shared state references.
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

    /// Entry point that performs top-level routing based on the parsed CLI command.
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
            P2pCommands::Peers | P2pCommands::Send { .. } => Ok(
                "This command is not handled by the daemon's command handler.".to_string(),
            ),
            _ => Ok("Command not yet implemented or invalid.".to_string()),
        }
    }
} 