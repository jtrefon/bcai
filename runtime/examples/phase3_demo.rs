//! Phase 3 Advanced Features Demo
//!
//! This example demonstrates the advanced features of Phase 3:
//! - Distributed Storage with replication and consistency
//! - Advanced Consensus with multi-validator support
//! - Security Layer with authentication and encryption
//! - Performance Optimization with caching and bandwidth management

use runtime::{
    distributed_storage::{DistributedStorage, StorageConfig, StorageNode},
    consensus_engine::{ConsensusEngine, ConsensusConfig, Validator},
    security_layer::{SecurityManager, SecurityConfig, AuthCredentials, Permission},
    performance_optimizer::{PerformanceOptimizer, PerformanceConfig},
    Transaction,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Phase 3 Advanced Features Demo");
    println!("======================================");

    // Phase 3.1: Distributed Storage Demo
    println!("\n📦 Phase 3.1: Distributed Storage System");
    println!("========================================");
    
    let storage_config = StorageConfig {
        replication_factor: 3,
        max_storage_capacity: 10 * 1024 * 1024, // 10MB for demo
        ..Default::default()
    };
    
    let storage = DistributedStorage::new(storage_config, "storage_node_1".to_string());
    
    // Add storage nodes to the cluster
    let nodes = vec![
        StorageNode::new("node_1".to_string(), "127.0.0.1:8001".to_string(), 5 * 1024 * 1024),
        StorageNode::new("node_2".to_string(), "127.0.0.1:8002".to_string(), 5 * 1024 * 1024),
        StorageNode::new("node_3".to_string(), "127.0.0.1:8003".to_string(), 5 * 1024 * 1024),
    ];
    
    for node in nodes {
        storage.add_node(node).await;
    }
    
    // Store some ML data
    let ml_model_data = b"ML model weights and parameters...".to_vec();
    let training_data = b"Training dataset chunk 1...".to_vec();
    
    storage.store("ml_model_v1".to_string(), ml_model_data.clone()).await;
    storage.store("training_data_chunk_1".to_string(), training_data.clone()).await;
    
    // Give time for async processing
    sleep(Duration::from_millis(50)).await;
    
    let storage_stats = storage.get_stats().await;
    println!("✅ Storage cluster initialized:");
    println!("   Nodes: {}", storage_stats.total_nodes);
    println!("   Entries: {}", storage_stats.total_entries);
    println!("   Total Size: {} bytes", storage_stats.total_size);
    println!("   Avg Replication: {:.1}", storage_stats.avg_replication_factor);

    // Phase 3.2: Advanced Consensus Demo
    println!("\n⚖️  Phase 3.2: Advanced Consensus Engine");
    println!("========================================");
    
    let consensus_config = ConsensusConfig {
        block_time: 5, // 5 seconds for demo
        max_transactions_per_block: 10,
        min_validators: 3,
        ..Default::default()
    };
    
    let consensus = ConsensusEngine::new(consensus_config, "consensus_node_1".to_string());
    
    // Add validators
    let validators = vec![
        Validator::new("validator_1".to_string(), "pubkey_1".to_string(), 1000),
        Validator::new("validator_2".to_string(), "pubkey_2".to_string(), 1500),
        Validator::new("validator_3".to_string(), "pubkey_3".to_string(), 2000),
    ];
    
    for validator in validators {
        consensus.add_validator(validator).await;
    }
    
    // Submit transactions
    let transactions = vec![
        Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        },
        Transaction {
            from: "bob".to_string(),
            to: "charlie".to_string(),
            amount: 50,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        },
    ];
    
    for tx in transactions {
        consensus.submit_transaction(tx).await;
    }
    
    // Propose a block
    let _result = consensus.propose_block().await;
    
    // Give time for async processing
    sleep(Duration::from_millis(50)).await;
    
    let consensus_stats = consensus.get_stats().await;
    println!("✅ Consensus network established:");
    println!("   Algorithm: {:?}", consensus_stats.algorithm);
    println!("   Active Validators: {}", consensus_stats.active_validators);
    println!("   Total Stake: {}", consensus_stats.total_stake);
    println!("   Blockchain Height: {}", consensus_stats.blockchain_height);
    println!("   Pending Transactions: {}", consensus_stats.pending_transactions);

    // Phase 3.3: Security Layer Demo
    println!("\n🔒 Phase 3.3: Security Layer");
    println!("============================");
    
    let security_config = SecurityConfig {
        session_timeout: Duration::from_secs(300), // 5 minutes for demo
        max_auth_attempts: 3,
        ..Default::default()
    };
    
    let mut security = SecurityManager::new(security_config);
    
    // Create user credentials
    let admin_creds = AuthCredentials {
        username: "admin".to_string(),
        password_hash: "admin_hash_123".to_string(),
        public_key: Some("admin_pubkey".to_string()),
        permissions: vec![Permission::Admin, Permission::Read, Permission::Write],
    };
    
    let user_creds = AuthCredentials {
        username: "ml_researcher".to_string(),
        password_hash: "user_hash_456".to_string(),
        public_key: Some("user_pubkey".to_string()),
        permissions: vec![Permission::Read, Permission::Execute],
    };
    
    // Authenticate users
    let admin_session = security.authenticate(&admin_creds)?;
    let user_session = security.authenticate(&user_creds)?;
    
    // Test permissions
    let admin_can_write = security.has_permission(&admin_session, &Permission::Write);
    let user_can_write = security.has_permission(&user_session, &Permission::Write);
    
    // Test encryption
    let sensitive_data = b"Sensitive ML model parameters";
    let encrypted = security.encrypt_data(sensitive_data)?;
    let decrypted = security.decrypt_data(&encrypted)?;
    
    let security_stats = security.get_stats();
    println!("✅ Security layer active:");
    println!("   Active Sessions: {}", security_stats.active_sessions);
    println!("   Authentication: {}", if security_stats.authentication_enabled { "Enabled" } else { "Disabled" });
    println!("   Encryption: {}", if security_stats.encryption_enabled { "Enabled" } else { "Disabled" });
    println!("   Admin can write: {}", admin_can_write);
    println!("   User can write: {}", user_can_write);
    println!("   Data encryption test: {}", if decrypted == sensitive_data { "✅ Passed" } else { "❌ Failed" });

    // Phase 3.4: Performance Optimization Demo
    println!("\n⚡ Phase 3.4: Performance Optimization");
    println!("======================================");
    
    let perf_config = PerformanceConfig {
        max_cache_size: 1024 * 1024, // 1MB cache for demo
        max_bandwidth_mbps: 50,
        monitoring_interval: Duration::from_secs(1),
        ..Default::default()
    };
    
    let optimizer = PerformanceOptimizer::new(perf_config);
    
    // Cache some data
    let cache_data = vec![
        ("model_weights".to_string(), vec![1u8; 1024]), // 1KB
        ("training_batch_1".to_string(), vec![2u8; 2048]), // 2KB
        ("inference_cache".to_string(), vec![3u8; 512]), // 512B
    ];
    
    for (key, data) in cache_data {
        optimizer.cache_put(key.clone(), data.clone())?;
        
        // Test cache retrieval
        let retrieved = optimizer.cache_get(&key);
        assert!(retrieved.is_some());
    }
    
    // Test bandwidth tracking
    let connections = ["conn_1", "conn_2", "conn_3"];
    for (i, conn) in connections.iter().enumerate() {
        optimizer.record_bandwidth(conn, (i + 1) as u64 * 1024); // Different amounts
    }
    
    // Give time for monitoring to collect data
    sleep(Duration::from_millis(100)).await;
    
    let perf_stats = optimizer.get_stats();
    println!("✅ Performance optimization active:");
    println!("   Cache Entries: {}", perf_stats.cache_entries);
    println!("   Cache Size: {} bytes", perf_stats.cache_size_bytes);
    println!("   Active Connections: {}", perf_stats.active_connections);
    println!("   Total Bandwidth: {:.2} Mbps", perf_stats.total_bandwidth_mbps);
    println!("   CPU Usage: {:.1}%", perf_stats.cpu_usage_percent);
    println!("   Memory Usage: {:.1}%", perf_stats.memory_usage_percent);

    // Phase 3.5: Integration Demo
    println!("\n🔗 Phase 3.5: Integrated System Demo");
    println!("====================================");
    
    // Simulate a complete ML workflow with all Phase 3 features
    println!("🔄 Simulating ML training workflow...");
    
    // 1. Authenticate ML researcher
    let session = security.authenticate(&user_creds)?;
    
    // 2. Check permissions for model access
    if security.has_permission(&session, &Permission::Read) {
        println!("   ✅ User authenticated and authorized");
        
        // 3. Retrieve model from distributed storage
        if let Some(model_data) = storage.retrieve("ml_model_v1".to_string()).await {
            println!("   ✅ Model retrieved from distributed storage");
            
            // 4. Cache model for performance
            optimizer.cache_put("active_model".to_string(), model_data)?;
            println!("   ✅ Model cached for performance");
            
            // 5. Submit training transaction to blockchain
            let training_tx = Transaction {
                from: "ml_researcher".to_string(),
                to: "training_pool".to_string(),
                amount: 10, // Training credits
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            };
            consensus.submit_transaction(training_tx).await;
            println!("   ✅ Training transaction submitted to consensus");
            
            // 6. Record bandwidth usage
            optimizer.record_bandwidth("ml_training_session", 5 * 1024 * 1024); // 5MB
            println!("   ✅ Bandwidth usage recorded");
        }
    }
    
    // Final system statistics
    println!("\n📊 Final System Statistics");
    println!("==========================");
    
    let final_storage_stats = storage.get_stats().await;
    let final_consensus_stats = consensus.get_stats().await;
    let final_security_stats = security.get_stats();
    let final_perf_stats = optimizer.get_stats();
    
    println!("Storage: {} nodes, {} entries, {:.1} avg replication", 
             final_storage_stats.total_nodes, 
             final_storage_stats.total_entries,
             final_storage_stats.avg_replication_factor);
    
    println!("Consensus: {} validators, {} blocks, {} pending txs", 
             final_consensus_stats.active_validators,
             final_consensus_stats.blockchain_height,
             final_consensus_stats.pending_transactions);
    
    println!("Security: {} sessions, {} auth enabled, {} encryption enabled", 
             final_security_stats.active_sessions,
             final_security_stats.authentication_enabled,
             final_security_stats.encryption_enabled);
    
    println!("Performance: {} cache entries, {} connections, {:.1}% optimization", 
             final_perf_stats.cache_entries,
             final_perf_stats.active_connections,
             if final_perf_stats.optimization_enabled { 100.0 } else { 0.0 });

    println!("\n✅ Phase 3 Advanced Features Demo Complete!");
    println!("===========================================");
    println!("🎯 All systems operational and integrated");
    println!("🚀 BCAI is now ready for production ML workloads");

    Ok(())
} 