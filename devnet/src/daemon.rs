//! The logic for the devnet daemon process.

use crate::cli::P2pCommands;
use crate::command_handler::CommandHandler;
use log::error;
use runtime::{
    blockchain::Blockchain,
    job::Job,
    p2p_service::{P2PConfig, P2PHandle, P2PService},
};
use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fs;
use std::process;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::Mutex,
};
use tracing::info;

pub const SOCKET_PATH: &str = "/tmp/bcai_devnet.sock";
pub const PID_FILE: &str = "/tmp/bcai_devnet.pid";

type Mempool = Arc<Mutex<HashSet<runtime::blockchain::Transaction>>>;
type JobQueue = Arc<Mutex<VecDeque<Job>>>;

pub struct Daemon {
    pid: String,
    command_handler: CommandHandler,
}

impl Daemon {
    async fn new() -> Self {
        let blockchain = Arc::new(Mutex::new(Blockchain::new(Default::default())));
        let mempool = Arc::new(Mutex::new(HashSet::new()));
        let job_queue = Arc::new(Mutex::new(VecDeque::new()));

        let (p2p_service, p2p_handle) = P2PService::new(P2PConfig::default()).await.unwrap();

        tokio::spawn(async move {
            p2p_service.run().await;
        });

        let command_handler = CommandHandler::new(
            blockchain.clone(),
            mempool.clone(),
            job_queue.clone(),
            p2p_handle.clone(),
        );

        Self {
            pid: process::id().to_string(),
            command_handler,
        }
    }

    async fn handle_socket_command(
        &mut self,
        stream: &mut UnixStream,
    ) -> Result<(), Box<dyn Error>> {
        let mut cmd_bytes = Vec::new();
        stream.read_to_end(&mut cmd_bytes).await?;

        let cmd: P2pCommands = match bincode::deserialize(&cmd_bytes) {
            Ok(cmd) => cmd,
            Err(e) => {
                let response = format!("Failed to deserialize command: {}", e);
                stream.write_all(response.as_bytes()).await?;
                return Err(e.into());
            }
        };

        let response = self.command_handler.handle_command(cmd).await?;

        stream.write_all(response.as_bytes()).await?;
        Ok(())
    }
}

/// The main entry point for the daemon process.
pub async fn daemon_main() {
    if let Err(e) = fs::write(PID_FILE, process::id().to_string()) {
        error!("Failed to write PID file: {}", e);
        return;
    }

    let mut daemon = Daemon::new().await;

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to socket {}: {}", SOCKET_PATH, e);
            error!("Is the daemon already running? If not, try 'rm {}'", SOCKET_PATH);
            return;
        }
    };
    info!("Daemon listening on socket: {}", SOCKET_PATH);

    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            if let Err(e) = daemon.handle_socket_command(&mut stream).await {
                error!("Error handling command: {}", e);
            }
        }
    }
} 