use clap::{Parser, Subcommand};
use keygen_lib::{generate_keypair, KeyPair};
use serde_json;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "keygen")]
#[command(about = "BCAI Key Generation and Management Tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new keypair
    Generate {
        /// Output file for private key
        #[arg(short, long, default_value = "private_key.json")]
        private_key: String,
        
        /// Output file for public key
        #[arg(short, long, default_value = "public_key.json")]
        public_key: String,
        
        /// Key name/identifier
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Show public key from private key file
    PublicKey {
        /// Private key file
        #[arg(short, long)]
        private_key: String,
    },
    /// Sign a message or file
    Sign {
        /// Private key file
        #[arg(short, long)]
        private_key: String,
        
        /// Message to sign (use --file for file input)
        #[arg(short, long)]
        message: Option<String>,
        
        /// File to sign
        #[arg(short, long)]
        file: Option<String>,
        
        /// Output signature file
        #[arg(short, long, default_value = "signature.hex")]
        output: String,
    },
    /// Verify a signature
    Verify {
        /// Public key file
        #[arg(short, long)]
        public_key: String,
        
        /// Message that was signed
        #[arg(short, long)]
        message: Option<String>,
        
        /// File that was signed
        #[arg(short, long)]
        file: Option<String>,
        
        /// Signature file
        #[arg(short, long)]
        signature: String,
    },
    /// List saved keys
    List,
    /// Show key information
    Info {
        /// Key file (private or public)
        key_file: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { private_key, public_key, name } => {
            generate_keys(&private_key, &public_key, name).await?;
        }
        Commands::PublicKey { private_key } => {
            show_public_key(&private_key).await?;
        }
        Commands::Sign { private_key, message, file, output } => {
            sign_data(&private_key, message, file, &output).await?;
        }
        Commands::Verify { public_key, message, file, signature } => {
            verify_signature(&public_key, message, file, &signature).await?;
        }
        Commands::List => {
            list_keys().await?;
        }
        Commands::Info { key_file } => {
            show_key_info(&key_file).await?;
        }
    }

    Ok(())
}

async fn generate_keys(
    private_key_file: &str,
    public_key_file: &str,
    name: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Generating new BCAI keypair...");
    
    let keypair = generate_keypair();
    let key_name = name.unwrap_or_else(|| "bcai-key".to_string());
    
    // Save private key
    let private_key_data = serde_json::json!({
        "name": key_name,
        "type": "Ed25519",
        "private_key": hex::encode(keypair.private_key.to_bytes()),
        "public_key": hex::encode(keypair.public_key.to_bytes()),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "version": "1.0"
    });
    
    fs::write(private_key_file, serde_json::to_string_pretty(&private_key_data)?)?;
    
    // Save public key
    let public_key_data = serde_json::json!({
        "name": key_name,
        "type": "Ed25519",
        "public_key": hex::encode(keypair.public_key.to_bytes()),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "version": "1.0"
    });
    
    fs::write(public_key_file, serde_json::to_string_pretty(&public_key_data)?)?;
    
    println!("✅ Keypair generated successfully!");
    println!("📄 Private key: {}", private_key_file);
    println!("📄 Public key: {}", public_key_file);
    println!("🏷️  Key name: {}", key_name);
    println!("🔑 Public key (hex): {}", hex::encode(keypair.public_key.to_bytes()));
    
    println!();
    println!("⚠️  SECURITY WARNING:");
    println!("   • Keep your private key file secure and never share it");
    println!("   • Back up your private key in multiple secure locations");
    println!("   • The public key can be shared safely with others");
    
    Ok(())
}

async fn show_public_key(private_key_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(private_key_file).exists() {
        return Err(format!("Private key file not found: {}", private_key_file).into());
    }

    let content = fs::read_to_string(private_key_file)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    
    let public_key_hex = data["public_key"]
        .as_str()
        .ok_or("Invalid private key file format")?;
    
    let key_name = data["name"]
        .as_str()
        .unwrap_or("unknown");

    println!("🔑 Public Key Information");
    println!("========================");
    println!("🏷️  Name: {}", key_name);
    println!("📱 Public Key: {}", public_key_hex);
    println!("🔐 Key Type: Ed25519");
    
    if let Some(created_at) = data["created_at"].as_str() {
        println!("📅 Created: {}", created_at);
    }

    Ok(())
}

async fn sign_data(
    private_key_file: &str,
    message: Option<String>,
    file: Option<String>,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(private_key_file).exists() {
        return Err(format!("Private key file not found: {}", private_key_file).into());
    }

    // Get data to sign
    let data_to_sign = match (message, file) {
        (Some(msg), None) => msg.into_bytes(),
        (None, Some(file_path)) => {
            if !Path::new(&file_path).exists() {
                return Err(format!("File not found: {}", file_path).into());
            }
            fs::read(&file_path)?
        }
        _ => return Err("Must specify either --message or --file".into()),
    };

    // Load private key
    let content = fs::read_to_string(private_key_file)?;
    let key_data: serde_json::Value = serde_json::from_str(&content)?;
    
    let private_key_hex = key_data["private_key"]
        .as_str()
        .ok_or("Invalid private key file format")?;
    
    let private_key_bytes = hex::decode(private_key_hex)?;
    let keypair = KeyPair::from_private_key_bytes(&private_key_bytes)?;

    // Sign the data
    let signature = keypair.sign(&data_to_sign);
    let signature_hex = hex::encode(signature.to_bytes());

    // Save signature
    let signature_data = serde_json::json!({
        "signature": signature_hex,
        "public_key": hex::encode(keypair.public_key.to_bytes()),
        "data_hash": hex::encode(sha2::Sha256::digest(&data_to_sign)),
        "signed_at": chrono::Utc::now().to_rfc3339(),
        "algorithm": "Ed25519"
    });

    fs::write(output, serde_json::to_string_pretty(&signature_data)?)?;

    println!("✅ Data signed successfully!");
    println!("📄 Signature file: {}", output);
    println!("🔏 Signature: {}", signature_hex);
    println!("🔑 Public key: {}", hex::encode(keypair.public_key.to_bytes()));

    Ok(())
}

