use runtime::{
    ConsensusNode, ConsensusConfig, Transaction, 
    Block, BlockchainStats, MiningStats
};
use clap::{Args, Parser, Subcommand};
use serde_json;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "blockchain")]
#[command(about = "BCAI Blockchain CLI - Full consensus and mining functionality")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a blockchain node with mining
    Node(NodeArgs),
    /// Mine blocks manually
    Mine(MineArgs),
    /// Submit transactions
    Transaction(TransactionArgs),
    /// Query blockchain state
    Query(QueryArgs),
    /// Run blockchain demo
    Demo(DemoArgs),
    /// Show blockchain statistics
    Stats,
}

#[derive(Args)]
struct NodeArgs {
    /// Node identifier
    #[arg(long, default_value = "miner1")]
    node_id: String,
    /// Enable mining
    #[arg(long, default_value = "true")]
    mining: bool,
    /// Target block time in seconds
    #[arg(long, default_value = "10")]
    block_time: u64,
    /// Maximum transactions per block
    #[arg(long, default_value = "100")]
    max_tx: usize,
    /// Run for specified duration in seconds
    #[arg(long, default_value = "60")]
    duration: u64,
}

#[derive(Args)]
struct MineArgs {
    /// Number of blocks to mine
    #[arg(long, default_value = "5")]
    blocks: u64,
    /// Node identifier
    #[arg(long, default_value = "miner1")]
    node_id: String,
}

#[derive(Args)]
struct TransactionArgs {
    /// Transaction type
    #[command(subcommand)]
    tx_type: TransactionType,
}

#[derive(Subcommand)]
enum TransactionType {
    /// Transfer tokens
    Transfer {
        /// From account
        #[arg(long)]
        from: String,
        /// To account
        #[arg(long)]
        to: String,
        /// Amount to transfer
        #[arg(long)]
        amount: u64,
    },
    /// Stake tokens
    Stake {
        /// Account to stake from
        #[arg(long)]
        account: String,
        /// Amount to stake
        #[arg(long)]
        amount: u64,
    },
    /// Submit AI training result
    AiResult {
        /// Worker node ID
        #[arg(long)]
        worker: String,
        /// Job ID
        #[arg(long)]
        job_id: u64,
        /// Training accuracy
        #[arg(long)]
        accuracy: f32,
    },
}

#[derive(Args)]
struct QueryArgs {
    /// Query type
    #[command(subcommand)]
    query_type: QueryType,
}

#[derive(Subcommand)]
enum QueryType {
    /// Get account balance
    Balance {
        /// Account name
        #[arg(long)]
        account: String,
    },
    /// Get block by height/hash
    Block {
        /// Block identifier (height or hash)
        #[arg(long)]
        block: String,
    },
    /// Get transaction details
    Transaction {
        /// Transaction hash
        #[arg(long)]
        hash: String,
    },
    /// List recent blocks
    Recent {
        /// Number of blocks to show
        #[arg(long, default_value = "10")]
        count: usize,
    },
}

#[derive(Args)]
struct DemoArgs {
    /// Demo scenario
    #[command(subcommand)]
    scenario: DemoScenario,
}

#[derive(Subcommand)]
enum DemoScenario {
    /// Mining demonstration
    Mining {
        /// Duration in seconds
        #[arg(long, default_value = "30")]
        duration: u64,
    },
    /// Transaction flow demo
    Transactions {
        /// Number of transactions
        #[arg(long, default_value = "10")]
        count: u64,
    },
    /// Full consensus demo
    Consensus {
        /// Number of nodes
        #[arg(long, default_value = "3")]
        nodes: u64,
        /// Duration in seconds
        #[arg(long, default_value = "60")]
        duration: u64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Node(args) => run_node(args).await?,
        Commands::Mine(args) => mine_blocks(args).await?,
        Commands::Transaction(args) => submit_transaction(args).await?,
        Commands::Query(args) => query_blockchain(args).await?,
        Commands::Demo(args) => run_demo(args).await?,
        Commands::Stats => show_stats().await?,
    }

    Ok(())
}

