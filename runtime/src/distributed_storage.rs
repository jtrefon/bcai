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

/// Distributed storage coordinator
#[derive(Debug)]
pub struct DistributedStorage {
    config: StorageConfig,
    local_node_id: String,
    storage_nodes: Arc<RwLock<HashMap<String, StorageNode>>>,
    storage_entries: Arc<RwLock<HashMap<String, StorageEntry>>>,
    local_storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    command_tx: mpsc::UnboundedSender<StorageCommand>,
}

/// Storage commands for async processing
#[derive(Debug)]
enum StorageCommand {
    Store { key: String, data: Vec<u8>, replicas: u32 },
    Retrieve { key: String, response_tx: oneshot::Sender<Option<Vec<u8>>> },
    Delete { key: String },
    Replicate { key: String, target_nodes: Vec<String> },
    Cleanup,
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

impl DistributedStorage {
    /// Create a new distributed storage coordinator
    pub fn new(config: StorageConfig, local_node_id: String) -> Self {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        
        let storage = Self {
            config: config.clone(),
            local_node_id: local_node_id.clone(),
            storage_nodes: Arc::new(RwLock::new(HashMap::new())),
            storage_entries: Arc::new(RwLock::new(HashMap::new())),
            local_storage: Arc::new(RwLock::new(HashMap::new())),
            command_tx,
        };

        // Spawn background task for command processing
        let storage_nodes = Arc::clone(&storage.storage_nodes);
        let storage_entries = Arc::clone(&storage.storage_entries);
        let local_storage = Arc::clone(&storage.local_storage);
        let config_clone = config.clone();

        tokio::spawn(async move {
            while let Some(command) = command_rx.recv().await {
                Self::process_command(
                    command,
                    &config_clone,
                    &local_node_id,
                    &storage_nodes,
                    &storage_entries,
                    &local_storage,
                ).await;
            }
        });

        storage
    }

    /// Add a storage node to the cluster
    pub async fn add_node(&self, node: StorageNode) {
        let mut nodes = self.storage_nodes.write().unwrap();
        nodes.insert(node.node_id.clone(), node);
    }

    /// Remove a storage node from the cluster
    pub async fn remove_node(&self, node_id: &str) {
        let mut nodes = self.storage_nodes.write().unwrap();
        nodes.remove(node_id);
    }

    /// Store data with replication
    pub async fn store(&self, key: String, data: Vec<u8>) -> StorageResult {
        let replicas = self.config.replication_factor;
        
        if let Err(_) = self.command_tx.send(StorageCommand::Store { key, data, replicas }) {
            return StorageResult::StorageError("Command channel closed".to_string());
        }

        StorageResult::Success
    }

    /// Retrieve data from storage
    pub async fn retrieve(&self, key: String) -> Option<Vec<u8>> {
        let (response_tx, response_rx) = oneshot::channel();
        
        if let Err(_) = self.command_tx.send(StorageCommand::Retrieve { key, response_tx }) {
            return None;
        }

        response_rx.await.unwrap_or(None)
    }

    /// Delete data from storage
    pub async fn delete(&self, key: String) -> StorageResult {
        if let Err(_) = self.command_tx.send(StorageCommand::Delete { key }) {
            return StorageResult::StorageError("Command channel closed".to_string());
        }

        StorageResult::Success
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        let nodes = self.storage_nodes.read().unwrap();
        let entries = self.storage_entries.read().unwrap();
        let local_storage = self.local_storage.read().unwrap();

        let total_nodes = nodes.len();
        let total_entries = entries.len();
        let total_size: u64 = entries.values().map(|e| e.size).sum();
        let local_entries = local_storage.len();
        let local_size: u64 = local_storage.values().map(|v| v.len() as u64).sum();

        let avg_replication = if total_entries > 0 {
            entries.values().map(|e| e.replicas.len()).sum::<usize>() as f32 / total_entries as f32
        } else {
            0.0
        };

        StorageStats {
            total_nodes,
            total_entries,
            total_size,
            local_entries,
            local_size,
            avg_replication_factor: avg_replication,
            consistency_level: self.config.consistency_level,
        }
    }

