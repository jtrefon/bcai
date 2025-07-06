//! Phase 2 Network Integration Demo
//!
//! This example demonstrates the enhanced P2P service with
//! large data transfer capabilities.

// Guard all imports and code with #[cfg(feature = "node")] if they depend on it
// or if the example is primarily about node functionality.
// If some parts are independent, they can remain outside.

#[cfg(feature = "node")]
use runtime::node::{UnifiedNode, NodeCapability, CapabilityType};
#[cfg(feature = "node")]
use runtime::large_data_transfer::LargeDataConfig;
#[cfg(feature = "node")]
use std::time::Duration;
#[cfg(feature = "node")]
use tokio::time::sleep;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Phase 2 Network Integration Demo");
    println!("=======================================");

    #[cfg(feature = "node")]
    {
        // Create demo nodes with different capabilities
        let cap_train = NodeCapability {
            cpus: 4, gpus: 1, gpu_memory_gb: 16, available_stake: 1000, reputation: 100,
            capability_types: vec![CapabilityType::Training, CapabilityType::GpuAccelerated],
        };
        let _training_node = UnifiedNode::new( // Mark as unused as P2P part is out
            "training_node_001".to_string(),
            cap_train,
            1_000_000,
        );

        let cap_infer = NodeCapability {
            cpus: 2, gpus: 1, gpu_memory_gb: 8, available_stake: 500, reputation: 100,
            capability_types: vec![CapabilityType::Inference, CapabilityType::GpuAccelerated],
        };
        let _inference_node = UnifiedNode::new( // Mark as unused
            "inference_node_001".to_string(),
            cap_infer,
            1_000_000,
        );

        let cap_store = NodeCapability {
            cpus: 2, gpus: 0, gpu_memory_gb: 0, available_stake: 2000, reputation: 100,
            capability_types: vec![CapabilityType::Storage, CapabilityType::Network],
        };
        let _storage_node = UnifiedNode::new( // Mark as unused
            "storage_node_001".to_string(),
            cap_store,
            1_000_000,
        );

        println!("🏗️  Created 3 demo nodes with varied capabilities (if 'node' feature is enabled).");

        // Commenting out P2P service parts as EnhancedP2PService is not found
        // and porting to current p2p_service is a larger task.
        /*
        // This section would also need to be under #[cfg(feature = "node")] if it uses training_node etc.
        // And potentially another feature for EnhancedP2PService itself if it were to exist.
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
            // ... other configs
        ];

        let nodes = vec![_training_node, _inference_node, _storage_node];
        let mut services = Vec::new();

        // Initialize enhanced P2P services
        for (i, (config, node)) in configs.into_iter().zip(nodes).enumerate() {
            // let service = EnhancedP2PService::new(config, node)?; // EnhancedP2PService missing
            // service.start().await?;
            // services.push(service);

            println!("✅ Started enhanced P2P service for node {} on port {}",
                     i, 4000 + i);
        }

        sleep(Duration::from_secs(2)).await;

        println!("\n📊 Network Performance Metrics");
        // ... stats logic ...
        */
        println!("ℹ️  P2P service section skipped (EnhancedP2PService missing/refactored).");
    } // End of #[cfg(feature = "node")] block
    
    #[cfg(not(feature = "node"))]
    {
        println!("ℹ️  Node creation and P2P service section skipped ('node' feature not enabled).");
    }
    
    println!("✅ Phase 2 Demo conceptually compiled (P2P section commented out, node creation conditional).");

    Ok(())
} 