use crate::commands::{Cli, Commands};
use crate::error::DevnetError;
use crate::ops::{job_ops, ledger_ops, system_ops};
use clap::Parser;

/// Entry point invoked by `main.rs`.
pub fn run() -> Result<(), DevnetError> {
    use Commands::*;

    let cli = Cli::parse();

    match cli.command {
        Init => ledger_ops::init_ledger(),
        Mint { account, amount } => ledger_ops::mint(&account, amount),
        Transfer { from, to, amount } => ledger_ops::transfer(&from, &to, amount),
        Stake { account, amount } => ledger_ops::stake(&account, amount),
        Unstake { account, amount } => ledger_ops::unstake(&account, amount),
        Slash { account, amount } => ledger_ops::slash(&account, amount),
        Burn { account, amount } => ledger_ops::burn(&account, amount),
        Balance { account } => ledger_ops::balance(&account),
        Reputation { account } => ledger_ops::reputation(&account),
        AdjustRep { account, delta } => ledger_ops::adjust_reputation(&account, delta),
        Mine => system_ops::mine(),
        Train { size, seed, difficulty } => system_ops::train_pouw(size, seed, difficulty),
        Mnist => system_ops::train_mnist(),
        Neural { layers, epochs, samples } => system_ops::train_neural(layers, epochs, samples),
        Job { job } => job_ops::handle_job_command(job),
    }
} 