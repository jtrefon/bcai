//! Defines the command-line interface for the devnet.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Serialize, Deserialize, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
pub enum Commands {
    /// Starts the devnet daemon process in the background.
    Start,
    /// Stops the devnet daemon process.
    Stop,
    /// Sends a command to the running devnet daemon.
    P2p {
        #[command(subcommand)]
        p2p_command: P2pCommands,
    },
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
pub enum P2pCommands {
    /// List connected peers.
    Peers,
    /// Send a raw message on a topic (for debugging).
    Send { topic: String, message: String },
    /// Mine a new block and broadcast it.
    Mine,
    /// Manage transactions.
    Tx {
        #[command(subcommand)]
        tx_command: TxCommands,
    },
    /// Manage accounts.
    Account {
        #[command(subcommand)]
        account_command: AccountCommands,
    },
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
pub enum TxCommands {
    /// Create and broadcast a new transfer transaction.
    /// Use 'keygen' to create a keypair and 'p2p account nonce' to get the next nonce.
    Create {
        /// Path to the secret key file of the sender (e.g., 'wallet.key').
        #[arg(long)]
        from_secret_key_file: PathBuf,
        /// Public key of the recipient (hex-encoded).
        #[arg(long)]
        to_pubkey: String,
        /// Amount to transfer.
        #[arg(long)]
        amount: u64,
        /// The next valid nonce for the sender's account.
        #[arg(long)]
        nonce: u64,
        /// Fee for the transaction.
        #[arg(long, default_value_t = 1)]
        fee: u64,
    },
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
pub enum AccountCommands {
    /// Get the current nonce for an account.
    Nonce {
        /// Public key of the account (hex-encoded).
        pubkey: String,
    },
} 