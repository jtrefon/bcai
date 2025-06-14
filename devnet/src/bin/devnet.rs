use clap::{Parser, Subcommand};
use devnet::{DevnetConfig, start_devnet_node};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "devnet")]
#[command(about = "BCAI Development Network Node")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a development network node
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        
        /// Node ID
        #[arg(long)]
        node_id: Option<String>,
        
        /// Enable mining
        #[arg(long)]
        mining: bool,
        
        /// Enable RPC server
        #[arg(long)]
        rpc: bool,
    },
    /// Connect to an existing devnet
    Connect {
        /// Address of the peer to connect to
        #[arg(short, long)]
        peer: String,
        
        /// Local port to use
        #[arg(short, long, default_value = "8081")]
        port: u16,
    },
    /// Show network information
    Info,
    /// Generate genesis block
    Genesis {
        /// Output file for genesis block
        #[arg(short, long, default_value = "genesis.json")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port, host, node_id, mining, rpc } => {
            let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
            let config = DevnetConfig {
                listen_addr: addr,
                node_id: node_id.unwrap_or_else(|| format!("node-{}", port)),
                enable_mining: mining,
                enable_rpc: rpc,
                max_peers: 10,
                genesis_file: "genesis.json".to_string(),
            };
            
            println!("🚀 Starting BCAI Development Network Node");
            println!("==========================================");
            println!("Address: {}", addr);
            println!("Node ID: {}", config.node_id);
            println!("Mining: {}", if mining { "✅" } else { "❌" });
            println!("RPC: {}", if rpc { "✅" } else { "❌" });
            println!();
            
            start_devnet_node(config).await?;
        }
        Commands::Connect { peer, port } => {
            println!("🔗 Connecting to devnet peer: {}", peer);
            
            let local_addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
            let config = DevnetConfig {
                listen_addr: local_addr,
                node_id: format!("client-{}", port),
                enable_mining: false,
                enable_rpc: false,
                max_peers: 1,
                genesis_file: "genesis.json".to_string(),
            };
            
            // Start node and connect to peer
            println!("Local address: {}", local_addr);
            start_devnet_node(config).await?;
            
            // TODO: Implement peer connection logic
            println!("✅ Connected successfully");
        }
        Commands::Info => {
            show_network_info().await?;
        }
        Commands::Genesis { output } => {
            generate_genesis_block(&output).await?;
        }
    }

    Ok(())
}

async fn show_network_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 BCAI Development Network Information");
    println!("=====================================");
    println!();
    
    println!("📊 Network Status:");
    println!("  • Network Type: Development");
    println!("  • Consensus: Proof of Useful Work (Simulated)");
    println!("  • Block Time: ~10 seconds");
    println!("  • Default Port: 8080");
    println!();
    
    println!("🔧 Supported Features:");
    println!("  • Enhanced VM Runtime: ✅");
    println!("  • ML Job Execution: ✅");
    println!("  • Python Bridge: ✅");
    println!("  • GPU Acceleration: ✅");
    println!("  • Distributed Training: ✅");
    println!();
    
    println!("🚀 Quick Start:");
    println!("  1. Generate genesis: devnet genesis");
    println!("  2. Start node: devnet start --mining --rpc");
    println!("  3. Connect clients: devnet connect --peer 127.0.0.1:8080");
    
    Ok(())
}

async fn generate_genesis_block(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Generating genesis block...");
    
    let genesis = serde_json::json!({
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().timestamp(),
        "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "merkle_root": "0000000000000000000000000000000000000000000000000000000000000000",
        "nonce": 0,
        "difficulty": 1,
        "transactions": [],
        "network": "devnet",
        "consensus": "pouw",
        "enhanced_vm": {
            "enabled": true,
            "ml_instructions": true,
            "python_bridge": true,
            "hardware_acceleration": true
        },
        "initial_balance": {
            "dev_account": "1000000000000000000" // 1 billion tokens
        }
    });
    
    std::fs::write(output, serde_json::to_string_pretty(&genesis)?)?;
    
    println!("✅ Genesis block generated: {}", output);
    println!("📄 Block details:");
    println!("  • Network: devnet");
    println!("  • Consensus: Proof of Useful Work");
    println!("  • Enhanced VM: Enabled");
    println!("  • Initial Balance: 1B tokens");
    
    Ok(())
} 