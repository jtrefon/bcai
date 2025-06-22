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

// --- Data Model & Placeholder Modules ---
pub mod consensus_engine;
#[cfg(feature="bridge")] pub mod cross_chain_bridge;
pub mod distributed_storage;
pub mod federated;
#[cfg(feature="federated-coord")] pub mod federated_network_coordinator;
pub mod large_data_transfer;
pub mod performance_optimizer;
pub mod security_layer;

// Note: The `token` module has been removed as its functionality is
// now part of the `blockchain::state` module.

#[cfg(not(feature = "node"))]
pub mod node {
    use serde::{Serialize, Deserialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
    pub enum NodeCapability {
        Cpu,
        Gpu,
        Tpu,
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
