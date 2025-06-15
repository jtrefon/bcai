//! Time-Bound Permissions Demo
//!
//! Demonstrates:
//! - Temporary access grants with expiration dates
//! - Different access types (Trial, Subscription, Emergency)
//! - Usage limits and automatic cleanup
//! - Subscription model with renewals
//! - Emergency access scenarios

use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;
use chrono::{DateTime, Utc};

use runtime::{
    decentralized_filesystem::{
        DecentralizedFilesystem, DfsConfig, FilePermissions, TemporaryAccessType,
        TemporaryAccess, DfsError
    },
    distributed_storage::{DistributedStorage, StorageConfig, StorageNode},  
    token::TokenLedger,
    smart_contracts::SmartContractEngine,
    network::NetworkCoordinator,
    node::{UnifiedNode, NodeCapability, CapabilityType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕰️  Time-Bound Permissions System Demo");
    println!("=====================================");
    
    // Setup infrastructure
    let dfs = setup_demo_infrastructure().await?;
    
    // Demo scenarios
    run_temporary_access_demo(&dfs).await?;
    run_trial_access_demo(&dfs).await?;
    run_subscription_model_demo(&dfs).await?;
    run_emergency_access_demo(&dfs).await?;
    run_usage_limits_demo(&dfs).await?;
    run_expiration_cleanup_demo(&dfs).await?;
    
    println!("\n🎉 Time-bound permissions demo completed successfully!");
    Ok(())
}

async fn setup_demo_infrastructure() -> Result<Arc<DecentralizedFilesystem>, Box<dyn std::error::Error>> {
    println!("\n🏗️  Setting up demo infrastructure...");
    
    // Create node with storage capability
    let local_node = UnifiedNode::new(
        "demo_node".to_string(),
        NodeCapability {
            cpus: 8,
            gpus: 2,
            gpu_memory_gb: 16,
            available_stake: 10000,
            reputation: 100,
            capability_types: vec![CapabilityType::Storage, CapabilityType::BasicCompute],
        },
        10000, // Initial tokens
    );
    let local_node = Arc::new(AsyncRwLock::new(local_node));
    
    // Setup components
    let temp_node = local_node.read().await.clone();
    let network_coordinator = Arc::new(AsyncRwLock::new(NetworkCoordinator::new(temp_node)));
    let storage_config = StorageConfig::default();
    let storage_coordinator = Arc::new(DistributedStorage::new(storage_config, "demo_node".to_string()));
    let token_ledger = Arc::new(AsyncRwLock::new(TokenLedger::new()));
    let contract_engine = Arc::new(AsyncRwLock::new(SmartContractEngine::new()));
    
    // Setup token accounts for demo users
    {
        let mut ledger = token_ledger.write().await;
        ledger.mint("researcher", 50000).unwrap();
        ledger.mint("collaborator", 20000).unwrap();
        ledger.mint("trial_user", 1000).unwrap();
        ledger.mint("emergency_responder", 30000).unwrap();
        ledger.mint("subscriber", 25000).unwrap();
    }
    
    // Create DFS
    let config = DfsConfig::default();
    let dfs = Arc::new(DecentralizedFilesystem::new(
        config,
        local_node,
        network_coordinator,
        storage_coordinator,
        None,
        token_ledger,
        contract_engine,
    ));
    
    // Add some storage nodes
    add_demo_storage_nodes(&dfs).await?;
    
    println!("✅ Infrastructure setup complete");
    Ok(dfs)
}

async fn add_demo_storage_nodes(dfs: &DecentralizedFilesystem) -> Result<(), DfsError> {
    use runtime::decentralized_filesystem::StorageNodeMetrics;
    
    let storage_nodes = vec![
        StorageNodeMetrics {
            node_id: "storage_node_1".to_string(),
            total_storage: 1000 * 1024 * 1024 * 1024, // 1TB
            available_storage: 800 * 1024 * 1024 * 1024, // 800GB free
            reliability: 0.99,
            avg_response_time: 50,
            bandwidth_capacity: 100 * 1024 * 1024, // 100 MB/s
            total_earnings: 5000,
            active_contracts: 25,
            last_heartbeat: Utc::now(),
            region: "us-east-1".to_string(),
        },
        StorageNodeMetrics {
            node_id: "storage_node_2".to_string(),
            total_storage: 2000 * 1024 * 1024 * 1024, // 2TB
            available_storage: 1500 * 1024 * 1024 * 1024, // 1.5TB free
            reliability: 0.97,
            avg_response_time: 75,
            bandwidth_capacity: 80 * 1024 * 1024, // 80 MB/s
            total_earnings: 8200,
            active_contracts: 40,
            last_heartbeat: Utc::now(),
            region: "us-west-2".to_string(),
        },
    ];
    
    for node in storage_nodes {
        dfs.add_storage_node_metrics(node).await;
    }
    
    Ok(())
}

/// Demo 1: Basic temporary access
async fn run_temporary_access_demo(dfs: &DecentralizedFilesystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 1: Basic Temporary Access");
    println!("==================================");
    
    // Store a confidential research file
    let research_data = b"Confidential AI research data - breakthrough findings on neural architecture".to_vec();
    let file_hash = dfs.store_file_with_permissions(
        "research_breakthrough.pdf".to_string(),
        research_data,
        "application/pdf".to_string(),
        "researcher".to_string(),
        FilePermissions::OwnerOnly {
            owner: "researcher".to_string(),
            encrypted_key: "researcher_key_123".to_string(),
        },
        168, // 1 week storage
        vec!["research".to_string(), "confidential".to_string()],
    ).await?;
    
    println!("📄 Stored confidential research file: {}", file_hash);
    
    // Grant temporary access to collaborator for 2 hours
    println!("\n⏰ Granting temporary access to collaborator...");
    dfs.grant_temporary_access(
        file_hash.clone(),
        "collaborator".to_string(),
        TemporaryAccessType::ReadOnly,
        Duration::from_secs(2 * 60 * 60), // 2 hours
        "researcher".to_string(),
        Some(5), // Max 5 uses
    ).await?;
    
    // Collaborator accesses the file
    println!("🔍 Collaborator accessing file...");
    let retrieved_data = dfs.retrieve_file_with_permissions(
        file_hash.clone(),
        "collaborator".to_string(),
    ).await?;
    
    println!("✅ Collaborator successfully accessed {} bytes", retrieved_data.len());
    
    // Show temporary access grants
    let temp_grants = dfs.list_temporary_access(file_hash.clone()).await?;
    println!("📊 Current temporary grants: {}", temp_grants.len());
    for grant in &temp_grants {
        println!("   👤 {}: {} access (expires: {}, used: {}/{})",
            grant.user_id,
            match grant.access_type {
                TemporaryAccessType::ReadOnly => "Read-only",
                TemporaryAccessType::ReadWrite => "Read-write",
                TemporaryAccessType::Trial => "Trial",
                TemporaryAccessType::Emergency => "Emergency",
                TemporaryAccessType::Subscription => "Subscription",
            },
            grant.expires_at.format("%Y-%m-%d %H:%M UTC"),
            grant.usage_count,
            grant.max_usage.unwrap_or(0)
        );
    }
    
    // Revoke access early
    println!("\n🚫 Revoking collaborator's access early...");
    dfs.revoke_temporary_access(
        file_hash.clone(),
        "collaborator".to_string(),
        "researcher".to_string(),
    ).await?;
    
    // Try to access after revocation (should fail)
    println!("🔒 Collaborator trying to access after revocation...");
    match dfs.retrieve_file_with_permissions(file_hash, "collaborator".to_string()).await {
        Ok(_) => println!("❌ Access should have been denied!"),
        Err(e) => println!("✅ Access correctly denied: {}", e),
    }
    
    Ok(())
}

/// Demo 2: Trial access with time limits
async fn run_trial_access_demo(dfs: &DecentralizedFilesystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 2: Trial Access System");
    println!("==============================");
    
    // Store a premium dataset
    let dataset = b"Premium AI training dataset - high-quality labeled data for computer vision".repeat(1000);
    let dataset_size = dataset.len();
    let file_hash = dfs.store_file_with_permissions(
        "premium_cv_dataset.tar.gz".to_string(),
        dataset,
        "application/gzip".to_string(),
        "researcher".to_string(),
        FilePermissions::OwnerOnly {
            owner: "researcher".to_string(),
            encrypted_key: "dataset_key_456".to_string(),
        },
        720, // 1 month storage
        vec!["dataset".to_string(), "premium".to_string(), "computer-vision".to_string()],
    ).await?;
    
    println!("📊 Stored premium dataset: {} ({} bytes)", file_hash, dataset_size);
    
    // Grant trial access for 30 minutes with usage limit
    println!("\n🆓 Granting trial access to trial_user (30 min, max 3 downloads)...");
    dfs.grant_temporary_access(
        file_hash.clone(),
        "trial_user".to_string(),
        TemporaryAccessType::Trial,
        Duration::from_secs(30 * 60), // 30 minutes
        "researcher".to_string(),
        Some(3), // Max 3 downloads
    ).await?;
    
    // Trial user downloads the dataset multiple times
    for i in 1..=4 {
        println!("\n🔽 Trial download attempt #{}", i);
        match dfs.retrieve_file_with_permissions(file_hash.clone(), "trial_user".to_string()).await {
            Ok(data) => {
                println!("✅ Trial download #{} successful: {} bytes", i, data.len());
            },
            Err(e) => {
                println!("❌ Trial download #{} failed: {}", i, e);
                break;
            }
        }
        
        // Small delay between downloads
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Show updated grant stats
    let temp_grants = dfs.list_temporary_access(file_hash).await?;
    if let Some(trial_grant) = temp_grants.iter().find(|g| g.user_id == "trial_user") {
        println!("📈 Trial usage stats: {}/{} downloads used", 
            trial_grant.usage_count, 
            trial_grant.max_usage.unwrap_or(0)
        );
    }
    
    Ok(())
}

/// Demo 3: Subscription model with renewals
async fn run_subscription_model_demo(dfs: &DecentralizedFilesystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 3: Subscription Model");
    println!("=============================");
    
    // Store a subscription-based service dataset
    let service_data = b"Enterprise AI service dataset - continuously updated market intelligence".repeat(500);
    let service_data_size = service_data.len();
    let file_hash = dfs.store_file_with_permissions(
        "market_intelligence_feed.json".to_string(),
        service_data,
        "application/json".to_string(),
        "researcher".to_string(),
        FilePermissions::OwnerOnly {
            owner: "researcher".to_string(),
            encrypted_key: "service_key_789".to_string(),
        },
        2160, // 3 months storage
        vec!["service".to_string(), "intelligence".to_string(), "enterprise".to_string()],
    ).await?;
    
    println!("📡 Stored enterprise service dataset: {} ({} bytes)", file_hash, service_data_size);
    
    // Grant initial subscription access (1 week)
    println!("\n💳 Granting subscription access to subscriber (1 week)...");
    dfs.grant_temporary_access(
        file_hash.clone(),
        "subscriber".to_string(),
        TemporaryAccessType::Subscription,
        Duration::from_secs(7 * 24 * 60 * 60), // 1 week
        "researcher".to_string(),
        Some(100), // Max 100 API calls per week
    ).await?;
    
    // Subscriber uses the service
    println!("🔌 Subscriber accessing service data...");
    for call_num in 1..=5 {
        match dfs.retrieve_file_with_permissions(file_hash.clone(), "subscriber".to_string()).await {
            Ok(data) => {
                println!("✅ API call #{}: {} bytes retrieved", call_num, data.len());
            },
            Err(e) => {
                println!("❌ API call #{} failed: {}", call_num, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // Show subscription usage
    let temp_grants = dfs.list_temporary_access(file_hash.clone()).await?;
    if let Some(sub_grant) = temp_grants.iter().find(|g| g.user_id == "subscriber") {
        println!("📊 Subscription usage: {}/{} API calls used", 
            sub_grant.usage_count, 
            sub_grant.max_usage.unwrap_or(0)
        );
        println!("⏰ Subscription expires: {}", sub_grant.expires_at.format("%Y-%m-%d %H:%M UTC"));
    }
    
    // Simulate subscription renewal
    println!("\n🔄 Renewing subscription for another week...");
    dfs.grant_temporary_access(
        file_hash,
        "subscriber".to_string(),
        TemporaryAccessType::Subscription,
        Duration::from_secs(7 * 24 * 60 * 60), // Another week
        "researcher".to_string(),
        Some(100), // Another 100 API calls
    ).await?;
    
    println!("✅ Subscription renewed successfully");
    
    Ok(())
}

/// Demo 4: Emergency access scenarios
async fn run_emergency_access_demo(dfs: &DecentralizedFilesystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 4: Emergency Access");
    println!("===========================");
    
    // Store critical emergency response data
    let emergency_data = b"CRITICAL: Emergency response protocols and contact information for disaster management".repeat(200);
    let emergency_data_size = emergency_data.len();
    let file_hash = dfs.store_file_with_permissions(
        "emergency_protocols.txt".to_string(),
        emergency_data,
        "text/plain".to_string(),
        "researcher".to_string(),
        FilePermissions::OwnerOnly {
            owner: "researcher".to_string(),
            encrypted_key: "emergency_key_911".to_string(),
        },
        8760, // 1 year storage
        vec!["emergency".to_string(), "critical".to_string(), "protocols".to_string()],
    ).await?;
    
    println!("🚨 Stored emergency protocols: {} ({} bytes)", file_hash, emergency_data_size);
    
    // Grant emergency access (shorter duration, unlimited usage for critical situations)
    println!("\n🆘 Granting emergency access to emergency_responder (4 hours, unlimited usage)...");
    dfs.grant_temporary_access(
        file_hash.clone(),
        "emergency_responder".to_string(),
        TemporaryAccessType::Emergency,
        Duration::from_secs(4 * 60 * 60), // 4 hours
        "researcher".to_string(),
        None, // No usage limit for emergencies
    ).await?;
    
    // Emergency responder accesses critical data multiple times
    println!("🚑 Emergency responder accessing critical data rapidly...");
    for access_num in 1..=8 {
        match dfs.retrieve_file_with_permissions(file_hash.clone(), "emergency_responder".to_string()).await {
            Ok(data) => {
                println!("⚡ Emergency access #{}: {} bytes (PRIORITY)", access_num, data.len());
            },
            Err(e) => {
                println!("❌ Emergency access #{} failed: {}", access_num, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await; // Rapid access
    }
    
    // Show emergency access stats
    let temp_grants = dfs.list_temporary_access(file_hash).await?;
    if let Some(emergency_grant) = temp_grants.iter().find(|g| g.user_id == "emergency_responder") {
        println!("🚨 Emergency access stats: {} uses (unlimited)", emergency_grant.usage_count);
        println!("⏰ Emergency access expires: {}", emergency_grant.expires_at.format("%Y-%m-%d %H:%M UTC"));
    }
    
    Ok(())
}

/// Demo 5: Usage limits and tracking
async fn run_usage_limits_demo(dfs: &DecentralizedFilesystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 5: Usage Limits & Tracking");
    println!("==================================");
    
    // Store limited-access premium content
    let premium_content = b"Premium AI model weights - state-of-the-art language model trained on proprietary data".repeat(300);
    let premium_content_size = premium_content.len();
    let file_hash = dfs.store_file_with_permissions(
        "premium_ai_model.bin".to_string(),
        premium_content,
        "application/octet-stream".to_string(),
        "researcher".to_string(),
        FilePermissions::OwnerOnly {
            owner: "researcher".to_string(),
            encrypted_key: "model_key_premium".to_string(),
        },
        168, // 1 week
        vec!["ai-model".to_string(), "premium".to_string(), "weights".to_string()],
    ).await?;
    
    println!("🤖 Stored premium AI model: {} ({} bytes)", file_hash, premium_content_size);
    
    // Grant limited access (strict usage limits)
    println!("\n🎯 Granting limited access (24 hours, max 2 downloads)...");
    dfs.grant_temporary_access(
        file_hash.clone(),
        "trial_user".to_string(),
        TemporaryAccessType::Trial,
        Duration::from_secs(24 * 60 * 60), // 24 hours
        "researcher".to_string(),
        Some(2), // Only 2 downloads allowed
    ).await?;
    
    // Test usage limits
    println!("\n📥 Testing usage limits...");
    for attempt in 1..=4 {
        println!("📥 Download attempt #{}...", attempt);
        
        match dfs.retrieve_file_with_permissions(file_hash.clone(), "trial_user".to_string()).await {
            Ok(data) => {
                println!("✅  Download #{} successful: {} bytes", attempt, data.len());
                
                // Show current usage
                let temp_grants = dfs.list_temporary_access(file_hash.clone()).await?;
                if let Some(grant) = temp_grants.iter().find(|g| g.user_id == "trial_user") {
                    println!("📊  Usage: {}/{} downloads", 
                        grant.usage_count, 
                        grant.max_usage.unwrap_or(0)
                    );
                }
            },
            Err(e) => {
                println!("❌  Download #{} failed: {}", attempt, e);
                println!("🚫  Usage limit reached!");
                break;
            }
        }
        
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    
    Ok(())
}

/// Demo 6: Automatic expiration and cleanup
async fn run_expiration_cleanup_demo(dfs: &DecentralizedFilesystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 6: Expiration & Cleanup");
    println!("===============================");
    
    // Create multiple files with different expiration times
    let mut file_hashes = Vec::new();
    
    for i in 1..=3 {
        let temp_data = format!("Temporary file #{} - expires soon", i).repeat(50);
        let file_hash = dfs.store_file_with_permissions(
            format!("temp_file_{}.txt", i),
            temp_data.into_bytes(),
            "text/plain".to_string(),
            "researcher".to_string(),
            FilePermissions::OwnerOnly {
                owner: "researcher".to_string(),
                encrypted_key: format!("temp_key_{}", i),
            },
            24, // 1 day storage
            vec!["temporary".to_string()],
        ).await?;
        
        file_hashes.push(file_hash);
    }
    
    println!("📁 Created {} temporary files", file_hashes.len());
    
    // Grant various temporary access with different expiration times
    println!("\n⏰ Granting temporary access with different expiration times...");
    
    // Very short access (5 seconds) - will expire quickly
    dfs.grant_temporary_access(
        file_hashes[0].clone(),
        "trial_user".to_string(),
        TemporaryAccessType::Trial,
        Duration::from_secs(5),
        "researcher".to_string(),
        Some(10),
    ).await?;
    
    // Medium access (10 seconds)
    dfs.grant_temporary_access(
        file_hashes[1].clone(),
        "collaborator".to_string(),
        TemporaryAccessType::ReadOnly,
        Duration::from_secs(10),
        "researcher".to_string(),
        Some(5),
    ).await?;
    
    // Longer access (1 hour) - won't expire during demo
    dfs.grant_temporary_access(
        file_hashes[2].clone(),
        "subscriber".to_string(),
        TemporaryAccessType::Subscription,
        Duration::from_secs(60 * 60),
        "researcher".to_string(),
        Some(100),
    ).await?;
    
    // Show all temporary grants before expiration
    let mut total_grants = 0;
    for (i, file_hash) in file_hashes.iter().enumerate() {
        let grants = dfs.list_temporary_access(file_hash.clone()).await?;
        total_grants += grants.len();
        println!("📄 File {}: {} temporary grants", i + 1, grants.len());
    }
    println!("📊 Total temporary grants: {}", total_grants);
    
    // Wait for some grants to expire
    println!("\n⏳ Waiting for short-term grants to expire...");
    tokio::time::sleep(Duration::from_secs(12)).await;
    
    // Try to access expired grants
    println!("\n🔒 Testing access after expiration...");
    
    match dfs.retrieve_file_with_permissions(file_hashes[0].clone(), "trial_user".to_string()).await {
        Ok(_) => println!("❌ Short-term access should have expired!"),
        Err(e) => println!("✅ Short-term access correctly expired: {}", e),
    }
    
    match dfs.retrieve_file_with_permissions(file_hashes[1].clone(), "collaborator".to_string()).await {
        Ok(_) => println!("❌ Medium-term access should have expired!"),
        Err(e) => println!("✅ Medium-term access correctly expired: {}", e),
    }
    
    match dfs.retrieve_file_with_permissions(file_hashes[2].clone(), "subscriber".to_string()).await {
        Ok(data) => println!("✅ Long-term access still valid: {} bytes", data.len()),
        Err(e) => println!("❌ Long-term access should still be valid: {}", e),
    }
    
    // Run cleanup to remove expired grants
    println!("\n🧹 Running automatic cleanup of expired grants...");
    let cleaned_count = dfs.cleanup_expired_access().await?;
    println!("✅ Cleanup complete: removed {} expired grants", cleaned_count);
    
    // Verify cleanup worked
    let mut remaining_grants = 0;
    for (i, file_hash) in file_hashes.iter().enumerate() {
        let grants = dfs.list_temporary_access(file_hash.clone()).await?;
        remaining_grants += grants.len();
        println!("📄 File {} after cleanup: {} grants", i + 1, grants.len());
    }
    println!("📊 Total remaining grants: {}", remaining_grants);
    
    Ok(())
} 