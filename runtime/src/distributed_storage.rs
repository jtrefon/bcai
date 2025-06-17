//! Distributed Storage System for BCAI
//! 
//! This module provides a distributed storage layer with replication,
//! consistency guarantees, and fault tolerance for ML data and models.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use crate::large_data_transfer::chunk::ChunkId;

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

/// Storage node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode {
    pub node_id: String,
    pub address: String,
    pub capacity: u64,
    pub used_space: u64,
    pub last_seen: u64,
    pub reliability_score: f32,
}

impl StorageNode {
    pub fn new(node_id: String, address: String, capacity: u64) -> Self {
        Self {
            node_id,
            address,
            capacity,
            used_space: 0,
            last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            reliability_score: 1.0,
        }
    }

    pub fn available_space(&self) -> u64 {
        self.capacity.saturating_sub(self.used_space)
    }

    pub fn utilization(&self) -> f32 {
        if self.capacity == 0 {
            0.0
        } else {
            self.used_space as f32 / self.capacity as f32
        }
    }
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

// NOTE: Removed placeholder implementation structs:
// - DistributedStorage
// This file now only defines the data models for distributed storage. 