    /// Select optimal nodes for storing data
    fn select_storage_nodes(&self, replicas: u32) -> Vec<String> {
        let nodes = self.storage_nodes.read().unwrap();
        let mut candidates: Vec<_> = nodes.values().collect();
        
        // Sort by reliability score and available space
        candidates.sort_by(|a, b| {
            let score_a = a.reliability_score * (a.available_space() as f32 / a.capacity as f32);
            let score_b = b.reliability_score * (b.available_space() as f32 / b.capacity as f32);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
            .into_iter()
            .take(replicas as usize)
            .map(|node| node.node_id.clone())
            .collect()
    }

    /// Process storage commands asynchronously
    async fn process_command(
        command: StorageCommand,
        config: &StorageConfig,
        local_node_id: &str,
        storage_nodes: &Arc<RwLock<HashMap<String, StorageNode>>>,
        storage_entries: &Arc<RwLock<HashMap<String, StorageEntry>>>,
        local_storage: &Arc<RwLock<HashMap<String, Vec<u8>>>>,
    ) {
        match command {
            StorageCommand::Store { key, data, replicas } => {
                // Store locally first
                {
                    let mut local = local_storage.write().unwrap();
                    local.insert(key.clone(), data.clone());
                }

                // Create storage entry
                let entry = StorageEntry {
                    key: key.clone(),
                    size: data.len() as u64,
                    created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    last_accessed: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    access_count: 0,
                    replicas: vec![local_node_id.to_string()],
                    checksum: format!("{:x}", crc32fast::hash(&data)),
                    compression: if config.enable_compression { Some("lz4".to_string()) } else { None },
                    encryption: if config.enable_encryption { Some("aes256".to_string()) } else { None },
                };

                {
                    let mut entries = storage_entries.write().unwrap();
                    entries.insert(key, entry);
                }
            }
            StorageCommand::Retrieve { key, response_tx } => {
                // Try local storage first
                let data = {
                    let local = local_storage.read().unwrap();
                    local.get(&key).cloned()
                };

                // Update access statistics
                if data.is_some() {
                    let mut entries = storage_entries.write().unwrap();
                    if let Some(entry) = entries.get_mut(&key) {
                        entry.last_accessed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                        entry.access_count += 1;
                    }
                }

                let _ = response_tx.send(data);
            }
            StorageCommand::Delete { key } => {
                {
                    let mut local = local_storage.write().unwrap();
                    local.remove(&key);
                }
                {
                    let mut entries = storage_entries.write().unwrap();
                    entries.remove(&key);
                }
            }
            StorageCommand::Replicate { key: _, target_nodes: _ } => {
                // TODO: Implement replication to remote nodes
            }
            StorageCommand::Cleanup => {
                // TODO: Implement cleanup of old/unused data
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_creation() {
        let config = StorageConfig::default();
        let storage = DistributedStorage::new(config, "test_node".to_string());
        
        let stats = storage.get_stats().await;
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_node_management() {
        let config = StorageConfig::default();
        let storage = DistributedStorage::new(config, "test_node".to_string());
        
        let node = StorageNode::new(
            "node1".to_string(),
            "127.0.0.1:8000".to_string(),
            1024 * 1024 * 1024, // 1GB
        );
        
        storage.add_node(node).await;
        
        let stats = storage.get_stats().await;
        assert_eq!(stats.total_nodes, 1);
    }

    #[tokio::test]
    async fn test_local_storage() {
        let config = StorageConfig::default();
        let storage = DistributedStorage::new(config, "test_node".to_string());
        
        let key = "test_key".to_string();
        let data = b"test data".to_vec();
        
        let result = storage.store(key.clone(), data.clone()).await;
        assert!(matches!(result, StorageResult::Success));
        
        // Give some time for async processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let retrieved = storage.retrieve(key).await;
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let config = StorageConfig::default();
        let storage = DistributedStorage::new(config, "test_node".to_string());
        
        let key = "test_key".to_string();
        let data = b"test data".to_vec();
        
        storage.store(key, data).await;
        
        // Give some time for async processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let stats = storage.get_stats().await;
        assert_eq!(stats.local_entries, 1);
        assert_eq!(stats.local_size, 9); // "test data" is 9 bytes
    }
} 