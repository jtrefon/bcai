use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for distributed storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Replication factor (number of copies)
    pub replication_factor: u32,
    /// Maximum storage capacity per node (bytes)
    pub max_storage_capacity: u64,
    /// Consistency level for reads/writes
    pub consistency_level: ConsistencyLevel,
    /// Storage cleanup interval
    pub cleanup_interval: Duration,
    /// Enable compression for stored data
    pub enable_compression: bool,
    /// Enable encryption for stored data
    pub enable_encryption: bool,
    pub avg_replication_factor: f32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            replication_factor: 3,
            max_storage_capacity: 100 * 1024 * 1024 * 1024, // 100GB
            consistency_level: ConsistencyLevel::Quorum,
            cleanup_interval: Duration::from_secs(3600), // 1 hour
            enable_compression: true,
            enable_encryption: true,
            avg_replication_factor: 0.0,
        }
    }
}

/// Consistency levels for distributed operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Read/write from any single replica
    One,
    /// Read/write from majority of replicas
    Quorum,
    /// Read/write from all replicas
    All,
}

/// Storage entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    pub key: String,
    pub size: u64,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub replicas: Vec<String>, // Node IDs where this data is stored
    pub checksum: String,
    pub compression: Option<String>,
    pub encryption: Option<String>,
}

/// Storage operation results
#[derive(Debug, Clone)]
pub enum StorageResult {
    Success,
    NotFound,
    InsufficientReplicas,
    ConsistencyError,
    StorageError(String),
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_nodes: usize,
    pub total_entries: usize,
    pub total_size: u64,
    pub local_entries: usize,
    pub local_size: u64,
    pub avg_replication_factor: f32,
    pub consistency_level: ConsistencyLevel,
} 