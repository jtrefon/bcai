//! The main entry point for the `devnet` command-line utility.

use clap::Parser;
use fork::{daemon, Fork};
use log::{error, info};
use std::{
    fs,
    io::{Read, Write},
    net::UnixStream,
    process,
    time::Duration,
};
use std::sync::{Arc, Mutex};
use hex;

// Declare the new modules
mod cli;
mod daemon;

// Use the structs and functions from the new modules
use cli::{Cli, Commands};
use daemon::{daemon_main, PID_FILE, SOCKET_PATH};

#[derive(Debug, Subcommand)]
pub enum TxCommands {
    /// Creates and broadcasts a new transaction.
    Create {
        #[clap(long)]
        from_secret_key_file: String,
        #[clap(long)]
        to_pubkey: String,
        #[clap(long)]
        amount: u64,
        #[clap(long, default_value_t = 10)]
        fee: u64,
        /// If omitted, the nonce will be fetched automatically.
        #[clap(long)]
        nonce: Option<u64>,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum P2pCommands {
    /// Get information about the blockchain.
    Info,
    /// Mine a new block.
    Mine,
    /// Commands for managing transactions.
    #[clap(subcommand)]
    Tx(TxCommands),
    /// Commands for managing accounts.
    #[clap(subcommand)]
    Account(AccountCommands),
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum AccountCommands {
    /// Get the current nonce for an account.
    Nonce {
        #[clap(long)]
        pubkey: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            start_daemon();
        }
        Commands::Stop => {
            stop_daemon();
        }
        Commands::P2p(p2p_command) => {
            let mut stream = connect_to_daemon().await?;

            // Special handling for tx create to fetch nonce automatically
            let final_command = if let P2pCommands::Tx(TxCommands::Create {
                from_secret_key_file,
                to_pubkey,
                amount,
                fee,
                nonce: None, // Only if nonce is NOT provided
            }) = p2p_command
            {
                // 1. Get the public key from the secret key file
                let secret_key_bytes = std::fs::read(&from_secret_key_file)?;
                let secret_key = SecretKey::from_bytes(&secret_key_bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
                let pubkey = hex::encode(secret_key.public_key().to_bytes());

                // 2. Send a request to the daemon to get the current nonce
                println!("Nonce not provided. Fetching from daemon for pubkey: {}", pubkey);
                let nonce_cmd = P2pCommands::Account(AccountCommands::Nonce { pubkey });
                let nonce_cmd_bytes = bincode::serialize(&nonce_cmd).unwrap();

                // Need a new connection for this separate request
                let mut nonce_stream = connect_to_daemon().await?;
                nonce_stream.write_all(&nonce_cmd_bytes).await?;
                nonce_stream.shutdown().await?; // End write half to signal end of request
                let mut response_bytes = Vec::new();
                nonce_stream.read_to_end(&mut response_bytes).await?;
                let response_str = String::from_utf8(response_bytes)?;
                let fetched_nonce: u64 = response_str.trim().parse().map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to parse nonce from daemon response: {}", e),
                    )
                })?;
                println!("Fetched nonce: {}", fetched_nonce);

                // 3. Create a new command with the fetched nonce
                P2pCommands::Tx(TxCommands::Create {
                    from_secret_key_file,
                    to_pubkey,
                    amount,
                    fee,
                    nonce: Some(fetched_nonce),
                })
            } else {
                p2p_command
            };

            let cmd_bytes = bincode::serialize(&final_command).unwrap();
            stream.write_all(&cmd_bytes).await?;
            stream.shutdown().await?; // End write half to signal end of request

            let mut response_bytes = Vec::new();
            stream.read_to_end(&mut response_bytes).await?;
            let response_str = String::from_utf8(response_bytes)?;
            println!("{}", response_str);
        }
    }

    Ok(())
}

fn start_daemon() {
    if std::path::Path::new(PID_FILE).exists() {
        eprintln!("Daemon is already running. Use 'devnet stop' first.");
        return;
    }

    info!("Starting devnet daemon...");
    if let Ok(Fork::Child) = daemon(false, false) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            daemon_main().await;
        });
        // The child process will exit here.
    }
    // Give the daemon a moment to start and write its PID file.
    std::thread::sleep(Duration::from_millis(500));
    if std::path::Path::new(PID_FILE).exists() {
        info!("Daemon process started successfully.");
    } else {
        error!("Failed to start daemon process.");
    }
}

