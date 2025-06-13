use clap::{Parser, Subcommand};
use devnet::*;
use runtime::token::LedgerError;
use runtime::token::TokenLedger;

#[derive(Parser)]
#[command(name = "devnet")]
#[command(about = "Dev network CLI with token and staking", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    /// Show balances
    Balance { account: String },
    /// Mine a block executing a dummy GPU task
    Mine,
}

fn main() -> Result<(), DevnetError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            save_ledger(&TokenLedger::new())?;
            println!("Initialized ledger");
        }
        _ => {
            let mut ledger = load_ledger()?;
            match cli.command {
                Commands::Mint { account, amount } => {
                    mint(&mut ledger, &account, amount);
                }
                Commands::Transfer { from, to, amount } => {
                    match transfer(&mut ledger, &from, &to, amount) {
                        Ok(()) => {}
                        Err(LedgerError::InsufficientBalance) => {
                            println!("insufficient balance");
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(DevnetError::Io(std::io::Error::other(e.to_string())));
                        }
                    }
                }
                Commands::Stake { account, amount } => {
                    if let Err(e) = stake(&mut ledger, &account, amount) {
                        println!("{e}");
                    }
                }
                Commands::Unstake { account, amount } => {
                    if let Err(e) = unstake(&mut ledger, &account, amount) {
                        println!("{e}");
                    }
                }
                Commands::Balance { account } => {
                    println!(
                        "balance: {} staked: {}",
                        ledger.balance(&account),
                        ledger.staked(&account)
                    );
                }
                Commands::Mine => {
                    let input = vec![1.0f32, 2.0, 3.0, 4.0];
                    match runtime::gpu::double_numbers(&input) {
                        Ok(res) => println!("mined block with result: {:?}", res),
                        Err(e) => println!("gpu task failed: {e}"),
                    }
                }
                Commands::Init => unreachable!(),
            }
            save_ledger(&ledger)?;
        }
    }
    Ok(())
}