async fn run_node(args: NodeArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting BCAI Blockchain Node");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Configuration:");
    println!("   Node ID: {}", args.node_id);
    println!("   Mining: {}", if args.mining { "✅ Enabled" } else { "❌ Disabled" });
    println!("   Block Time Target: {}s", args.block_time);
    println!("   Max TX per Block: {}", args.max_tx);
    println!("   Run Duration: {}s", args.duration);
    println!();

    let config = ConsensusConfig {
        node_id: args.node_id.clone(),
        mining_enabled: args.mining,
        max_peers: 50,
        block_time_target: args.block_time,
        max_transactions_per_block: args.max_tx,
        staking_enabled: true,
        minimum_stake: 1000,
    };

    let mut node = ConsensusNode::new(config)?;
    node.start().await?;

    // Add some initial transactions for mining
    let _ = node.create_transfer("alice", 1000);
    let _ = node.create_transfer("bob", 2000);
    let _ = node.create_stake(500);

    println!("⏱️  Node running for {} seconds...", args.duration);
    
    // Run for specified duration
    for i in 0..args.duration {
        sleep(Duration::from_secs(1)).await;
        
        if i % 10 == 0 && i > 0 {
            let stats = node.get_blockchain_stats();
            let mining_stats = node.get_mining_stats();
            
            println!("📊 Status Update ({}s):", i);
            println!("   ⛓️  Height: {}", stats.block_height);
            println!("   📦 Total Blocks: {}", stats.active_validators);
            println!("   📤 Pending TX: {}", stats.pending_transactions);
            println!("   ⛏️  Blocks Mined: {}", mining_stats.blocks_mined);
            println!("   ⚡ Hash Rate: {:.1} H/s", mining_stats.hash_rate);
            println!();
        }
    }

    // Final statistics
    let final_stats = node.get_blockchain_stats();
    let final_mining = node.get_mining_stats();
    
    println!("🏁 Final Results:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⛓️  Blockchain Height: {}", final_stats.block_height);
    println!("📦 Total Blocks: {}", final_stats.active_validators);
    println!("⛏️  Blocks Mined: {}", final_mining.blocks_mined);
    println!("⚡ Final Hash Rate: {:.1} H/s", final_mining.hash_rate);
    println!("👥 Accounts: {}", final_stats.active_validators);
    println!("🎯 Current Difficulty: 0x{:08x}", final_stats.current_difficulty);

    node.stop()?;
    Ok(())
}

async fn mine_blocks(args: MineArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("⛏️  Mining {} blocks with node '{}'", args.blocks, args.node_id);
    
    let config = ConsensusConfig {
        node_id: args.node_id,
        mining_enabled: true,
        block_time_target: 5, // Fast mining for demo
        ..Default::default()
    };

    let mut node = ConsensusNode::new(config)?;
    node.start().await?;

    let mut blocks_found = 0;
    let start_height = node.get_blockchain_stats().block_height;

    while blocks_found < args.blocks {
        sleep(Duration::from_millis(100)).await;
        
        let current_height = node.get_blockchain_stats().block_height;
        let new_blocks = current_height - start_height;
        
        if new_blocks > blocks_found {
            blocks_found = new_blocks;
            let mining_stats = node.get_mining_stats();
            println!("⛏️  Block #{} mined! (Hash rate: {:.1} H/s)", 
                     current_height, mining_stats.hash_rate);
        }
    }

    println!("✅ Successfully mined {} blocks!", args.blocks);
    node.stop()?;
    Ok(())
}

async fn submit_transaction(args: TransactionArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config = ConsensusConfig::default();
    let _node = ConsensusNode::new(config)?;

    let tx_hash = match args.tx_type {
        TransactionType::Transfer { from, to, amount } => {
            println!("💸 Creating transfer: {} → {} ({} tokens)", from, to, amount);
            
            // For demo purposes, create a node with the 'from' account
            let config = ConsensusConfig { node_id: from, ..Default::default() };
            let node = ConsensusNode::new(config)?;
            node.create_transfer(&to, amount)?
        },
        
        TransactionType::Stake { account, amount } => {
            println!("🥩 Creating stake: {} staking {} tokens", account, amount);
            
            let config = ConsensusConfig { node_id: account, ..Default::default() };
            let node = ConsensusNode::new(config)?;
            node.create_stake(amount)?
        },
        
        TransactionType::AiResult { worker, job_id, accuracy } => {
            println!("🤖 Submitting AI training result: worker={}, job={}, accuracy={:.2}%", 
                     worker, job_id, accuracy * 100.0);
            
            let config = ConsensusConfig { node_id: worker, ..Default::default() };
            let node = ConsensusNode::new(config)?;
            node.train_and_submit(job_id, 100).await?
        },
    };

    println!("✅ Transaction submitted successfully!");
    println!("📋 Transaction Hash: {}", tx_hash);
    
    Ok(())
}

