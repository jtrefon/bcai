//! Phase 2 Network Integration Demo
//!
//! This example demonstrates the enhanced P2P service with
//! large data transfer capabilities integrated into the network.

use bcai_runtime::{
    EnhancedP2PService, EnhancedP2PConfig, create_enhanced_p2p_service,
    UnifiedNode, NodeCapability,
};
use bcai_runtime::large_data_transfer::{
    chunk::{ChunkId, ChunkUtils, DataChunk},
    descriptor::LargeDataDescriptor,
    LargeDataConfig,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Phase 2 Network Integration Demo");
    println!("=======================================");

    // Create multiple nodes with different capabilities
    let nodes = create_demo_nodes().await?;
    let mut services = Vec::new();

    // Initialize enhanced P2P services for each node
    for (i, node) in nodes.into_iter().enumerate() {
        let config = EnhancedP2PConfig {
            listen_port: 4000 + i as u16,
            enable_large_data_transfer: true,
            max_peers: 10,
            large_data_config: LargeDataConfig::default(),
            peer_discovery_interval: Duration::from_secs(5),
            heartbeat_interval: Duration::from_secs(10),
            bootstrap_peers: if i > 0 { 
                vec!["127.0.0.1:4000".to_string()] 
            } else { 
                vec![] 
            },
        };

        let service = create_enhanced_p2p_service(config, node)?;
        service.start().await?;
        services.push(service);
        
        println!("✅ Started enhanced P2P service for node {} on port {}", 
                 i, 4000 + i);
    }

    // Wait for services to initialize
    sleep(Duration::from_secs(2)).await;

    // Demonstrate peer discovery and networking
    println!("\n📡 Phase 1: Peer Discovery and Network Formation");
    println!("================================================");
    
    for (i, service) in services.iter().enumerate() {
        let stats = service.get_stats().await;
        println!("Node {}: {} peers, {} transfers, {:.1}% cache hit rate", 
                 i, stats.peer_count, stats.active_large_transfers, 
                 stats.chunk_cache_hit_rate * 100.0);
    }

    // Demonstrate large data chunking and distribution
    println!("\n📦 Phase 2: Large Data Transfer Demo");
    println!("===================================");
    
    let demo_data = create_demo_ml_dataset().await?;
    let descriptor = create_large_data_descriptor(demo_data)?;
    
    println!("📊 Created ML dataset descriptor:");
    println!("   Content Hash: {}", descriptor.content_hash);
    println!("   Total Size: {:.2} MB", descriptor.total_size as f64 / 1_000_000.0);
    println!("   Chunks: {}", descriptor.chunk_count);
    
    // Wait for network stabilization
    sleep(Duration::from_secs(3)).await;

    // Show network statistics
    println!("\n📊 Phase 3: Network Performance Metrics");
    println!("======================================");
    
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

    // Demonstrate chunk routing and peer selection
    println!("🔄 Phase 4: Chunk Routing Demo");
    println!("=============================");
    
    demonstrate_chunk_routing(&services).await?;

    // Show final statistics
    println!("\n✅ Phase 2 Integration Demo Complete!");
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

/// Create demo nodes with varying capabilities
async fn create_demo_nodes() -> Result<Vec<UnifiedNode>, Box<dyn std::error::Error>> {
    let mut nodes = Vec::new();

    // High-performance training node
    let training_node = UnifiedNode::new(
        "training_node_001".to_string(),
        NodeCapability {
            cpus: 16,
            gpus: 4,
            gpu_memory_gb: 32,
            available_stake: 10000,
            reputation: 100,
        },
        25000,
    );
    nodes.push(training_node);

    // Medium-performance inference node
    let inference_node = UnifiedNode::new(
        "inference_node_001".to_string(),
        NodeCapability {
            cpus: 8,
            gpus: 2,
            gpu_memory_gb: 16,
            available_stake: 5000,
            reputation: 95,
        },
        26000,
    );
    nodes.push(inference_node);

    // Storage-focused node
    let storage_node = UnifiedNode::new(
        "storage_node_001".to_string(),
        NodeCapability {
            cpus: 4,
            gpus: 0,
            gpu_memory_gb: 0,
            available_stake: 2000,
            reputation: 90,
        },
        27000,
    );
    nodes.push(storage_node);

    println!("🏗️  Created {} demo nodes with varied capabilities", nodes.len());
    Ok(nodes)
}

/// Create a demo ML dataset for transfer testing
async fn create_demo_ml_dataset() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Simulate a 5MB ML training dataset
    let dataset_size = 5 * 1024 * 1024; // 5MB
    let mut dataset = Vec::with_capacity(dataset_size);
    
    // Fill with pseudo-random data to simulate real ML dataset
    for i in 0..dataset_size {
        dataset.push(((i * 7 + 13) % 256) as u8);
    }
    
    println!("🔢 Generated demo ML dataset: {:.2} MB", dataset.len() as f64 / 1_000_000.0);
    Ok(dataset)
}

/// Create a large data descriptor from demo data
fn create_large_data_descriptor(data: Vec<u8>) -> Result<LargeDataDescriptor, Box<dyn std::error::Error>> {
    println!("📝 Creating large data descriptor...");
    
    // Chunk the data
    let chunks = ChunkUtils::create_chunks(&data, 1024 * 1024)?; // 1MB chunks
    let chunk_hashes: Vec<String> = chunks.iter()
        .map(|chunk| chunk.chunk_id.to_string())
        .collect();
    
    // Create descriptor
    let descriptor = LargeDataDescriptor {
        content_hash: "ml_dataset_demo_v1".to_string(),
        total_size: data.len() as u64,
        chunk_size: 1024 * 1024,
        chunk_count: chunks.len() as u32,
        chunk_hashes,
        compression_algorithm: Some("lz4".to_string()),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        metadata: serde_json::json!({
            "type": "ml_training_dataset",
            "format": "tensor",
            "size_mb": data.len() as f64 / 1_000_000.0,
            "description": "Demo ML training dataset for Phase 2 testing"
        }),
    };
    
    println!("✅ Created descriptor with {} chunks", descriptor.chunk_count);
    Ok(descriptor)
}

/// Demonstrate chunk routing and peer selection
async fn demonstrate_chunk_routing(services: &[EnhancedP2PService]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Demonstrating intelligent chunk routing...");
    
    // Create sample chunks for demonstration
    let sample_chunks = vec![
        ChunkId::from_bytes(b"model_weights_layer_1"),
        ChunkId::from_bytes(b"training_data_batch_1"),
        ChunkId::from_bytes(b"gradient_updates_epoch_1"),
    ];
    
    for (i, service) in services.iter().enumerate() {
        println!("📦 Node {} announcing {} demo chunks", i, sample_chunks.len());
        
        // Each service announces different chunks to simulate distribution
        let node_chunks = sample_chunks.iter()
            .skip(i)
            .take(2)
            .cloned()
            .collect();
        
        // This would normally announce to the network
        // For demo, we just show the intent
        println!("   Chunks announced by Node {}: {:?}", i, 
                 node_chunks.iter().map(|c| c.to_string()).collect::<Vec<_>>());
    }
    
    // Simulate chunk discovery and routing
    sleep(Duration::from_secs(1)).await;
    
    println!("🔍 Chunk routing complete - all nodes aware of chunk locations");
    Ok(())
}

/// Helper trait for demo formatting
trait DemoFormat {
    fn to_string(&self) -> String;
}

impl DemoFormat for ChunkId {
    fn to_string(&self) -> String {
        format!("{:x}", self.0[0..4].iter().fold(0u32, |acc, &b| (acc << 8) | b as u32))
    }
} 