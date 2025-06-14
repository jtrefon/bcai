//! Simplified Phase 2A Demo
//!
//! This demonstrates the key Phase 2A features:
//! - P2P service integration
//! - Federated learning engine
//! - Advanced consensus validation

use runtime::{
    federated::{FederatedEngine, FederatedConfig, ModelParameters, ModelMetadata, AggregationStrategy},
    node::{UnifiedNode, NodeCapability},
    p2p_service::{P2PConfig, create_p2p_service},
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Phase 2A: Simplified Demo");
    println!("=================================");
    
    // Demo 1: P2P Service Creation and Management
    demo_p2p_service()?;
    
    // Demo 2: Federated Learning Engine
    demo_federated_learning()?;
    
    println!("\n🎉 Phase 2A Simplified Demo completed successfully!");
    Ok(())
}

/// Demonstrate P2P service creation and basic operations
fn demo_p2p_service() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌐 P2P Service Demo");
    println!("==================");
    
    // Create nodes with different capabilities
    let mut nodes = Vec::new();
    
    for i in 0..3 {
        let capability = NodeCapability {
            cpus: 8 + (i as u8 * 2),
            gpus: 1 + i as u8,
            gpu_memory_gb: 8 + (i as u16 * 4),
            available_stake: 5000 + (i as u64 * 2000),
            reputation: 80 + (i as i32 * 5),
        };
        let node = UnifiedNode::new(format!("node_{}", i), capability, 25000);
        nodes.push(node);
    }
    
    println!("✅ Created {} nodes with varying capabilities", nodes.len());
    
    // Create P2P services for each node
    let mut services = Vec::new();
    
    for (i, node) in nodes.into_iter().enumerate() {
        let config = P2PConfig {
            listen_port: 4001 + i as u16,
            bootstrap_peers: if i == 0 {
                vec![]
            } else {
                vec![format!("/ip4/127.0.0.1/tcp/4001")]
            },
            max_peers: 20,
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(10),
        };
        
        let (mut service, handle) = create_p2p_service(config, node);
        
        // Start the service
        service.start()?;
        handle.start()?;
        
        // Get and display stats
        let stats = service.get_stats();
        println!("📊 Node {} - Port: {}, Peers: {}, Uptime: {:?}",
                 i, 4001 + i as u16, stats.peer_count, stats.uptime);
        
        services.push((service, handle));
    }
    
    println!("🌐 P2P network established with {} services", services.len());
    
    Ok(())
}

/// Demonstrate federated learning with model aggregation
fn demo_federated_learning() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Federated Learning Demo");
    println!("==========================");
    
    // Create federated learning engine with weighted averaging
    let config = FederatedConfig {
        min_participants: 3,
        max_participants: 8,
        aggregation_strategy: AggregationStrategy::WeightedAveraging,
        training_rounds: 3,
        convergence_threshold: 0.05,
        participant_timeout: Duration::from_secs(300),
        model_validation_enabled: true,
    };
    
    let mut fed_engine = FederatedEngine::new(config);
    
    // Add participants with different data sizes
    let participants = vec![
        ("high_data_node", 2000),
        ("medium_data_node", 1200),
        ("small_data_node", 800),
        ("edge_node", 400),
    ];
    
    for (node_id, data_size) in &participants {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 3000,
            reputation: 85,
        };
        fed_engine.add_participant(node_id.to_string(), capability, *data_size)?;
        println!("📥 Added participant: {} (data size: {})", node_id, data_size);
    }
    
    // Initialize global model
    let initial_model = ModelParameters {
        weights: vec![0.1, 0.2, 0.3],
        biases: vec![0.01, 0.02],
        layer_sizes: vec![10, 5, 2],
        metadata: ModelMetadata {
            model_id: "simple_classifier".to_string(),
            version: 1,
            training_rounds: 0,
            participant_count: 0,
            accuracy: 0.2, // Random baseline
            loss: 1.6,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        },
    };
    
    fed_engine.initialize_global_model(initial_model)?;
    println!("🚀 Initialized global model");
    
    // Simulate federated training rounds
    for round in 1..=3 {
        println!("\n🔄 Federated Training Round {}", round);
        
        // Create local models from participants
        let mut local_models = Vec::new();
        
        for (i, (node_id, _)) in participants.iter().enumerate() {
            // Simulate local training with improving metrics
            let local_model = ModelParameters {
                weights: vec![
                    0.1 + (round as f32 * 0.05) + (i as f32 * 0.02),
                    0.2 + (round as f32 * 0.04) + (i as f32 * 0.01),
                    0.3 + (round as f32 * 0.03) + (i as f32 * 0.01),
                ],
                biases: vec![
                    0.01 + (round as f32 * 0.001),
                    0.02 + (round as f32 * 0.001),
                ],
                layer_sizes: vec![10, 5, 2],
                metadata: ModelMetadata {
                    model_id: "simple_classifier".to_string(),
                    version: round as u32,
                    training_rounds: round as u32,
                    participant_count: 1,
                    accuracy: 0.2 + (round as f32 * 0.2) + (i as f32 * 0.05),
                    loss: 1.6 - (round as f32 * 0.3) - (i as f32 * 0.1),
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                },
            };
            
            local_models.push((node_id.to_string(), local_model));
            println!("   📤 {} uploaded local model (acc: {:.3})",
                     node_id, 0.2 + (round as f32 * 0.2) + (i as f32 * 0.05));
        }
        
        // Aggregate models using federated averaging
        let global_model = fed_engine.aggregate_models(local_models)?;
        
        println!("   🔄 Aggregated {} models", participants.len());
        println!("   📈 Global accuracy: {:.3}", global_model.metadata.accuracy);
        println!("   📉 Global loss: {:.3}", global_model.metadata.loss);
        
        // Check for convergence
        if fed_engine.has_converged() {
            println!("   ✅ Training converged!");
            break;
        }
        
        // Brief pause between rounds
        std::thread::sleep(Duration::from_millis(200));
    }
    
    // Display final statistics
    let stats = fed_engine.get_federated_stats();
    println!("\n📊 Final Federated Learning Results:");
    println!("   Total rounds: {}", stats.current_round);
    println!("   Participants: {}", stats.total_participants);
    println!("   Final accuracy: {:.3}", stats.global_accuracy);
    println!("   Final loss: {:.3}", stats.global_loss);
    println!("   Convergence score: {:.3}", stats.convergence_score);
    println!("   Converged: {}", stats.has_converged);
    println!("   Total training time: {:?}", stats.total_training_time);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_p2p_demo() {
        // Test that the P2P demo can be created without errors
        assert!(demo_p2p_service().is_ok());
    }
    
    #[test]
    fn test_federated_demo() {
        // Test that the federated learning demo works
        assert!(demo_federated_learning().is_ok());
    }
} 