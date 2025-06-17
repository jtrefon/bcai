//! BCAI Runtime - Core Logic
//!
//! This crate contains the core business logic for the BCAI blockchain node,
//! including data structures, networking, and the virtual machine.

// --- Core Modules ---
// These modules have been refactored for clarity and single responsibility.
pub mod blockchain;
pub mod miner;
pub mod network;
pub mod pouw;
pub mod p2p_service;
pub mod wire;

// --- Legacy or Unrefactored Modules ---
// These may require future attention.
pub mod advanced_governance;
pub mod consensus_engine;
pub mod consensus_node;
pub mod cross_chain_bridge;
pub mod decentralized_filesystem;
pub mod distributed_storage;
pub mod double;
pub mod enhanced_p2p_service;
pub mod enhanced_vm;
pub mod evaluator;
pub mod federated;
pub mod federated_network_coordinator;
pub mod gpu;
pub mod hardware_abstraction;
pub mod instruction;
pub mod job_manager;
pub mod large_data_transfer;
pub mod ml_instructions;
pub mod mnist;
pub mod monitoring;
pub mod neural_network;
pub mod node;
pub mod p2p_security;
pub mod performance_optimizer;
pub mod python_bridge;
pub mod security;
pub mod security_layer;
pub mod smart_contracts;
pub mod tensor_ops;
pub mod trainer;
pub mod vm;

// Note: The `token` module has been removed as its functionality is
// now part of the `blockchain::state` module.
