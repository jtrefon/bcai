//! BCAI Runtime - Core Logic
//!
//! This crate contains the core business logic for the BCAI blockchain node,
//! including data structures, networking, and the virtual machine.

// --- Core Modules ---
// These modules have been refactored for clarity and single responsibility.
pub mod blockchain;
pub mod miner;
pub mod network;
pub mod node;
pub mod pouw;
pub mod p2p_service;
pub mod wire;
pub mod job;

// --- Data Model & Placeholder Modules ---
pub mod consensus_engine;
pub mod cross_chain_bridge;
pub mod distributed_storage;
pub mod federated;
pub mod federated_network_coordinator;
pub mod large_data_transfer;
pub mod performance_optimizer;
pub mod security_layer;

// Note: The `token` module has been removed as its functionality is
// now part of the `blockchain::state` module.