fn stop_daemon() {
    info!("Stopping devnet daemon...");
    let pid_path = std::path::Path::new(PID_FILE);
    if !pid_path.exists() {
        error!("Daemon is not running (PID file not found).");
        // Clean up socket file just in case it's orphaned.
        let _ = fs::remove_file(SOCKET_PATH);
        return;
    }

    match fs::read_to_string(pid_path) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
                info!("Sent SIGTERM to daemon process (PID {}).", pid);
            } else {
                error!("Invalid PID found in PID file: {}", pid_str);
            }
        }
        Err(e) => error!("Failed to read PID file: {}", e),
    }

    // Clean up both files.
    let _ = fs::remove_file(PID_FILE);
    let _ = fs::remove_file(SOCKET_PATH);
    info!("Daemon stopped and resources cleaned up.");
}

async fn connect_to_daemon() -> Result<UnixStream, Box<dyn std::error::Error>> {
    UnixStream::connect(SOCKET_PATH).map_err(|e| {
        format!(
            "Failed to connect to daemon socket (is it running?): {}",
            e
        )
    })
}

async fn handle_daemon_command(handle: P2PHandle, command: P2pCommands, network_coordinator: Arc<Mutex<NetworkCoordinator>>) -> String {
    match command {
        P2pCommands::Peers => match handle.get_peers().await {
            Ok(peers) => format!("Connected peers: {:?}", peers),
            Err(e) => format!("Error getting peers: {}", e),
        },
        P2pCommands::Send { topic, message } => {
            match handle.send_message(topic, message.into_bytes()).await {
                Ok(_) => "Message sent successfully.".to_string(),
                Err(e) => format!("Error: {}", e),
            }
        }
        P2pCommands::Mine => {
            let mut coordinator = network_coordinator.lock().await;
            let mut blockchain = coordinator.blockchain.lock().unwrap();

            // 1. Get the current tip of the chain
            let tip = blockchain.get_tip();
            let new_height = tip.height + 1;
            let prev_hash = tip.hash.clone();
            let difficulty = blockchain.calculate_next_difficulty();

            // 2. Create a dummy task and solution for PoUW
            let task = runtime::pouw::generate_task(1, 1);
            let solution = runtime::pouw::Solution {
                nonce: 123,
                result: vec![],
                computation_time: 1,
            };
            
            // 3. Create a new block
            let new_block = runtime::blockchain::Block::new(
                new_height,
                prev_hash,
                vec![], // No transactions for now
                difficulty,
                "devnet-miner".to_string(),
                task,
                solution,
            );

            // 4. Add the block to our own chain
            let block_hash = new_block.hash.clone();
            if let Err(e) = blockchain.add_block(new_block.clone()) {
                return format!("Error adding block to local chain: {}", e);
            }

            // 5. Broadcast the new block to the network
            let wire_message = WireMessage::Block(new_block);
            let message_bytes = bincode::serialize(&wire_message).unwrap();

            match handle.send_message("bcai_global".to_string(), message_bytes).await {
                Ok(_) => format!("Mined and broadcast new block: {}", block_hash),
                Err(e) => format!("Error broadcasting block: {}", e),
            }
        }
    }
}

async fn listen_for_commands(p2p_handle: P2PHandle, network_coordinator: Arc<Mutex<NetworkCoordinator>>) {
    // Clean up old socket if it exists
    let listener = UnixStream::bind(SOCKET_PATH).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let handle = p2p_handle.clone();
                let coordinator_clone = network_coordinator.clone();
                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    if stream.read_to_end(&mut buffer).is_ok() {
                        if let Ok(command) = bincode::deserialize::<P2pCommands>(&buffer) {
                            let response = handle_daemon_command(handle, command, coordinator_clone).await;
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }
                });
            }
        }
    }
}

rt.block_on(async {
    let p2p_config = P2PConfig {
        // ... existing code ...
    };

    let local_node = UnifiedNode::new(NodeRole::Validator, NodeCapability::Light);
    let coordinator = Arc::new(Mutex::new(NetworkCoordinator::new(local_node)));
    let (p2p_service, p2p_handle) = P2PService::new(p2p_config, coordinator.clone()).await.unwrap();

    // Run the P2P service and the command listener concurrently
    tokio::spawn(p2p_service.run());
    listen_for_commands(p2p_handle, coordinator).await;
});

// Cleanup on exit (this part is tricky, relies on signal handling in a real app) 