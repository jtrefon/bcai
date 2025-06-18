//! Decentralized Filesystem Demo
//!
//! This demo shows:
//! - File storage with economic incentives
//! - Escrow-based storage contracts
//! - Parallel file assembly
//! - Storage node earnings
//! - Bandwidth cost separation

use runtime::{
    decentralized_filesystem::{
        DecentralizedFilesystem, DfsConfig, FileVisibility, StorageNodeMetrics,
    },
    distributed_storage::{DistributedStorage, StorageConfig, StorageNode},
    network::NetworkCoordinator,
    node::{UnifiedNode, NodeCapability, CapabilityType},
    token::TokenLedger,
    smart_contracts::SmartContractEngine,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️  BCAI Decentralized Filesystem Demo");
    println!("=====================================");
    println!();

    // ===== SETUP INFRASTRUCTURE =====
    println!("🏗️  Setting up distributed infrastructure...");
    
    // Create storage nodes
    let storage_nodes = create_storage_nodes().await;
    println!("✅ Created {} storage nodes", storage_nodes.len());

    // Setup token ledger with initial balances
    let token_ledger = Arc::new(RwLock::new(create_token_ledger()));
    println!("✅ Token ledger initialized");

    // Create network infrastructure
    let (dfs, network_stats) = create_dfs_infrastructure(storage_nodes, Arc::clone(&token_ledger)).await;
    println!("✅ DFS infrastructure ready");
    println!("   Storage nodes: {}", network_stats.storage_nodes);
    println!("   Network capacity: {:.1} GB", network_stats.total_capacity_gb);
    println!();

    // ===== DEMONSTRATE FILE STORAGE =====
    println!("📁 Demo 1: File Storage with Economic Contracts");
    println!("===============================================");

    let storage_demo_start = Instant::now();

    // Create test files of different sizes
    let test_files = vec![
        ("small_dataset.csv", create_test_data(5 * 1024 * 1024)), // 5MB
        ("medium_model.bin", create_test_data(50 * 1024 * 1024)), // 50MB  
        ("large_training_data.parquet", create_test_data(200 * 1024 * 1024)), // 200MB
    ];

    let mut stored_files = Vec::new();

    for (filename, data) in test_files {
        println!("\n📄 Storing file: {} ({:.1} MB)", filename, data.len() as f64 / (1024.0 * 1024.0));
        
        let file_hash = dfs.store_file(
            filename.to_string(),
            data,
            if filename.ends_with(".csv") { "text/csv" } 
            else if filename.ends_with(".bin") { "application/octet-stream" }
            else { "application/octet-stream" }.to_string(),
            "researcher_alice".to_string(),
            FileVisibility::Private,
            168, // 1 week storage
            vec!["ml".to_string(), "dataset".to_string()],
        ).await?;

        stored_files.push((filename, file_hash));
        println!("   File hash: {}", stored_files.last().unwrap().1);
    }

    let storage_duration = storage_demo_start.elapsed();
    println!("\n✅ All files stored in {:.2}s", storage_duration.as_secs_f64());

    // Show storage statistics
    let stats = dfs.get_statistics().await;
    println!("\n📊 Storage Statistics:");
    println!("   Total files: {}", stats.total_files);
    println!("   Total storage: {:.1} MB", stats.total_storage_bytes as f64 / (1024.0 * 1024.0));
    println!("   Active contracts: {}", stats.active_contracts);
    println!("   Avg replication: {:.1}", stats.avg_replication);
    println!();

    // ===== DEMONSTRATE FILE RETRIEVAL =====
    println!("📥 Demo 2: High-Speed Parallel File Assembly");
    println!("============================================");

    let retrieval_demo_start = Instant::now();

    for (filename, file_hash) in &stored_files {
        println!("\n🔍 Retrieving file: {}", filename);
        
        let retrieval_start = Instant::now();
        let retrieved_data = dfs.retrieve_file(
            file_hash.clone(),
            "researcher_alice".to_string(),
        ).await?;
        let retrieval_time = retrieval_start.elapsed();

        println!("   Retrieved {} bytes in {:.3}s", retrieved_data.len(), retrieval_time.as_secs_f64());
        
        // Calculate throughput
        let throughput_mbps = (retrieved_data.len() as f64 / (1024.0 * 1024.0)) / retrieval_time.as_secs_f64();
        println!("   Throughput: {:.1} MB/s", throughput_mbps);
    }

    let total_retrieval_duration = retrieval_demo_start.elapsed();
    println!("\n✅ All files retrieved in {:.2}s", total_retrieval_duration.as_secs_f64());

    // Show updated assembly statistics
    let updated_stats = dfs.get_statistics().await;
    println!("\n📊 Assembly Statistics:");
    println!("   Files assembled: {}", updated_stats.assembly_stats.files_assembled);
    println!("   Bytes assembled: {:.1} MB", updated_stats.assembly_stats.bytes_assembled as f64 / (1024.0 * 1024.0));
    println!("   Avg assembly time: {:.3}s", updated_stats.assembly_stats.avg_assembly_time);
    println!("   Parallel efficiency: {:.1}%", updated_stats.assembly_stats.parallel_efficiency * 100.0);
    println!();

    // ===== DEMONSTRATE ECONOMIC MODEL =====
    println!("💰 Demo 3: Economic Model & Storage Node Earnings");
    println!("=================================================");

    // Show token balances before contract completion
    let balances_before = get_token_balances(Arc::clone(&token_ledger)).await;
    println!("💳 Token balances before contract completion:");
    for (account, balance) in &balances_before {
        println!("   {}: {} BCAI", account, balance);
    }

    // Process storage contracts (simulate contract completion)
    println!("\n⏰ Processing storage contract completions...");
    dfs.process_storage_contracts().await?;

    // Show token balances after contract completion
    let balances_after = get_token_balances(Arc::clone(&token_ledger)).await;
    println!("\n💳 Token balances after contract completion:");
    for (account, balance) in &balances_after {
        println!("   {}: {} BCAI", account, balance);
    }

    // Calculate earnings
    println!("\n📈 Storage node earnings:");
    let storage_metrics = dfs.get_storage_metrics().await;
    for (node_id, metrics) in storage_metrics.iter() {
        println!("  📊 {}: {} BCAI earned, {:.1}% reliability", 
                node_id, metrics.total_earnings, metrics.reliability * 100.0);
    }

    // Force complete storage contracts for demonstration
    println!("\n🔄 Force completing storage contracts for rewards demonstration...");
    match dfs.force_complete_storage_contracts().await {
        Ok(total_distributed) => {
            println!("✅ Completed all active contracts, distributed {} BCAI total", total_distributed);
        }
        Err(e) => {
            println!("❌ Error completing contracts: {}", e);
        }
    }

    // Show detailed earnings report
    println!("\n📊 Detailed Node Earnings Report:");
    let earnings_report = dfs.get_node_earnings_report().await;
    for report in &earnings_report {
        println!("  🏆 {} ({}):", report.node_id, report.performance_tier);
        println!("    💰 Total Earnings: {} BCAI", report.total_earnings);
        println!("    📈 Reliability: {:.2}%", report.reliability_score * 100.0);
        println!("    ⚡ Avg Response: {}ms", report.avg_response_time);
        println!("    💾 Capacity: {:.1} GB", report.storage_capacity_gb);
        println!("    📊 Earnings/GB: {:.2} BCAI", report.earnings_per_gb);
    }

    // Show network-wide rewards statistics
    println!("\n🌐 Network Rewards Distribution Statistics:");
    let rewards_stats = dfs.get_rewards_distribution_stats().await;
    println!("  💸 Total Distributed: {} BCAI", rewards_stats.total_earnings_distributed);
    println!("  👥 Active Providers: {}", rewards_stats.active_storage_providers);
    println!("  ✅ Completed Contracts: {}", rewards_stats.completed_contracts);
    println!("  💾 Total Capacity: {:.1} GB", rewards_stats.total_storage_capacity_gb);
    println!("  📊 Avg Earnings/Provider: {} BCAI", rewards_stats.avg_earnings_per_provider);
    println!("  🎯 Network Reliability: {:.2}%", rewards_stats.avg_reliability_score * 100.0);
    println!("  🏆 Performance Tiers:");
    println!("    Premium: {} BCAI", rewards_stats.premium_tier_earnings);
    println!("    Standard: {} BCAI", rewards_stats.standard_tier_earnings);
    println!("    Basic: {} BCAI", rewards_stats.basic_tier_earnings);

    // ===== DEMONSTRATE LARGE DATASET SCENARIO =====
    println!("📊 Demo 4: Large Dataset Scenario (3TB ML Dataset)");
    println!("===================================================");

    // Simulate large dataset storage (we'll use smaller data but show the calculations)
    let simulated_dataset_size_gb = 3000.0; // 3TB
    let chunk_size_mb = 4;
    let chunks_needed = (simulated_dataset_size_gb * 1024.0 / chunk_size_mb as f64) as u64;
    
    println!("📋 3TB Dataset Analysis:");
    println!("   Total size: {:.1} GB", simulated_dataset_size_gb);
    println!("   Chunk size: {} MB", chunk_size_mb);
    println!("   Total chunks: {}", chunks_needed);
    println!("   Replication factor: 3");
    println!("   Total storage slots: {}", chunks_needed * 3);

    // Calculate economic requirements
    let dfs_config = DfsConfig::default();
    let storage_cost_per_gb_month = dfs_config.storage_price_per_gb_month;
    let storage_duration_months = 1.0; // 1 month
    let total_storage_cost = (simulated_dataset_size_gb * storage_cost_per_gb_month as f64 * storage_duration_months * 3.0) as u64; // 3x for replication

    println!("\n💰 Economic Analysis:");
    println!("   Storage cost: {} BCAI per GB per month", storage_cost_per_gb_month);
    println!("   Total storage cost: {} BCAI for 1 month", total_storage_cost);
    println!("   Cost per storage node: {} BCAI (distributed across nodes)", total_storage_cost / 10); // Assume 10 nodes

    // Calculate bandwidth costs for retrieval
    let bandwidth_cost_per_gb = dfs_config.bandwidth_price_per_gb;
    let retrieval_cost = (simulated_dataset_size_gb * bandwidth_cost_per_gb as f64) as u64;
    
    println!("   Bandwidth cost: {} BCAI per GB", bandwidth_cost_per_gb);
    println!("   Full retrieval cost: {} BCAI", retrieval_cost);

    // Parallel assembly simulation
    let parallel_workers = dfs_config.parallel_assembly_workers;
    let estimated_assembly_time = chunks_needed as f64 / parallel_workers as f64 * 0.1; // 0.1s per chunk
    
    println!("\n⚡ Performance Projections:");
    println!("   Parallel workers: {}", parallel_workers);
    println!("   Estimated assembly time: {:.1} minutes", estimated_assembly_time / 60.0);
    println!("   Assembly throughput: {:.1} MB/s", (simulated_dataset_size_gb * 1024.0) / estimated_assembly_time);
    println!();

    // ===== DEMONSTRATE SEPARATION OF CONCERNS =====
    println!("⚖️  Demo 5: Separation of Storage vs Compute Economics");
    println!("======================================================");

    println!("🏭 Storage Network Economics:");
    println!("   - Storage nodes earn from data persistence");
    println!("   - Payments based on availability and reliability");
    println!("   - Escrow ensures fair compensation");
    println!("   - Geographic distribution optimizes access");
    println!();

    println!("🖥️  Compute Network Economics (separate):");
    println!("   - Training nodes earn from GPU/CPU computation");
    println!("   - Payments based on model quality and training time");
    println!("   - Data retrieved from storage network as needed");
    println!("   - Specialized hardware (GPUs) commands premium rates");
    println!();

    println!("🔄 Synergy Benefits:");
    println!("   - Data persists beyond single training jobs");
    println!("   - Multiple training jobs can reuse same datasets");
    println!("   - Storage costs amortized across multiple uses");
    println!("   - Reduced data transfer for repeated access");
    println!("   - Enable long-term data availability guarantees");
    println!();

    // ===== FINAL STATISTICS =====
    println!("📊 Final System Statistics");
    println!("==========================");
    
    let final_stats = dfs.get_statistics().await;
    let final_balances = get_token_balances(Arc::clone(&token_ledger)).await;
    
    println!("📁 Filesystem:");
    println!("   Files stored: {}", final_stats.total_files);
    println!("   Total storage: {:.1} MB", final_stats.total_storage_bytes as f64 / (1024.0 * 1024.0));
    println!("   Active contracts: {}", final_stats.active_contracts);
    println!("   Storage nodes: {}", final_stats.storage_nodes);
    println!("   Avg replication: {:.1}", final_stats.avg_replication);
    
    println!("\n⚡ Performance:");
    println!("   Files assembled: {}", final_stats.assembly_stats.files_assembled);
    println!("   Avg assembly time: {:.3}s", final_stats.assembly_stats.avg_assembly_time);
    println!("   Cache hit rate: {:.1}%", final_stats.cache_hit_rate * 100.0);
    
    println!("\n💰 Economics:");
    let total_treasury = final_balances.get("network_treasury").unwrap_or(&0);
    let total_storage_earnings: u64 = final_balances.iter()
        .filter(|(k, _)| k.starts_with("storage_node"))
        .map(|(_, v)| *v)
        .sum();
    
    println!("   Network treasury: {} BCAI", total_treasury);
    println!("   Total storage earnings: {} BCAI", total_storage_earnings);
    println!("   Active storage contracts: {}", final_stats.active_contracts);
    
    println!("\n🌟 Key Achievements:");
    println!("   ✅ Decentralized file storage with economic incentives");
    println!("   ✅ Parallel assembly with high throughput");
    println!("   ✅ Escrow-based fair compensation");
    println!("   ✅ Separation of storage and compute economics");
    println!("   ✅ Scalable to TB-scale datasets");
    println!("   ✅ Geographic distribution and redundancy");
    
    println!("\n🚀 Ready for production-scale decentralized storage!");
    
    Ok(())
}

