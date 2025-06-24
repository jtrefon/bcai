//! BCAI Runtime - Core Logic
//!
//! This crate contains the core business logic for the BCAI blockchain node,
//! including data structures, networking, and the virtual machine.

// --- Core Modules ---
// These modules have been refactored for clarity and single responsibility.
pub mod blockchain;
pub mod miner;
pub mod network;
#[cfg(feature="node")]
pub mod node;
pub mod pouw;
#[cfg(feature="p2p")]
pub mod p2p_service;
pub mod wire;
pub mod job;
pub mod evaluator;
pub mod trainer;
pub mod job_manager;
pub mod token;

// --- Data Model & Placeholder Modules ---
pub mod consensus_engine;
#[cfg(feature="bridge")] pub mod cross_chain_bridge;
pub mod distributed_storage;
pub mod federated;
#[cfg(feature="federated-coord")] pub mod federated_network_coordinator;
pub mod large_data_transfer;
pub mod performance_optimizer;
pub mod security_layer;

// Temporary token module until full ledger is integrated into
// `blockchain::state`.

#[cfg(not(feature = "node"))]
pub mod node {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum CapabilityType {
        BasicCompute,
        GpuAccelerated,
        HighMemory,
        Storage,
        Network,
        Training,
        Inference,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NodeCapability {
        pub cpus: u32,
        pub gpus: u32,
        pub gpu_memory_gb: u32,
        pub available_stake: u64,
        pub reputation: i32,
        pub capability_types: Vec<CapabilityType>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DistributedJob {
        pub id: u64,
        pub description: String,
        pub required_capability: NodeCapability,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TrainingResult {
        pub job_id: u64,
        pub model_hash: String,
        pub metrics: HashMap<String, f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct NodeStats {
        pub completed_jobs: usize,
        pub uptime_secs: u64,
    }
}
