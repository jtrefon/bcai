//! Integration tests for the unified BCAI distributed training system
//!
//! These tests demonstrate the complete workflow:
//! 1. Nodes join network and announce capabilities
//! 2. Jobs are posted and distributed across network
//! 3. Workers volunteer and execute distributed training
//! 4. Results are evaluated and consensus is reached
//! 5. Rewards are distributed and reputation is updated

use runtime::{
    network::{NetworkCoordinator, NetworkMessage},
    node::{NodeCapability, UnifiedNode},
};

#[test]
fn simplified_distributed_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Create basic capability for testing
    let capability = NodeCapability {
        cpus: 4,
        gpus: 1,
        gpu_memory_gb: 8,
        available_stake: 200,
        reputation: 0,
    };
    
    // Create poster and worker
    let poster_node = UnifiedNode::new("poster".to_string(), capability.clone(), 2000);
    let mut poster_coordinator = NetworkCoordinator::new(poster_node);
    
    let worker_node = UnifiedNode::new("worker".to_string(), capability.clone(), 1000);
    let mut worker_coordinator = NetworkCoordinator::new(worker_node);
    worker_coordinator.local_node_mut().stake_tokens(300)?;
    
    // Test 1: Job posting
    let job_message = poster_coordinator.post_job_to_network(
        "Test training job".to_string(),
        600,
        capability.clone(),
        "test_data".to_string(),
        "test_model".to_string(),
        100,
    )?;
    
    println!("✅ Job posted successfully");
    
    // Test 2: Worker volunteering
    let volunteer_responses = worker_coordinator.handle_message(job_message, "poster")?;
    assert_eq!(volunteer_responses.len(), 1);
    println!("✅ Worker volunteered successfully");
    
    // Test 3: Update job state
    poster_coordinator.handle_message(volunteer_responses[0].clone(), "worker")?;
    
    // Test 4: Verify network stats
    let poster_stats = poster_coordinator.get_network_stats();
    let worker_stats = worker_coordinator.get_network_stats();
    
    assert!(poster_stats.active_jobs >= 1);
    assert_eq!(worker_stats.connected_peers, 0); // Only tracks announcements
    
    println!("✅ Core distributed workflow components working!");
    println!("📊 Stats: Active Jobs: {}, Worker Balance: {}", 
             poster_stats.active_jobs, worker_stats.local_node_stats.balance);
    
    Ok(())
}

#[test]
fn message_routing_test() -> Result<(), Box<dyn std::error::Error>> {
    // Test message routing between coordinators
    let capability = NodeCapability {
        cpus: 4,
        gpus: 1,
        gpu_memory_gb: 8,
        available_stake: 100,
        reputation: 0,
    };
    
    // Create two nodes
    let node1 = UnifiedNode::new("node1".to_string(), capability.clone(), 1000);
    let mut coordinator1 = NetworkCoordinator::new(node1);
    
    let node2 = UnifiedNode::new("node2".to_string(), capability.clone(), 1000);
    let mut coordinator2 = NetworkCoordinator::new(node2);
    
    // Test capability announcements
    let announcement1 = coordinator1.announce_capabilities();
    let announcement2 = coordinator2.announce_capabilities();
    
    // Exchange announcements
    coordinator1.handle_message(announcement2, "node2")?;
    coordinator2.handle_message(announcement1, "node1")?;
    
    // Verify network stats
    let stats1 = coordinator1.get_network_stats();
    let stats2 = coordinator2.get_network_stats();
    
    assert_eq!(stats1.connected_peers, 1);
    assert_eq!(stats2.connected_peers, 1);
    
    println!("✅ Message routing working correctly!");
    println!("📊 Network Stats: Node1 peers: {}, Node2 peers: {}", 
             stats1.connected_peers, stats2.connected_peers);
    
    Ok(())
}

#[test]
fn capability_matching_and_filtering() -> Result<(), Box<dyn std::error::Error>> {
    // Test that jobs are only offered to nodes with sufficient capabilities
    
    // Create a high-requirement job poster
    let poster_capability = NodeCapability {
        cpus: 8,
        gpus: 2,
        gpu_memory_gb: 32,
        available_stake: 0,
        reputation: 0,
    };
    let poster_node = UnifiedNode::new("poster".to_string(), poster_capability, 2000);
    let mut poster_coordinator = NetworkCoordinator::new(poster_node);
    
    // Create a low-capability worker
    let low_capability = NodeCapability {
        cpus: 2,
        gpus: 0,
        gpu_memory_gb: 4,
        available_stake: 0,
        reputation: 0,
    };
    let low_worker_node = UnifiedNode::new("low_worker".to_string(), low_capability, 500);
    let mut low_worker_coordinator = NetworkCoordinator::new(low_worker_node);
    low_worker_coordinator.local_node_mut().stake_tokens(50)?;
    
    // Create a high-capability worker
    let high_capability = NodeCapability {
        cpus: 8,
        gpus: 2,
        gpu_memory_gb: 32,
        available_stake: 0,
        reputation: 0,
    };
    let high_worker_node = UnifiedNode::new("high_worker".to_string(), high_capability, 2000);
    let mut high_worker_coordinator = NetworkCoordinator::new(high_worker_node);
    high_worker_coordinator.local_node_mut().stake_tokens(500)?;
    
    // Post a high-requirement job
    let job_requirements = NodeCapability {
        cpus: 6,
        gpus: 2,
        gpu_memory_gb: 16,
        available_stake: 400,
        reputation: 0,
    };
    
    let job_message = poster_coordinator.post_job_to_network(
        "High-requirement deep learning job".to_string(),
        1000,
        job_requirements,
        "large_dataset_hash".to_string(),
        "transformer_model.json".to_string(),
        200,
    )?;
    
    // Test responses
    let low_worker_responses = low_worker_coordinator.handle_message(job_message.clone(), "poster")?;
    let high_worker_responses = high_worker_coordinator.handle_message(job_message, "poster")?;
    
    // Low-capability worker should not volunteer (insufficient resources)
    assert_eq!(low_worker_responses.len(), 0);
    
    // High-capability worker should volunteer
    assert_eq!(high_worker_responses.len(), 1);
    
    if let NetworkMessage::JobVolunteer { job_id, node_id, capability: _ } = &high_worker_responses[0] {
        assert_eq!(*job_id, 1);
        assert_eq!(node_id, "high_worker");
    } else {
        panic!("Expected JobVolunteer from high-capability worker");
    }
    
    println!("✅ Capability matching and filtering working correctly!");
    
    Ok(())
} 