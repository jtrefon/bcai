use clap::{Parser, Subcommand};
use schnorrkel::{SecretKey, PublicKey};
use std::fs;
use std::path::PathBuf;
use hex;

#[derive(Parser)]
#[command(name = "keygen")]
#[command(about = "BCAI Key Generation Tool for Schnorrkel Keys")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new keypair and save the secret key.
    Generate {
        /// Output file for the raw secret key
        #[arg(short, long, default_value = "wallet.key")]
        output: PathBuf,
    },
    /// Show the public key from a raw secret key file.
    Pubkey {
        /// Path to the raw secret key file
        #[arg(short, long, default_value = "wallet.key")]
        secret_key_file: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output } => {
            generate_keypair(&output)?;
        }
        Commands::Pubkey { secret_key_file } => {
            show_public_key(&secret_key_file)?;
        }
    }

    Ok(())
}

fn generate_keypair(output_file: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Generating new Schnorrkel keypair...");
    
    let secret_key = SecretKey::generate();
    let public_key = secret_key.public_key();

    // Save the raw bytes of the secret key
    fs::write(output_file, secret_key.to_bytes())?;
    
    println!("✅ Keypair generated successfully!");
    println!("📄 Secret key saved to: {}", output_file.display());
    println!("🔑 Public key (hex): {}", hex::encode(public_key.to_bytes()));
    println!("\n⚠️  Keep the secret key file safe and private!");
    
    Ok(())
}

fn show_public_key(secret_key_file: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !secret_key_file.exists() {
        return Err(format!("Secret key file not found: {}", secret_key_file.display()).into());
    }

    let secret_key_bytes = fs::read(secret_key_file)?;
    let secret_key = SecretKey::from_bytes(&secret_key_bytes)
        .map_err(|_| "Invalid secret key file format")?;
    
    let public_key = secret_key.public_key();

    println!("🔑 Public Key (hex): {}", hex::encode(public_key.to_bytes()));
    
    Ok(())
} 