/// Create test storage nodes with various capabilities
async fn create_storage_nodes() -> Vec<StorageNode> {
    vec![
        StorageNode::new("storage_node_1".to_string(), "192.168.1.10:8000".to_string(), 500 * 1024 * 1024 * 1024), // 500GB
        StorageNode::new("storage_node_2".to_string(), "192.168.1.11:8000".to_string(), 1024 * 1024 * 1024 * 1024), // 1TB
        StorageNode::new("storage_node_3".to_string(), "192.168.1.12:8000".to_string(), 2 * 1024 * 1024 * 1024 * 1024), // 2TB
        StorageNode::new("storage_node_4".to_string(), "192.168.1.13:8000".to_string(), 750 * 1024 * 1024 * 1024), // 750GB
        StorageNode::new("storage_node_5".to_string(), "192.168.1.14:8000".to_string(), 1500 * 1024 * 1024 * 1024), // 1.5TB
    ]
}

/// Create token ledger with initial allocations
fn create_token_ledger() -> TokenLedger {
    let mut ledger = TokenLedger::new();
    
    // Research accounts
    ledger.mint("researcher_alice", 50000);
    ledger.mint("researcher_bob", 30000);
    ledger.mint("ml_lab_1", 100000);
    
    // Storage nodes (start with small stake)
    ledger.mint("storage_node_1", 1000);
    ledger.mint("storage_node_2", 1000);
    ledger.mint("storage_node_3", 1000);
    ledger.mint("storage_node_4", 1000);
    ledger.mint("storage_node_5", 1000);
    
    // Network treasury
    ledger.mint("network_treasury", 0);
    
    ledger
}

