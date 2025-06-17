//! The main entry point for the `devnet` command-line utility.

use clap::Parser;
use fork::{daemon, Fork};
use log::{error, info};
use std::{
    io::{Read, Write},
    net::UnixStream,
    process,
};

// Declare the new modules
mod cli;
mod daemon;

// Use the structs and functions from the new modules
use cli::{Cli, Commands};
use daemon::{daemon_main, SOCKET_PATH};

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
    info!("Starting devnet daemon...");
    if let Ok(Fork::Child) = daemon(false, false) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            daemon_main().await;
        });
    }
    info!("Daemon process started.");
}

fn stop_daemon() {
    // A simple way to stop is to remove the socket file. The daemon will then exit.
    // A more robust implementation would use a proper PID file or a 'stop' command.
    info!("Stopping devnet daemon...");
    match std::fs::remove_file(SOCKET_PATH) {
        Ok(_) => info!("Daemon stopped successfully."),
        Err(e) => error!("Failed to stop daemon by removing socket: {}. It may not be running.", e),
    }
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