async fn verify_signature(
    public_key_file: &str,
    message: Option<String>,
    file: Option<String>,
    signature_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(public_key_file).exists() {
        return Err(format!("Public key file not found: {}", public_key_file).into());
    }
    
    if !Path::new(signature_file).exists() {
        return Err(format!("Signature file not found: {}", signature_file).into());
    }

    // Get data to verify
    let data_to_verify = match (message, file) {
        (Some(msg), None) => msg.into_bytes(),
        (None, Some(file_path)) => {
            if !Path::new(&file_path).exists() {
                return Err(format!("File not found: {}", file_path).into());
            }
            fs::read(&file_path)?
        }
        _ => return Err("Must specify either --message or --file".into()),
    };

    // Load public key
    let public_key_content = fs::read_to_string(public_key_file)?;
    let public_key_data: serde_json::Value = serde_json::from_str(&public_key_content)?;
    
    let public_key_hex = public_key_data["public_key"]
        .as_str()
        .ok_or("Invalid public key file format")?;

    // Load signature
    let signature_content = fs::read_to_string(signature_file)?;
    let signature_data: serde_json::Value = serde_json::from_str(&signature_content)?;
    
    let signature_hex = signature_data["signature"]
        .as_str()
        .ok_or("Invalid signature file format")?;

    // Verify the signature
    let public_key_bytes = hex::decode(public_key_hex)?;
    let signature_bytes = hex::decode(signature_hex)?;
    
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
        .map_err(|e| format!("Invalid public key: {}", e))?;
    
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes)
        .map_err(|e| format!("Invalid signature: {}", e))?;

    match public_key.verify(&data_to_verify, &signature) {
        Ok(_) => {
            println!("✅ Signature verification PASSED!");
            println!("🔐 The signature is valid and the data is authentic");
            
            if let Some(signed_at) = signature_data["signed_at"].as_str() {
                println!("📅 Signed at: {}", signed_at);
            }
        }
        Err(_) => {
            println!("❌ Signature verification FAILED!");
            println!("⚠️  The signature is invalid or the data has been tampered with");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn list_keys() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 BCAI Keys");
    println!("============");
    
    let current_dir = std::env::current_dir()?;
    let mut found_keys = false;

    // Look for key files in current directory
    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(key_type) = data.get("type") {
                        if key_type == "Ed25519" {
                            found_keys = true;
                            let name = data["name"].as_str().unwrap_or("unknown");
                            let is_private = data.get("private_key").is_some();
                            let key_kind = if is_private { "Private" } else { "Public" };
                            let icon = if is_private { "🔐" } else { "🔑" };
                            
                            println!("{} {} Key: {} ({})", icon, key_kind, name, path.display());
                            
                            if let Some(created_at) = data["created_at"].as_str() {
                                println!("   📅 Created: {}", created_at);
                            }
                            
                            if let Some(public_key) = data["public_key"].as_str() {
                                println!("   🔑 Public: {}...{}", &public_key[..16], &public_key[public_key.len()-8..]);
                            }
                            println!();
                        }
                    }
                }
            }
        }
    }

    if !found_keys {
        println!("No BCAI keys found in current directory.");
        println!("Use 'keygen generate' to create a new keypair.");
    }

    Ok(())
}

async fn show_key_info(key_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(key_file).exists() {
        return Err(format!("Key file not found: {}", key_file).into());
    }

    let content = fs::read_to_string(key_file)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;

    if data.get("type") != Some(&serde_json::Value::String("Ed25519".to_string())) {
        return Err("Not a valid BCAI key file".into());
    }

    println!("🔍 Key Information");
    println!("==================");
    
    let name = data["name"].as_str().unwrap_or("unknown");
    let is_private = data.get("private_key").is_some();
    let key_kind = if is_private { "Private Key" } else { "Public Key" };
    let icon = if is_private { "🔐" } else { "🔑" };

    println!("{} Type: {}", icon, key_kind);
    println!("🏷️  Name: {}", name);
    println!("🔐 Algorithm: Ed25519");
    
    if let Some(created_at) = data["created_at"].as_str() {
        println!("📅 Created: {}", created_at);
    }
    
    if let Some(public_key) = data["public_key"].as_str() {
        println!("🔑 Public Key: {}", public_key);
    }

    if is_private {
        println!();
        println!("⚠️  This is a PRIVATE key file - keep it secure!");
    } else {
        println!();
        println!("ℹ️  This is a PUBLIC key file - safe to share.");
    }

    Ok(())
} 