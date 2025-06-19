//! Clap-based command-line arguments for the devnet token/staking CLI.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "devnet", about = "Dev network CLI with token, staking and training features")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize ledger file
    Init,
    /// Mint tokens
    Mint { account: String, amount: u64 },
    /// Transfer tokens
    Transfer { from: String, to: String, amount: u64 },
    /// Stake tokens
    Stake { account: String, amount: u64 },
    /// Unstake tokens
    Unstake { account: String, amount: u64 },
    /// Slash staked tokens to the treasury
    Slash { account: String, amount: u64 },
    /// Burn tokens from an account
    Burn { account: String, amount: u64 },
    /// Show balances
    Balance { account: String },
    /// Show reputation score
    Reputation { account: String },
    /// Adjust reputation by delta
    AdjustRep { account: String, delta: i32 },
    /// Mine a block executing a dummy GPU task
    Mine,
    /// Run a PoUW training task
    Train { size: usize, seed: u64, difficulty: u32 },
    /// Train a logistic regression model on the digits dataset
    Mnist,
    /// Train a neural network
    Neural {
        #[arg(short, long, value_delimiter = ',')]
        layers: Vec<usize>,
        #[arg(short, long, default_value_t = 10)]
        epochs: usize,
        #[arg(short, long, default_value_t = 100)]
        samples: usize,
    },
    /// Manage jobs
    Job {
        #[command(subcommand)]
        job: JobCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum JobCommands {
    /// Post a new job
    Post { poster: String, description: String, reward: u64 },
    /// Assign a worker
    Assign { job_id: u64, worker: String },
    /// Complete a job
    Complete { job_id: u64 },
    /// List jobs
    List,
} 