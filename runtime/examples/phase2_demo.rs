//! Phase 2 Network Integration Demo
//!
//! This example demonstrates the enhanced P2P service with
//! large data transfer capabilities.

use runtime::{
    enhanced_p2p_service::{EnhancedP2PService, EnhancedP2PConfig},
    UnifiedNode, NodeCapability,
};
use runtime::large_data_transfer::LargeDataConfig;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Phase 2 Network Integration Demo");
    println!("=======================================");

    // Create demo nodes with different capabilities
    let training_node = UnifiedNode::new(
        "training_node_001".to_string(),
        NodeCapability::Training,
    );
    
    let inference_node = UnifiedNode::new(
        "inference_node_001".to_string(),
        NodeCapability::Inference,
    );
    
    let storage_node = UnifiedNode::new(
        "storage_node_001".to_string(),
        NodeCapability::Storage,
    );

    println!("🏗️  Created 3 demo nodes with varied capabilities");

    // Create enhanced P2P service configurations
    let configs = vec![
        EnhancedP2PConfig {
            listen_port: 4000,
            enable_large_data_transfer: true,
            max_peers: 10,
            large_data_config: LargeDataConfig::default(),
            peer_discovery_interval: Duration::from_secs(5),
            heartbeat_interval: Duration::from_secs(10),
            bootstrap_peers: vec![],
        },
        EnhancedP2PConfig {
            listen_port: 4001,
            enable_large_data_transfer: true,
            max_peers: 10,
            large_data_config: LargeDataConfig::default(),
            peer_discovery_interval: Duration::from_secs(5),
            heartbeat_interval: Duration::from_secs(10),
            bootstrap_peers: vec!["127.0.0.1:4000".to_string()],
        },
        EnhancedP2PConfig {
            listen_port: 4002,
            enable_large_data_transfer: true,
            max_peers: 10,
            large_data_config: LargeDataConfig::default(),
            peer_discovery_interval: Duration::from_secs(5),
            heartbeat_interval: Duration::from_secs(10),
            bootstrap_peers: vec!["127.0.0.1:4000".to_string()],
        },
    ];

    let nodes = vec![training_node, inference_node, storage_node];
    let mut services = Vec::new();

    // Initialize enhanced P2P services
    for (i, (config, node)) in configs.into_iter().zip(nodes).enumerate() {
        let service = EnhancedP2PService::new(config, node)?;
        service.start().await?;
        services.push(service);
        
        println!("✅ Started enhanced P2P service for node {} on port {}", 
                 i, 4000 + i);
    }

    // Wait for services to initialize
    sleep(Duration::from_secs(2)).await;

    // Show network statistics
    println!("\n📊 Network Performance Metrics");
    println!("==============================");
    
    for (i, service) in services.iter().enumerate() {
        let stats = service.get_stats().await;
        println!("Node {} Statistics:", i);
        println!("  📡 Peers: {}", stats.network_stats.connected_peers);
        println!("  📦 Available Chunks: {}", stats.network_stats.available_chunks);
        println!("  ⬆️  Upload: {:.1} Mbps", stats.network_stats.total_upload_mbps);
        println!("  ⬇️  Download: {:.1} Mbps", stats.network_stats.total_download_mbps);
        println!("  💾 Cache Hit Rate: {:.1}%", stats.chunk_cache_hit_rate * 100.0);
        println!();
    }

    // Show final statistics
    println!("✅ Phase 2 Integration Demo Complete!");
    println!("====================================");
    
    let total_peers: usize = services.iter()
        .map(|s| tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                s.get_stats().await.peer_count
            })
        }))
        .sum();
    
    println!("📊 Final Network State:");
    println!("   Total Network Size: {} nodes", services.len());
    println!("   Total Peer Connections: {}", total_peers);
    println!("   Large Data Transfer: ✅ Enabled");
    println!("   Network Integration: ✅ Complete");

    Ok(())
} 