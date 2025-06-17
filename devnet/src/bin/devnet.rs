//! Devnet CLI for BCAI
//!
//! This CLI tool is used to start, stop, and interact with a local BCAI development node.
//! It runs the P2P service as a background daemon process and uses a Unix socket for communication.

use clap::{Parser, Subcommand};
use runtime::{
    node::{NodeCapability, NodeRole, UnifiedNode},
    p2p_service::{P2PConfig, P2PHandle, P2PService},
};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use runtime::network::NetworkCoordinator;

const SOCKET_PATH: &str = "/tmp/bcai_devnet.sock";
const PID_FILE: &str = "/tmp/bcai_devnet.pid";

#[derive(Parser)]
#[command(name = "devnet")]
#[command(about = "BCAI Development Network Node")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a development network node in the background
    Start {
        /// Port to listen on for P2P connections
        #[arg(short, long, default_value = "4001")]
        p2p_port: u16,

        /// Optional peer to connect to (full multiaddress)
        #[arg(long)]
        peer: Option<String>,
    },
    /// Stop the background devnet node
    Stop,
    /// Interact with the P2P network
    P2p {
        #[command(subcommand)]
        p2p_command: P2pCommands,
    },
    /// Generate genesis block (remains a standalone utility)
    Genesis {
        /// Output file for genesis block
        #[arg(short, long, default_value = "genesis.json")]
        output: String,
    },
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
enum P2pCommands {
    /// List connected peers
    Peers,
    /// Send a message on a topic
    Send { topic: String, message: String },
    /// Mine a new block and broadcast it
    Mine,
}

// Main entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { p2p_port, peer } => {
            start_daemon(p2p_port, peer)?;
        }
        Commands::Stop => {
            stop_daemon()?;
        }
        Commands::P2p { p2p_command } => {
            send_command_to_daemon(&p2p_command)?;
        }
        Commands::Genesis { output } => {
            // This command doesn't need the daemon
            tokio::runtime::Runtime::new()?.block_on(generate_genesis_block(&output))?;
        }
    }

    Ok(())
}

// --- Daemon Control ---

fn start_daemon(p2p_port: u16, peer: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    if PathBuf::from(PID_FILE).exists() {
        eprintln!("Daemon already running. Use 'stop' first.");
        exit(1);
    }

    println!("Starting daemon...");

    // Fork the process
    if let Ok(0) = unsafe { libc::fork() } {
        // Child process (the daemon)
        // Detach from the terminal
        unsafe {
            libc::setsid();
            libc::close(libc::STDIN_FILENO);
            libc::close(libc::STDOUT_FILENO);
            libc::close(libc::STDERR_FILENO);
        }

        // Write PID file
        std::fs::write(PID_FILE, std::process::id().to_string())?;

        // Create and run the tokio runtime
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let p2p_config = P2PConfig {
                listen_port: p2p_port,
                bootstrap_peers: peer.into_iter().collect(),
                ..Default::default()
            };

            // 1. Create the node and the coordinator
            let local_node = UnifiedNode::new(NodeRole::Validator, NodeCapability::Light);
            let network_coordinator = Arc::new(Mutex::new(NetworkCoordinator::new(local_node)));

            // 2. Pass the shared coordinator to the P2P service
            let (p2p_service, p2p_handle) = P2PService::new(p2p_config, network_coordinator.clone())
                .await
                .unwrap();

            // Run the P2P service and the command listener concurrently
            tokio::spawn(p2p_service.run());
            // 3. Pass the shared coordinator to the command listener
            listen_for_commands(p2p_handle, network_coordinator).await;
        });

        // Cleanup on exit (this part is tricky, relies on signal handling in a real app)
        let _ = std::fs::remove_file(PID_FILE);
        let _ = std::fs::remove_file(SOCKET_PATH);
    } else {
        // Parent process
        // Wait a moment to see if the daemon started correctly
        std::thread::sleep(Duration::from_secs(1));
        if PathBuf::from(PID_FILE).exists() {
            println!("✅ Daemon started successfully.");
        } else {
            eprintln!("❌ Failed to start daemon.");
        }
    }
    Ok(())
}

fn stop_daemon() -> Result<(), Box<dyn std::error::Error>> {
    if !PathBuf::from(PID_FILE).exists() {
        eprintln!("Daemon not running.");
        return Ok(());
    }
    let pid_str = std::fs::read_to_string(PID_FILE)?;
    let pid: i32 = pid_str.parse()?;

    println!("Stopping daemon (PID: {})...", pid);
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    // Cleanup
    std::fs::remove_file(PID_FILE)?;
    std::fs::remove_file(SOCKET_PATH)?;
    println!("✅ Daemon stopped.");
    Ok(())
}

fn send_command_to_daemon(command: &P2pCommands) -> Result<(), Box<dyn std::error::Error>> {
    if !PathBuf::from(SOCKET_PATH).exists() {
        eprintln!("Daemon not running. Use 'start' first.");
        exit(1);
    }

    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    let command_bytes = bincode::serialize(command)?;
    stream.write_all(&command_bytes)?;

    // Read response
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("{}", response);

    Ok(())
}

// --- Daemon-side Logic ---

use serde::{Deserialize, Serialize};

async fn listen_for_commands(
    p2p_handle: P2PHandle,
    network_coordinator: Arc<Mutex<NetworkCoordinator>>,
) {
    // Clean up old socket if it exists
    if PathBuf::from(SOCKET_PATH).exists() {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }

    let listener = UnixListener::bind(SOCKET_PATH).unwrap();
    println!("[daemon] Listening for commands on {}", SOCKET_PATH);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let handle = p2p_handle.clone();
                let coordinator_clone = network_coordinator.clone();
                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    if stream.read_to_end(&mut buffer).is_ok() {
                        if let Ok(command) = bincode::deserialize::<P2pCommands>(&buffer) {
                            let response =
                                handle_daemon_command(handle, command, coordinator_clone).await;
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("[daemon] Command connection failed: {}", e);
            }
        }
    }
}

async fn handle_daemon_command(
    handle: P2PHandle,
    command: P2pCommands,
    network_coordinator: Arc<Mutex<NetworkCoordinator>>,
) -> String {
    match command {
        P2pCommands::Peers => match handle.get_peers().await {
            Ok(peers) => {
                let mut response = format!("Connected peers ({}):\n", peers.len());
                for peer in peers {
                    response.push_str(&format!("  - {}\n", peer));
                }
                response
            }
            Err(e) => format!("Error: {}", e),
        },
        P2pCommands::Send { topic, message } => {
            match handle.send_message(topic, message.into_bytes()).await {
                Ok(_) => "Message sent successfully.".to_string(),
                Err(e) => format!("Error: {}", e),
            }
        }
        P2pCommands::Mine => {
            let coordinator = network_coordinator.lock().await;
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

// Standalone utility function (no changes needed)
async fn generate_genesis_block(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Generating genesis block...");
    let genesis_json = serde_json::json!({ "creation_time": chrono::Utc::now() });
    std::fs::write(output, serde_json::to_string_pretty(&genesis_json)?)?;
    println!("✅ Genesis block generated: {}", output);
    Ok(())
} 