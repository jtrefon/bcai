use clap::Parser;
use keygen_lib::{generate_keypair, KeypairJson};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "keygen")]
#[command(about = "Generate an Ed25519 keypair", long_about = None)]
struct Cli {
    /// Optional output file path
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let pair: KeypairJson = generate_keypair();
    let json = serde_json::to_string_pretty(&pair)?;
    if let Some(path) = cli.output {
        fs::write(path, json)?;
    } else {
        println!("{}", json);
    }
    Ok(())
}