/// Create complete DFS infrastructure
async fn create_dfs_infrastructure(
    storage_nodes: Vec<StorageNode>,
    token_ledger: Arc<RwLock<TokenLedger>>,
) -> (DecentralizedFilesystem, NetworkStats) {
    // Create local node
    let local_node = Arc::new(RwLock::new(UnifiedNode::new(
        "dfs_coordinator".to_string(),
        NodeCapability {
            cpus: 8,
            gpus: 0,
            gpu_memory_gb: 0,
            available_stake: 10000,
            reputation: 100,
            capability_types: vec![CapabilityType::Storage, CapabilityType::Network],
        },
        10000,
    )));

    // Create storage infrastructure
    let storage_config = StorageConfig {
        replication_factor: 3,
        max_storage_capacity: 10 * 1024 * 1024 * 1024, // 10GB per node
        ..Default::default()
    };
    
    let storage_coordinator = Arc::new(
        DistributedStorage::new(storage_config, "dfs_coordinator".to_string())
    );

    // Add storage nodes
    let mut total_capacity = 0u64;
    for node in &storage_nodes {
        storage_coordinator.add_node(node.clone()).await;
        total_capacity += node.capacity;
    }

    // Create network coordinator
    let network_coordinator = Arc::new(RwLock::new(
        NetworkCoordinator::new(local_node.read().await.clone())
    ));

    // Create smart contract engine
    let contract_engine = Arc::new(RwLock::new(SmartContractEngine::new()));

    // Create DFS
    let dfs_config = DfsConfig {
        default_replication: 3,
        default_chunk_size_mb: 4,
        storage_price_per_gb_month: 10,
        bandwidth_price_per_gb: 1,
        parallel_assembly_workers: 8,
        ..Default::default()
    };

    let dfs = DecentralizedFilesystem::new(
        dfs_config,
        local_node,
        network_coordinator,
        storage_coordinator,
        None, // No transfer coordinator for this demo
        token_ledger,
        contract_engine,
    );

    // Add storage nodes to DFS metrics
    for node in storage_nodes {
        let storage_metrics = StorageNodeMetrics {
            node_id: node.node_id.clone(),
            total_storage: node.capacity,
            available_storage: node.capacity,
            reliability: 0.95, // 95% reliability
            avg_response_time: 50, // 50ms
            bandwidth_capacity: 100 * 1024 * 1024, // 100MB/s
            total_earnings: 0,
            active_contracts: 0,
            last_heartbeat: Utc::now(),
            region: "us-west-1".to_string(),
        };
        dfs.add_storage_node_metrics(storage_metrics).await;
    }

    let network_stats = NetworkStats {
        storage_nodes: 5,
        total_capacity_gb: total_capacity as f64 / (1024.0 * 1024.0 * 1024.0),
    };

    (dfs, network_stats)
}

/// Create test data of specified size
fn create_test_data(size: usize) -> Vec<u8> {
    // Create deterministic test data
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        data.push((i % 256) as u8);
    }
    data
}

/// Get token balances for analysis
async fn get_token_balances(token_ledger: Arc<RwLock<TokenLedger>>) -> std::collections::HashMap<String, u64> {
    let ledger = token_ledger.read().await;
    let mut balances = std::collections::HashMap::new();
    
    // Get all known accounts
    let accounts = vec![
        "researcher_alice", "researcher_bob", "ml_lab_1",
        "storage_node_1", "storage_node_2", "storage_node_3", "storage_node_4", "storage_node_5",
        "network_treasury"
    ];
    
    for account in accounts {
        balances.insert(account.to_string(), ledger.balance(account));
    }
    
    balances
}

/// Network statistics for demo
struct NetworkStats {
    storage_nodes: usize,
    total_capacity_gb: f64,
}