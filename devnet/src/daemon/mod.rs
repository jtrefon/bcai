//! Devnet daemon – IPC gateway + P2P bootstrap.
//!
//! `mod.rs` keeps the high-level orchestration logic below 100 LOC.  All
//! constants and shared type aliases live in `types.rs` so this file remains
//! lightweight.

mod types;

use crate::cli::P2pCommands;
use crate::command_handler::CommandHandler;
use runtime::{
    blockchain::Blockchain,
    job::Job,
    p2p_service::{P2PConfig, P2PHandle, P2PService},
};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::Mutex,
};
use tracing::{error, info};

use types::*;

/// Spawn the background daemon task; writes its PID and processes incoming
/// socket commands.
pub async fn daemon_main() {
    if let Err(e) = std::fs::write(PID_FILE, std::process::id().to_string()) {
        error!("Failed to write PID file: {}", e);
        return;
    }

    // --- Service bootstrap -------------------------------------------------
    let blockchain = Arc::new(Mutex::new(Blockchain::new(Default::default())));
    let mempool: Mempool = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let job_queue: JobQueue = Arc::new(Mutex::new(std::collections::VecDeque::<Job>::new()));

    let (p2p_service, p2p_handle) = P2PService::new(P2PConfig::default())
        .await
        .expect("failed to create P2P service");
    tokio::spawn(async move { p2p_service.run().await });

    let mut command_handler = CommandHandler::new(
        blockchain,
        mempool,
        job_queue,
        p2p_handle,
    );

    // --- IPC socket --------------------------------------------------------
    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to socket {}: {}", SOCKET_PATH, e);
            error!("Is the daemon already running? If not, try 'rm {}'", SOCKET_PATH);
            return;
        }
    };
    info!("Daemon listening on socket: {}", SOCKET_PATH);

    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            if let Err(e) = handle_socket_command(&mut command_handler, &mut stream).await {
                error!("Error handling command: {}", e);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Internal helpers
// -----------------------------------------------------------------------------

async fn handle_socket_command(
    handler: &mut CommandHandler,
    stream: &mut UnixStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd_bytes = Vec::new();
    stream.read_to_end(&mut cmd_bytes).await?;

    let cmd: P2pCommands = bincode::deserialize(&cmd_bytes)?;
    let response = handler.handle_command(cmd).await?;
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

// Add re-exports for external consumers
pub use types::{PID_FILE, SOCKET_PATH}; 