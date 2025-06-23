//! Devnet daemon – IPC gateway + P2P bootstrap.
//!
//! `mod.rs` keeps the high-level orchestration logic below 100 LOC.  All
//! constants and shared type aliases live in `types.rs` so this file remains
//! lightweight.

mod types;

use crate::cli::P2pCommands;
use crate::command_handler::CommandHandler;
use runtime::{
    blockchain::{Blockchain, transaction::Transaction, constants::METRICS_ORACLE_PUB},
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
    let p2p_handle_clone = p2p_handle.clone();
    let blockchain_clone = blockchain.clone();
    tokio::spawn(async move { p2p_service.run().await });

    // Spawn periodic metrics publisher (every 60s)
    tokio::spawn(async move {
        use runtime::distributed_storage::allocation::{NodeMetrics, StoragePolicy};
        use schnorrkel::SecretKey;
        use libp2p::gossipsub::IdentTopic;
        use runtime::p2p_service::WireMessage;
        loop {
            // sleep first to allow network init
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            // Load oracle secret key from $HOME/.bcai/metrics_oracle.key
            let key_path = std::env::var("HOME").unwrap_or(".".into()) + "/.bcai/metrics_oracle.key";
            let sk_bytes = match std::fs::read(&key_path) {
                Ok(b) => b,
                Err(_) => {
                    error!("Metrics oracle key not found at {}", key_path);
                    continue;
                }
            };
            let secret_key = match SecretKey::from_bytes(&sk_bytes) {
                Ok(k) => k,
                Err(e) => { error!("Invalid oracle key: {}", e); continue; }
            };

            // Build NodeMetrics for this node (stub values)
            let metrics = vec![NodeMetrics {
                node_id: METRICS_ORACLE_PUB.to_string(),
                reputation: 0.9,
                free_capacity: 0.8,
                latency_ms: 20,
                region: "us".into(),
                energy_score: 0.7,
                utilisation: 0.2,
            }];

            // determine nonce
            let nonce = {
                let chain = blockchain_clone.lock().await;
                chain.state.get_nonce(METRICS_ORACLE_PUB)
            };

            let tx = Transaction::new_update_metrics_signed(&secret_key, metrics, nonce);

            // broadcast
            if let Err(e) = p2p_handle_clone
                .send_message(IdentTopic::new("bcai_global"), serde_json::to_vec(&WireMessage::Transaction(tx)).unwrap())
                .await
            {
                error!("Failed to broadcast metrics: {}", e);
            }
        }
    });

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