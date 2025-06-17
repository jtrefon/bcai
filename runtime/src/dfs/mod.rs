//! Decentralized Filesystem with Economic Storage Network
//!
//! This module acts as a high-level facade for the Distributed File System (DFS).
//! It coordinates the other DFS modules (`types`, `contracts`, `crypto`, `permissions`).

// --- Module Declarations ---
pub mod contracts;
pub mod crypto;
pub mod permissions;
pub mod types;

// --- Imports ---
use crate::{
    distributed_storage::{DistributedStorage, StorageNode, StorageConfig},
    large_data_transfer::{LargeDataDescriptor, DataChunk, NetworkTransferCoordinator},
    smart_contracts::{SmartContractEngine, AIJobContract, ContractResult, ContractError},
    network::{NetworkMessage, NetworkCoordinator},
    node::{UnifiedNode, NodeCapability},
};
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock as AsyncRwLock};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

// --- Re-export key types for external usage ---
pub use types::*;
pub use contracts::StorageContract;
pub use permissions::PermissionGroup;

// --- Main DFS Struct ---

/// The main entry point and facade for the Decentralized Filesystem.
pub struct DecentralizedFilesystem {
    config: DfsConfig,
    local_node: Arc<AsyncRwLock<UnifiedNode>>,
    network_coordinator: Arc<AsyncRwLock<NetworkCoordinator>>,
    storage_coordinator: Arc<DistributedStorage>,
    transfer_coordinator: Option<Arc<NetworkTransferCoordinator>>,
    contract_engine: Arc<AsyncRwLock<SmartContractEngine>>,
    
    // Core data structures, now managed by their respective modules but cached here
    file_index: Arc<AsyncRwLock<HashMap<String, DfsFile>>>,
    storage_contracts: Arc<AsyncRwLock<HashMap<String, StorageContract>>>,
    storage_metrics: Arc<AsyncRwLock<HashMap<String, StorageNodeMetrics>>>,
    assembly_queue: Arc<AsyncRwLock<BTreeMap<AssemblyPriority, Vec<AssemblyRequest>>>>,
    
    // Statistics
    assembly_stats: Arc<AsyncRwLock<AssemblyStats>>,
    
    // Permission and encryption management
    permission_groups: Arc<AsyncRwLock<HashMap<String, PermissionGroup>>>,
    user_keys: Arc<AsyncRwLock<HashMap<String, UserKeyPair>>>,
    access_log: Arc<AsyncRwLock<Vec<AccessLogEntry>>>,
    
    // Event channels
    event_sender: mpsc::UnboundedSender<DfsEvent>,
}

impl DecentralizedFilesystem {
    pub fn new(
        config: DfsConfig,
        local_node: Arc<AsyncRwLock<UnifiedNode>>,
        network_coordinator: Arc<AsyncRwLock<NetworkCoordinator>>,
        storage_coordinator: Arc<DistributedStorage>,
        transfer_coordinator: Option<Arc<NetworkTransferCoordinator>>,
        contract_engine: Arc<AsyncRwLock<SmartContractEngine>>,
    ) -> Self {
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();
        
        Self {
            config,
            local_node,
            network_coordinator,
            storage_coordinator,
            transfer_coordinator,
            contract_engine,
            file_index: Arc::new(AsyncRwLock::new(HashMap::new())),
            storage_contracts: Arc::new(AsyncRwLock::new(HashMap::new())),
            storage_metrics: Arc::new(AsyncRwLock::new(HashMap::new())),
            assembly_queue: Arc::new(AsyncRwLock::new(BTreeMap::new())),
            assembly_stats: Arc::new(AsyncRwLock::new(Default::default())),
            permission_groups: Arc::new(AsyncRwLock::new(HashMap::new())),
            user_keys: Arc::new(AsyncRwLock::new(HashMap::new())),
            access_log: Arc::new(AsyncRwLock::new(Vec::new())),
            event_sender,
        }
    }

    // --- High-level API Methods ---
    
    /// Stores a file in the DFS with specified permissions.
    pub async fn store_file(
        &self,
        filename: String,
        data: Vec<u8>,
        content_type: String,
        owner: String,
        permissions: FilePermissions,
        storage_duration_hours: u64,
        tags: Vec<String>,
    ) -> Result<String, DfsError> {
        // 1. Encrypt the data based on permissions
        let (encrypted_data, encryption_metadata) =
            crypto::encrypt_file_data(&data, &permissions, &owner).await?;

        let file_hash = self.calculate_file_hash(&encrypted_data);

        // 2. Select storage nodes
        let replication = self.config.default_replication;
        let storage_nodes = self
            .select_storage_nodes(encrypted_data.len() as u64, replication)
            .await?;

        // 3. Create and distribute chunks
        let chunks = self
            .create_and_distribute_chunks(&encrypted_data, &file_hash, &storage_nodes)
            .await?;

        // 4. Create storage contract
        let contract_id = format!("contract-{}", file_hash);
        let escrow_amount = self.calculate_escrow(&encrypted_data, storage_duration_hours);
        
        // Ensure client has funds (simplified check)
        // In a real system, this would interact with the blockchain state
        
        let contract = self.create_storage_contract(
            contract_id.clone(),
            file_hash.clone(),
            storage_nodes,
            owner.clone(),
            escrow_amount,
            storage_duration_hours
        ).await?;

        self.storage_contracts.write().await.insert(contract_id.clone(), contract);

        // 5. Create and store file metadata
        let dfs_file = DfsFile {
            file_hash: file_hash.clone(),
            filename,
            size: encrypted_data.len() as u64,
            content_type,
            owner,
            storage_contracts: vec![contract_id],
            chunks,
            replication,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 0,
            tags,
            encryption_metadata,
        };

        self.file_index.write().await.insert(file_hash.clone(), dfs_file);

        Ok(file_hash)
    }
    
