//! BCAI Runtime - Enhanced VM with ML-First Architecture
//! 
//! This runtime provides both basic VM functionality and enhanced ML capabilities.

// Core modules (always available and working)
pub mod instruction;
pub mod large_data_transfer;
pub mod vm;
pub mod enhanced_p2p_service;

// Enhanced VM modules (optional, behind enhanced-vm feature)
#[cfg(feature = "enhanced-vm")]
pub mod enhanced_vm;
#[cfg(feature = "enhanced-vm")]
pub mod ml_instructions; 
#[cfg(feature = "enhanced-vm")]
pub mod tensor_ops;
#[cfg(feature = "enhanced-vm")]
pub mod hardware_abstraction;
#[cfg(feature = "enhanced-vm")]
pub mod python_bridge;

// Working additional modules
pub mod token;
pub mod pouw;

// Re-export core types
pub use instruction::Instruction;
pub use vm::{Vm, VmError};

// Re-export enhanced types conditionally
#[cfg(feature = "enhanced-vm")]
pub use enhanced_vm::{EnhancedVM, VMConfig, ExecutionContext};
#[cfg(feature = "enhanced-vm")]
pub use ml_instructions::MLInstruction;
#[cfg(feature = "enhanced-vm")]
pub use tensor_ops::{Tensor, DataType, TensorId};
#[cfg(feature = "enhanced-vm")]
pub use python_bridge::PythonConstraints;
#[cfg(feature = "enhanced-vm")]
pub use hardware_abstraction::HardwareBackend;

// Re-export working modules
pub use token::{TokenLedger, LedgerError};
pub use pouw::{Task, generate_task, solve, verify};

// Stub types for enhanced_p2p_service compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkMessage {
    Ping,
    Pong,
    Data(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct NetworkCoordinator {
    pub node_id: String,
}

impl NetworkCoordinator {
    pub fn new(node_id: String) -> Self {
        Self { node_id }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NodeCapability {
    BasicCompute,
    Training,
    Inference,
    Storage,
}

#[derive(Debug, Clone)]
pub struct UnifiedNode {
    pub node_id: String,
    pub capability: NodeCapability,
}

impl UnifiedNode {
    pub fn new(node_id: String, capability: NodeCapability) -> Self {
        Self { node_id, capability }
    }
}

// Basic types (always available)
/// Tensor identifier for referencing tensors in VM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TensorId(pub u64);

impl TensorId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Basic data types for VM operations
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DataType {
    Float32,
    Float64,
    Int32,
    Int64,
    Bool,
    String,
}

/// VM Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmConfig {
    pub max_stack_size: usize,
    pub max_memory_size: usize,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_stack_size: 1024,
            max_memory_size: 1024 * 1024, // 1MB
        }
    }
}

// Simple blockchain types for compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub data: String,
    pub hash: String,
    pub previous_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }
    
    pub fn add_block(&mut self, data: String) {
        let index = self.blocks.len() as u64;
        let previous_hash = self.blocks.last()
            .map(|b| b.hash.clone())
            .unwrap_or_else(|| "0".to_string());
        
        let block = Block {
            index,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data,
            hash: format!("hash_{}", index),
            previous_hash,
        };
        
        self.blocks.push(block);
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}