async fn query_blockchain(args: QueryArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config = ConsensusConfig::default();
    let node = ConsensusNode::new(config)?;

    match args.query_type {
        QueryType::Balance { account } => {
            let balance = node.get_balance(&account);
            println!("💰 Account '{}' balance: {} tokens", account, balance);
        },
        
        QueryType::Block { block } => {
            println!("📦 Block information for: {}", block);
            // In a real implementation, we'd parse height vs hash
            println!("(Block lookup by identifier not fully implemented in demo)");
        },
        
        QueryType::Transaction { hash } => {
            println!("📋 Transaction details for: {}", hash);
            println!("(Transaction lookup not fully implemented in demo)");
        },
        
        QueryType::Recent { count } => {
            let blocks = node.get_recent_blocks(count);
            println!("📚 {} most recent blocks:", blocks.len());
            for (i, block) in blocks.iter().enumerate() {
                println!("  {}. Block #{} - {} transactions - Validator: {}", 
                         i + 1, block.height, block.transactions.len(), block.validator);
            }
        },
    }

    Ok(())
}

async fn run_demo(args: DemoArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.scenario {
        DemoScenario::Mining { duration } => {
            println!("⛏️  BCAI Mining Demo ({} seconds)", duration);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let node_args = NodeArgs {
                node_id: "demo_miner".to_string(),
                mining: true,
                block_time: 5, // Fast blocks for demo
                max_tx: 50,
                duration,
            };
            
            run_node(node_args).await?;
        },
        
        DemoScenario::Transactions { count } => {
            println!("📤 BCAI Transaction Demo ({} transactions)", count);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let config = ConsensusConfig {
                node_id: "tx_demo".to_string(),
                mining_enabled: true,
                block_time_target: 3,
                ..Default::default()
            };
            
            let mut node = ConsensusNode::new(config)?;
            node.start().await?;
            
            // Submit various transaction types
            for i in 0..count {
                match i % 3 {
                    0 => {
                        let _ = node.create_transfer(&format!("user_{}", i), 100 + i * 10);
                        println!("📤 Transfer transaction {} submitted", i + 1);
                    },
                    1 => {
                        let _ = node.create_stake(50 + i * 5);
                        println!("🥩 Stake transaction {} submitted", i + 1);
                    },
                    2 => {
                        let _ = node.train_and_submit(i, 50).await;
                        println!("🤖 AI training result {} submitted", i + 1);
                    },
                    _ => unreachable!(),
                }
                
                sleep(Duration::from_millis(500)).await;
            }
            
            println!("⏱️  Waiting for transactions to be mined...");
            sleep(Duration::from_secs(15)).await;
            
            let final_stats = node.get_blockchain_stats();
            println!("✅ Demo completed! Final height: {}", final_stats.block_height);
            
            node.stop()?;
        },
        
        DemoScenario::Consensus { nodes: _nodes, duration } => {
            println!("🤝 BCAI Consensus Demo");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Note: Multi-node consensus demo simplified for CLI");
            
            let node_args = NodeArgs {
                node_id: "consensus_demo".to_string(),
                mining: true,
                block_time: 8,
                max_tx: 200,
                duration,
            };
            
            run_node(node_args).await?;
        },
    }

    Ok(())
}

async fn show_stats() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 BCAI Blockchain Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let config = ConsensusConfig::default();
    let node = ConsensusNode::new(config)?;
    
    let blockchain_stats = node.get_blockchain_stats();
    let mining_stats = node.get_mining_stats();
    
    println!("⛓️  Blockchain State:");
    println!("   Height: {}", blockchain_stats.block_height);
    println!("   Total Blocks: {}", blockchain_stats.active_validators);
    println!("   Pending Transactions: {}", blockchain_stats.pending_transactions);
    println!("   Total Accounts: {}", blockchain_stats.active_validators);
    println!("   Current Difficulty: 0x{:08x}", blockchain_stats.current_difficulty);
    println!();
    
    println!("⛏️  Mining State:");
    println!("   Blocks Mined: {}", mining_stats.blocks_mined);
    println!("   Is Mining: {}", if mining_stats.is_mining { "✅ Yes" } else { "❌ No" });
    println!("   Hash Rate: {:.1} H/s", mining_stats.hash_rate);
    println!("   Last Block Time: {}", mining_stats.last_block_time);
    println!();
    
    println!("🌐 Network State:");
    println!("   Connected Peers: {}", node.get_peer_count());
    println!();
    
    Ok(())
} 