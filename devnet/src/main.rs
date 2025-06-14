use clap::{Parser, Subcommand};
use devnet::*;

#[derive(Parser)]
#[command(name = "devnet")]
#[command(about = "Simplified devnet CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the token ledger
    Init,
    /// Mint tokens to an account
    Mint { account: String, amount: u64 },
    /// Show account balance
    Balance { account: String },
}

fn main() -> Result<(), DevnetError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            save_ledger(&TokenLedger::new())?;
            println!("Ledger initialized");
        }
        Commands::Mint { account, amount } => {
            let mut ledger = load_ledger()?;
            mint(&mut ledger, &account, amount);
            save_ledger(&ledger)?;
            println!("Minted {amount} to {account}");
        }
        Commands::Balance { account } => {
            let ledger = load_ledger()?;
            println!("Balance of {account}: {}", ledger.balance(&account));
        }
    }
    Ok(())
}
