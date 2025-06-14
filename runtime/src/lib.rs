//! BCAI Runtime - Minimal Working Version
//! 
//! This is a simplified version that compiles cleanly.

// Types are defined directly in this file

/// Simple VM error types
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VmError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("division by zero")]
    DivisionByZero,
    #[error("memory address {0} is invalid")]
    InvalidMemoryAddress(usize),
    #[error("program counter {0} is out of bounds")]
    InvalidProgramCounter(usize),
    #[error("stack overflow: maximum size {0} exceeded")]
    StackOverflow(usize),
    #[error("tensor operation failed: {0}")]
    TensorError(String),
    #[error("resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("hardware error: {0}")]
    HardwareError(String),
    #[error("python execution failed: {0}")]
    PythonError(String),
    #[error("ML instruction error: {0}")]
    MLInstructionError(String),
}

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

/// Simple blockchain types
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