    /// Retrieves a file from the DFS, handling permissions and decryption.
    pub async fn retrieve_file(
        &self,
        file_hash: String,
        requester: String,
    ) -> Result<Vec<u8>, DfsError> {
        let file = self.file_index.read().await.get(&file_hash).cloned().ok_or_else(|| DfsError::FileNotFound(file_hash.clone()))?;

        // 1. Check permissions
        permissions::check_file_access(&file, &requester).await?;

        // 2. Assemble the encrypted file from chunks
        let encrypted_data = self.assemble_file_parallel(&file).await?;
        
        // 3. Get decryption key
        let key = permissions::get_decryption_key(&file.encryption_metadata, &requester).await?;
        
        // 4. Decrypt and return data
        crypto::decrypt_file_data_with_key(&encrypted_data, &file.encryption_metadata, &key).await
    }

    // --- Helper methods that coordinate other modules ---
    
    async fn select_storage_nodes(
        &self,
        data_size: u64,
        replication: u32,
    ) -> Result<Vec<String>, DfsError> {
        // Simplified logic: get all available nodes.
        // A real implementation would consider node metrics (space, reliability, etc.)
        let available_nodes = self.storage_coordinator.get_available_nodes(data_size).await;
        if available_nodes.len() < replication as usize {
            return Err(DfsError::InsufficientStorage);
        }
        Ok(available_nodes.into_iter().map(|n| n.id).take(replication as usize).collect())
    }

    async fn create_and_distribute_chunks(
        &self,
        data: &[u8],
        file_hash: &str,
        storage_nodes: &[String],
    ) -> Result<Vec<DfsChunk>, DfsError> {
        let chunks_to_create = self.create_chunks(data, file_hash).await?;
        self.distribute_chunks(&chunks_to_create, storage_nodes).await
    }
    
    async fn create_chunks(&self, data: &[u8], file_hash: &str) -> Result<Vec<DataChunk>, DfsError> {
        // Simplified chunking logic
        data.chunks(self.config.default_chunk_size_mb as usize * 1024 * 1024)
            .enumerate()
            .map(|(i, chunk_data)| {
                let chunk_hash = self.calculate_file_hash(chunk_data);
                Ok(DataChunk {
                    file_hash: file_hash.to_string(),
                    index: i as u32,
                    data: chunk_data.to_vec(),
                    hash: chunk_hash,
                })
            })
            .collect()
    }

    async fn distribute_chunks(
        &self,
        chunks: &[DataChunk],
        storage_nodes: &[String],
    ) -> Result<Vec<DfsChunk>, DfsError> {
        // Simplified distribution logic. In reality, this would use the transfer coordinator.
        let mut dfs_chunks = Vec::new();
        for chunk in chunks {
            // Here you would send the chunk to the storage nodes
            dfs_chunks.push(DfsChunk {
                index: chunk.index,
                hash: chunk.hash.clone(),
                size: chunk.data.len() as u32,
                storage_nodes: storage_nodes.to_vec(),
                verified_at: chrono::Utc::now(),
            });
        }
        Ok(dfs_chunks)
    }
    
    async fn assemble_file_parallel(&self, dfs_file: &DfsFile) -> Result<Vec<u8>, DfsError> {
        // Simplified assembly logic. In reality, this would request chunks
        // from storage nodes via the transfer coordinator and assemble them in order.
        let mut file_data = Vec::with_capacity(dfs_file.size as usize);
        
        // This is a placeholder. A real implementation needs to fetch chunk data.
        // For now, it cannot work as we don't store the actual data here.
        // This highlights the need for the transfer coordinator.
        Err(DfsError::AssemblyError("File assembly not fully implemented. Cannot fetch chunk data.".to_string()))
    }

    fn calculate_file_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    fn calculate_escrow(&self, data: &[u8], duration_hours: u64) -> u64 {
        let data_gb = data.len() as f64 / (1024.0 * 1024.0 * 1024.0);
        let duration_months = duration_hours as f64 / (24.0 * 30.0);
        let storage_cost = data_gb * duration_months * self.config.storage_price_per_gb_month as f64;
        storage_cost as u64
    }

    // --- Bridge to Contracts Module ---
    async fn create_storage_contract(
        &self,
        contract_id: String,
        file_hash: String,
        storage_nodes: Vec<String>,
        client: String,
        escrow_amount: u64,
        duration_hours: u64,
    ) -> Result<StorageContract, DfsError> {
        contracts::create_storage_contract(
            contract_id,
            file_hash,
            storage_nodes,
            client,
            escrow_amount,
            duration_hours,
        )
    }

    // --- Bridge to Permissions Module ---
    
    pub async fn create_permission_group(
        &self,
        group_id: String,
        name: String,
        description: String,
        owner: String,
        initial_members: Vec<String>,
        permissions: GroupPermissions,
    ) -> Result<(), DfsError> {
        let group = permissions::create_permission_group(group_id.clone(), name, description, owner, initial_members, permissions)?;
        self.permission_groups.write().await.insert(group_id, group);
        Ok(())
    }

    pub async fn grant_temporary_access(
         &self,
        file_hash: String,
        user_id: String,
        access_type: TemporaryAccessType,
        duration: Duration,
        requester: String,
        max_usage: Option<u64>,
    ) -> Result<(), DfsError> {
        let mut file_index = self.file_index.write().await;
        let file = file_index.get_mut(&file_hash).ok_or_else(|| DfsError::FileNotFound(file_hash.clone()))?;
        
        permissions::grant_temporary_access(file, user_id, access_type, duration, requester, max_usage).await
    }
} 