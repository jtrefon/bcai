use clap::{Arg, Command};
use runtime::{Blockchain, Vm};
use serde_json;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let matches = Command::new("blockchain")
        .version("0.1.0")
        .author("BCAI Team")
        .about("BCAI Blockchain Runtime - Enterprise-Grade AI Network Management")
        .subcommand(
            Command::new("init")
                .about("Initialize a new blockchain")
                .arg(
                    Arg::new("genesis-data")
                        .short('d')
                        .long("data")
                        .value_name("DATA")
                        .help("Genesis block data")
                        .default_value("BCAI Genesis Block")
                )
        )
        .subcommand(
            Command::new("add-block")
                .about("Add a new block to the blockchain")
                .arg(
                    Arg::new("data")
                        .short('d')
                        .long("data")
                        .value_name("DATA")
                        .help("Block data")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("show")
                .about("Show blockchain information")
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format: json, pretty")
                        .default_value("pretty")
                )
        )
        .subcommand(
            Command::new("validate")
                .about("Validate the blockchain")
        )
        .subcommand(
            Command::new("vm")
                .about("Run VM operations")
                .arg(
                    Arg::new("program")
                        .short('p')
                        .long("program")
                        .value_name("FILE")
                        .help("Program file to execute")
                )
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count)
                .help("Increase verbosity")
        )
        .get_matches();

    // Print header
    println!("🚀 BCAI Blockchain Runtime v0.1.0");
    println!("📊 Enterprise-Grade AI Network Management");
    println!("═══════════════════════════════════════");

    let verbosity = matches.get_count("verbose");
    if verbosity > 0 {
        println!("🔍 Verbosity level: {}", verbosity);
    }

    // Load or create blockchain
    let blockchain_file = "blockchain.json";
    let mut blockchain = if Path::new(blockchain_file).exists() {
        let data = fs::read_to_string(blockchain_file)?;
        serde_json::from_str(&data).unwrap_or_else(|_| {
            println!("⚠️  Invalid blockchain file, creating new one");
            Blockchain::new()
        })
    } else {
        Blockchain::new()
    };

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            let genesis_data = sub_matches.get_one::<String>("genesis-data").unwrap();
            blockchain = Blockchain::new();
            blockchain.add_block(genesis_data.clone());
            
            println!("✅ Initialized new blockchain with genesis block");
            println!("📦 Genesis data: {}", genesis_data);
            
            save_blockchain(&blockchain, blockchain_file)?;
        }
        
        Some(("add-block", sub_matches)) => {
            let data = sub_matches.get_one::<String>("data").unwrap();
            blockchain.add_block(data.clone());
            
            println!("✅ Added new block to blockchain");
            println!("📦 Block data: {}", data);
            println!("🔗 Total blocks: {}", blockchain.blocks.len());
            
            save_blockchain(&blockchain, blockchain_file)?;
        }
        
        Some(("show", sub_matches)) => {
            let format = sub_matches.get_one::<String>("format").unwrap();
            
            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&blockchain)?);
                }
                "pretty" | _ => {
                    println!("📊 Blockchain Information:");
                    println!("   Total blocks: {}", blockchain.blocks.len());
                    
                    for (i, block) in blockchain.blocks.iter().enumerate() {
                        println!("\n🔗 Block #{}", i);
                        println!("   Index: {}", block.index);
                        println!("   Timestamp: {}", block.timestamp);
                        println!("   Data: {}", block.data);
                        println!("   Hash: {}", &block.hash[..16]);
                        println!("   Previous: {}", &block.previous_hash[..16]);
                    }
                }
            }
        }
        
        Some(("validate", _)) => {
            println!("🔍 Validating blockchain...");
            
            if blockchain.blocks.is_empty() {
                println!("⚠️  Blockchain is empty");
            } else {
                // Basic validation logic
                let mut valid = true;
                for i in 1..blockchain.blocks.len() {
                    let current = &blockchain.blocks[i];
                    let previous = &blockchain.blocks[i - 1];
                    
                    if current.previous_hash != previous.hash {
                        println!("❌ Block {} has invalid previous hash", i);
                        valid = false;
                    }
                    
                    if current.index != previous.index + 1 {
                        println!("❌ Block {} has invalid index", i);
                        valid = false;
                    }
                }
                
                if valid {
                    println!("✅ Blockchain is valid");
                } else {
                    println!("❌ Blockchain validation failed");
                }
            }
        }
        
        Some(("vm", sub_matches)) => {
            println!("🔧 Initializing BCAI VM...");
            
            let mut vm = Vm::new();
            
            if let Some(program_file) = sub_matches.get_one::<String>("program") {
                println!("📂 Loading program: {}", program_file);
                
                if Path::new(program_file).exists() {
                    let program_data = fs::read(program_file)?;
                    println!("✅ Program loaded ({} bytes)", program_data.len());
                    println!("🚀 VM ready for execution");
                } else {
                    println!("❌ Program file not found: {}", program_file);
                }
            } else {
                println!("🚀 VM initialized and ready");
                println!("💡 Use --program <file> to load a program");
            }
        }
        
        None => {
            println!("🔧 BCAI Blockchain Commands:");
            println!("   init        - Initialize new blockchain");
            println!("   add-block   - Add block to blockchain");
            println!("   show        - Display blockchain info");
            println!("   validate    - Validate blockchain integrity");
            println!("   vm          - VM operations");
            println!("");
            println!("💡 Use --help for detailed command information");
            
            if !blockchain.blocks.is_empty() {
                println!("");
                println!("📊 Current blockchain: {} blocks", blockchain.blocks.len());
            }
        }
        
        Some((cmd, _)) => {
            println!("❌ Unknown command: {}", cmd);
            println!("💡 Use --help to see available commands");
        }
    }

    println!("✅ Blockchain runtime completed successfully");
    Ok(())
}

fn save_blockchain(blockchain: &Blockchain, filename: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(blockchain)?;
    fs::write(filename, json)?;
    println!("💾 Blockchain saved to {}", filename);
    Ok(())
} 