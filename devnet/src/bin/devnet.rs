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
        #[clap(long)]
        nonce: u64,
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
        Commands::P2p { p2p_command } => {
            handle_cli_command(p2p_command).await?;
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

async fn handle_cli_command(
    command: cli::P2pCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(SOCKET_PATH).map_err(|e| {
        format!(
            "Failed to connect to daemon socket (is it running?): {}",
            e
        )
    })?;

    let command_bytes = bincode::serialize(&command)?;
    stream.write_all(&command_bytes)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    println!("{}", response);
    Ok(())
} 