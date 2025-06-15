//! Decentralized Filesystem with Economic Storage Network
//!
//! This module implements a decentralized filesystem where:
//! - Data blocks are stored across distributed nodes
//! - Storage nodes earn BCAI tokens through escrow contracts
//! - Files can be assembled on-demand from distributed chunks
//! - Redundancy and parallel assembly ensure high availability
//! - Separate economic models for storage vs compute resources

use crate::{
    distributed_storage::{DistributedStorage, StorageNode, StorageConfig},
    large_data_transfer::{LargeDataDescriptor, DataChunk, NetworkTransferCoordinator},
    token::TokenLedger,
    smart_contracts::{SmartContractEngine, AIJobContract, ContractResult, ContractError},
    network::{NetworkMessage, NetworkCoordinator},
    node::{UnifiedNode, NodeCapability},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock as AsyncRwLock};
use thiserror::Error;
use chrono::{DateTime, Utc};
use rand;

// Add encryption and security imports
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::{Aead, OsRng, AeadCore}};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

/// Decentralized filesystem errors
#[derive(Debug, Error)]
pub enum DfsError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Insufficient storage nodes available")]
    InsufficientStorage,
    #[error("Escrow payment failed: {0}")]
    EscrowError(String),
    #[error("File assembly failed: {0}")]
    AssemblyError(String),
    #[error("Storage contract error: {0}")]
    ContractError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Insufficient funds: need {required}, have {available}")]
    InsufficientFunds { required: u64, available: u64 },
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Key management error: {0}")]
    KeyError(String),
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    #[error("User not found: {0}")]
    UserNotFound(String),
}

/// Configuration for decentralized filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsConfig {
    /// Default replication factor for stored files
    pub default_replication: u32,
    /// Default chunk size (MB)
    pub default_chunk_size_mb: u32,
    /// Storage pricing per GB per month (BCAI tokens)
    pub storage_price_per_gb_month: u64,
    /// Bandwidth pricing per GB (BCAI tokens)
    pub bandwidth_price_per_gb: u64,
    /// Minimum storage contract duration (hours)
    pub min_storage_duration_hours: u64,
    /// Maximum storage contract duration (hours)
    pub max_storage_duration_hours: u64,
    /// Parallel assembly workers
    pub parallel_assembly_workers: u32,
    /// Escrow release confirmation blocks
    pub escrow_confirmation_blocks: u64,
}

impl Default for DfsConfig {
    fn default() -> Self {
        Self {
            default_replication: 3,
            default_chunk_size_mb: 4,
            storage_price_per_gb_month: 10, // 10 BCAI per GB per month
            bandwidth_price_per_gb: 1, // 1 BCAI per GB transfer
            min_storage_duration_hours: 24, // 1 day minimum
            max_storage_duration_hours: 8760, // 1 year maximum
            parallel_assembly_workers: 8,
            escrow_confirmation_blocks: 6, // ~6 minutes for confirmation
        }
    }
}

/// File metadata in the distributed filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsFile {
    /// Unique file hash (content-addressed)
    pub file_hash: String,
    /// Original filename
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    pub content_type: String,
    /// File owner/uploader
    pub owner: String,
    /// Storage contract addresses
    pub storage_contracts: Vec<String>,
    /// Chunk descriptors
    pub chunks: Vec<DfsChunk>,
    /// Replication factor
    pub replication: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub last_accessed: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// File visibility (deprecated - use encryption_metadata.permissions)
    pub visibility: FileVisibility,
    /// Encryption and permission metadata
    pub encryption_metadata: EncryptionMetadata,
}

/// Chunk information within a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsChunk {
    /// Chunk index within file
    pub index: u32,
    /// Chunk hash
    pub hash: String,
    /// Chunk size in bytes
    pub size: u32,
    /// Storage nodes holding this chunk
    pub storage_nodes: Vec<String>,
    /// Verification timestamp
    pub verified_at: DateTime<Utc>,
}

/// File access permissions with encryption support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilePermissions {
    /// Public access - no encryption, anyone can read
    Public,
    /// Owner-only access - encrypted with owner's key
    OwnerOnly { 
        owner: String,
        encrypted_key: String, // AES key encrypted with owner's public key
    },
    /// Group access - encrypted with shared group key
    Group { 
        group_id: String,
        encrypted_key: String, // AES key encrypted with group's shared key
        members: Vec<String>,   // List of authorized group members
    },
    /// Custom access list - encrypted with individual keys per user
    Custom {
        access_list: HashMap<String, String>, // user_id -> encrypted_key
    },
}

/// Group management for file permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub members: Vec<String>,
    pub group_key: String, // Base64 encoded group key
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub permissions: GroupPermissions,
}

/// Group permission levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupPermissions {
    Read,      // Can only read files
    ReadWrite, // Can read and add files to group
    Admin,     // Can manage group membership
}

/// User key management for encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKeyPair {
    pub user_id: String,
    pub public_key: String,  // Base64 encoded public key
    pub private_key: String, // Base64 encoded private key (should be client-side only)
    pub created_at: DateTime<Utc>,
}

/// Encryption metadata stored with file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    pub is_encrypted: bool,
    pub encryption_algorithm: String, // "AES-256-GCM" for encrypted files
    pub nonce: Option<String>,        // Base64 encoded nonce for AES-GCM
    pub permissions: FilePermissions,
}

/// Access audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    pub file_hash: String,
    pub requester: String,
    pub access_time: DateTime<Utc>,
    pub access_granted: bool,
    pub reason: String, // Success reason or denial reason
}

/// File visibility levels (deprecated - use FilePermissions instead)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileVisibility {
    Public,
    Private,
    Shared(Vec<String>),
}

impl From<FileVisibility> for FilePermissions {
    fn from(visibility: FileVisibility) -> Self {
        match visibility {
            FileVisibility::Public => FilePermissions::Public,
            FileVisibility::Private => FilePermissions::OwnerOnly { 
                owner: "unknown".to_string(), 
                encrypted_key: String::new() 
            },
            FileVisibility::Shared(users) => {
                let mut access_list = HashMap::new();
                for user in users {
                    access_list.insert(user, String::new());
                }
                FilePermissions::Custom { access_list }
            }
        }
    }
}

/// Storage contract for economic incentives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageContract {
    /// Contract ID
    pub contract_id: String,
    /// File being stored
    pub file_hash: String,
    /// Storage nodes participating
    pub storage_nodes: Vec<String>,
    /// Client paying for storage
    pub client: String,
    /// Total escrow amount
    pub escrow_amount: u64,
    /// Payment per storage node
    pub payment_per_node: u64,
    /// Contract duration
    pub duration: Duration,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
    /// Contract status
    pub status: StorageContractStatus,
    /// Last verification time
    pub last_verified: DateTime<Utc>,
    /// Required availability percentage
    pub required_availability: f32,
}

/// Storage contract status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageContractStatus {
    Active,
    Expired,
    Completed,
    Breached,
    Cancelled,
}

/// Storage node performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNodeMetrics {
    /// Node ID
    pub node_id: String,
    /// Total storage provided (bytes)
    pub total_storage: u64,
    /// Available storage (bytes)
    pub available_storage: u64,
    /// Reliability score (0.0 to 1.0)
    pub reliability: f32,
    /// Average response time (ms)
    pub avg_response_time: u32,
    /// Bandwidth capacity (bytes/sec)
    pub bandwidth_capacity: u64,
    /// Total earnings (BCAI)
    pub total_earnings: u64,
    /// Active contracts count
    pub active_contracts: u32,
    /// Last heartbeat
    pub last_heartbeat: DateTime<Utc>,
    /// Geographic region (for optimization)
    pub region: String,
}

/// File assembly request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyRequest {
    /// File to assemble
    pub file_hash: String,
    /// Requesting node
    pub requester: String,
    /// Priority level
    pub priority: AssemblyPriority,
    /// Maximum acceptable assembly time
    pub max_assembly_time: Duration,
    /// Bandwidth limit for assembly
    pub bandwidth_limit: Option<u64>,
}

/// Assembly priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssemblyPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Assembly statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyStats {
    /// Total files assembled
    pub files_assembled: u64,
    /// Total bytes assembled
    pub bytes_assembled: u64,
    /// Average assembly time (seconds)
    pub avg_assembly_time: f64,
    /// Cache hit rate
    pub cache_hit_rate: f32,
    /// Parallel efficiency ratio
    pub parallel_efficiency: f32,
}

/// Main decentralized filesystem coordinator
pub struct DecentralizedFilesystem {
    config: DfsConfig,
    local_node: Arc<AsyncRwLock<UnifiedNode>>,
    network_coordinator: Arc<AsyncRwLock<NetworkCoordinator>>,
    storage_coordinator: Arc<DistributedStorage>,
    transfer_coordinator: Option<Arc<NetworkTransferCoordinator>>,
    token_ledger: Arc<AsyncRwLock<TokenLedger>>,
    contract_engine: Arc<AsyncRwLock<SmartContractEngine>>,
    
    // Core data structures
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

/// DFS events for monitoring and coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DfsEvent {
    FileStored { file_hash: String, size: u64, replication: u32 },
    FileRetrieved { file_hash: String, assembly_time: Duration },
    StorageContractCreated { contract_id: String, duration: Duration },
    StorageContractCompleted { contract_id: String, payment: u64 },
    ChunkVerified { chunk_hash: String, node_id: String },
    EscrowReleased { contract_id: String, amount: u64 },
}

impl DecentralizedFilesystem {
    /// Create new decentralized filesystem
    pub fn new(
        config: DfsConfig,
        local_node: Arc<AsyncRwLock<UnifiedNode>>,
        network_coordinator: Arc<AsyncRwLock<NetworkCoordinator>>,
        storage_coordinator: Arc<DistributedStorage>,
        transfer_coordinator: Option<Arc<NetworkTransferCoordinator>>,
        token_ledger: Arc<AsyncRwLock<TokenLedger>>,
        contract_engine: Arc<AsyncRwLock<SmartContractEngine>>,
    ) -> Self {
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            local_node,
            network_coordinator,
            storage_coordinator,
            transfer_coordinator,
            token_ledger,
            contract_engine,
            file_index: Arc::new(AsyncRwLock::new(HashMap::new())),
            storage_contracts: Arc::new(AsyncRwLock::new(HashMap::new())),
            storage_metrics: Arc::new(AsyncRwLock::new(HashMap::new())),
            assembly_queue: Arc::new(AsyncRwLock::new(BTreeMap::new())),
            assembly_stats: Arc::new(AsyncRwLock::new(AssemblyStats {
                files_assembled: 0,
                bytes_assembled: 0,
                avg_assembly_time: 0.0,
                cache_hit_rate: 0.0,
                parallel_efficiency: 0.0,
            })),
            permission_groups: Arc::new(AsyncRwLock::new(HashMap::new())),
            user_keys: Arc::new(AsyncRwLock::new(HashMap::new())),
            access_log: Arc::new(AsyncRwLock::new(Vec::new())),
            event_sender,
        }
    }

    /// Store a file in the distributed filesystem with economic contracts
    pub async fn store_file(
        &self,
        filename: String,
        data: Vec<u8>,
        content_type: String,
        owner: String,
        visibility: FileVisibility,
        storage_duration_hours: u64,
        tags: Vec<String>,
    ) -> Result<String, DfsError> {
        println!("📁 Storing file: {} ({} bytes)", filename, data.len());

        // 1. Calculate file hash and chunk data
        let file_hash = self.calculate_file_hash(&data);
        let chunks = self.create_chunks(&data, &file_hash).await?;
        
        // 2. Calculate storage costs
        let file_size_gb = data.len() as f64 / (1024.0 * 1024.0 * 1024.0);
        let storage_cost = (file_size_gb * self.config.storage_price_per_gb_month as f64 
                           * storage_duration_hours as f64 / (24.0 * 30.0)) as u64;
        let total_cost = storage_cost * self.config.default_replication as u64;

        println!("💰 Storage cost: {} BCAI for {:.3} GB, {} hours", 
                total_cost, file_size_gb, storage_duration_hours);

        // 3. Check if owner has sufficient funds
        let owner_balance = {
            let ledger = self.token_ledger.read().await;
            ledger.balance(&owner)
        };

        if owner_balance < total_cost {
            return Err(DfsError::InsufficientFunds {
                required: total_cost,
                available: owner_balance,
            });
        }

        // 4. Select storage nodes based on reliability and capacity
        let storage_nodes = self.select_storage_nodes(
            data.len() as u64,
            self.config.default_replication,
        ).await?;

        // 5. Create storage contract with escrow
        let contract_id = format!("storage_{}_{}", file_hash, Utc::now().timestamp());
        let storage_contract = self.create_storage_contract(
            contract_id.clone(),
            file_hash.clone(),
            storage_nodes.clone(),
            owner.clone(),
            total_cost,
            storage_duration_hours,
        ).await?;

        // 6. Store chunks on selected nodes
        let dfs_chunks = self.distribute_chunks(&chunks, &storage_nodes).await?;

        // 7. Create file metadata
        let dfs_file = DfsFile {
            file_hash: file_hash.clone(),
            filename,
            size: data.len() as u64,
            content_type,
            owner,
            storage_contracts: vec![contract_id.clone()],
            chunks: dfs_chunks,
            replication: self.config.default_replication,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            tags,
            visibility: visibility.clone(),
            encryption_metadata: EncryptionMetadata {
                is_encrypted: false, // Will be updated based on permissions
                encryption_algorithm: "none".to_string(),
                nonce: None,
                permissions: visibility.into(),
            },
        };

        // 8. Register file in index
        {
            let mut file_index = self.file_index.write().await;
            file_index.insert(file_hash.clone(), dfs_file);
        }

        // 9. Emit event
        let _ = self.event_sender.send(DfsEvent::FileStored {
            file_hash: file_hash.clone(),
            size: data.len() as u64,
            replication: self.config.default_replication,
        });

        println!("✅ File stored successfully: {}", file_hash);
        Ok(file_hash)
    }

    /// Retrieve and assemble a file from distributed storage
    pub async fn retrieve_file(
        &self,
        file_hash: String,
        requester: String,
    ) -> Result<Vec<u8>, DfsError> {
        let assembly_start = SystemTime::now();
        println!("📥 Retrieving file: {}", file_hash);

        // 1. Get file metadata
        let dfs_file = {
            let file_index = self.file_index.read().await;
            file_index.get(&file_hash).cloned()
                .ok_or_else(|| DfsError::FileNotFound(file_hash.clone()))?
        };

        // 2. Check access permissions
        if !self.check_file_access(&dfs_file, &requester) {
            return Err(DfsError::FileNotFound(file_hash)); // Don't reveal file exists
        }

        // 3. Calculate bandwidth costs
        let bandwidth_cost = (dfs_file.size as f64 / (1024.0 * 1024.0 * 1024.0) 
                             * self.config.bandwidth_price_per_gb as f64) as u64;

        // 4. Charge bandwidth costs
        {
            let mut ledger = self.token_ledger.write().await;
            if ledger.balance(&requester) < bandwidth_cost {
                return Err(DfsError::InsufficientFunds {
                    required: bandwidth_cost,
                    available: ledger.balance(&requester),
                });
            }
            
            // Deduct bandwidth cost from requester
            if bandwidth_cost > 0 {
                ledger.transfer(&requester, "network_treasury", bandwidth_cost)
                    .map_err(|e| DfsError::EscrowError(format!("Bandwidth payment failed: {}", e)))?;
            }
        }

        // 5. Parallel chunk assembly
        let assembled_data = self.assemble_file_parallel(&dfs_file).await?;

        // 6. Update access statistics
        {
            let mut file_index = self.file_index.write().await;
            if let Some(file) = file_index.get_mut(&file_hash) {
                file.last_accessed = Utc::now();
                file.access_count += 1;
            }
        }

        // 7. Update assembly statistics
        let assembly_duration = assembly_start.elapsed()
            .unwrap_or(Duration::from_secs(0));
        
        {
            let mut stats = self.assembly_stats.write().await;
            stats.files_assembled += 1;
            stats.bytes_assembled += dfs_file.size;
            stats.avg_assembly_time = (stats.avg_assembly_time * (stats.files_assembled - 1) as f64
                                     + assembly_duration.as_secs_f64()) / stats.files_assembled as f64;
        }

        // 8. Emit event
        let _ = self.event_sender.send(DfsEvent::FileRetrieved {
            file_hash,
            assembly_time: assembly_duration,
        });

        println!("✅ File retrieved in {:.2}s ({} bytes)", 
                assembly_duration.as_secs_f64(), assembled_data.len());
        
        Ok(assembled_data)
    }

    /// Select optimal storage nodes based on capacity, reliability, and cost
    async fn select_storage_nodes(
        &self,
        data_size: u64,
        replication: u32,
    ) -> Result<Vec<String>, DfsError> {
        let storage_metrics = self.storage_metrics.read().await;
        
        // Filter nodes with sufficient capacity and good reliability
        let mut candidates: Vec<_> = storage_metrics
            .values()
            .filter(|node| {
                node.available_storage >= data_size &&
                node.reliability >= 0.8 && // Minimum 80% reliability
                node.last_heartbeat > Utc::now() - chrono::Duration::minutes(5)
            })
            .collect();

        if candidates.len() < replication as usize {
            return Err(DfsError::InsufficientStorage);
        }

        // Sort by composite score: reliability × available_storage × (1 / response_time)
        candidates.sort_by(|a, b| {
            let score_a = a.reliability 
                        * (a.available_storage as f32 / (1024.0 * 1024.0 * 1024.0)) // GB
                        * (1000.0 / a.avg_response_time.max(1) as f32); // Invert response time
            let score_b = b.reliability 
                        * (b.available_storage as f32 / (1024.0 * 1024.0 * 1024.0))
                        * (1000.0 / b.avg_response_time.max(1) as f32);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select top nodes
        let selected: Vec<String> = candidates
            .into_iter()
            .take(replication as usize)
            .map(|node| node.node_id.clone())
            .collect();

        println!("🎯 Selected {} storage nodes: {:?}", selected.len(), selected);
        Ok(selected)
    }

    /// Create storage contract with escrow mechanism
    async fn create_storage_contract(
        &self,
        contract_id: String,
        file_hash: String,
        storage_nodes: Vec<String>,
        client: String,
        escrow_amount: u64,
        duration_hours: u64,
    ) -> Result<StorageContract, DfsError> {
        // 1. Create escrow in token ledger
        {
            let mut ledger = self.token_ledger.write().await;
            let escrow_account = format!("escrow_{}", contract_id);
            
            // Transfer funds to escrow
            ledger.transfer(&client, &escrow_account, escrow_amount)
                .map_err(|e| DfsError::EscrowError(format!("Escrow creation failed: {}", e)))?;
        }

        // 2. Calculate payment per node
        let payment_per_node = escrow_amount / storage_nodes.len() as u64;

        // 3. Create contract
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::hours(duration_hours as i64);

        let contract = StorageContract {
            contract_id: contract_id.clone(),
            file_hash,
            storage_nodes,
            client,
            escrow_amount,
            payment_per_node,
            duration: Duration::from_secs(duration_hours * 3600),
            start_time,
            end_time,
            status: StorageContractStatus::Active,
            last_verified: start_time,
            required_availability: 0.95, // 95% availability required
        };

        // 4. Store contract
        {
            let mut contracts = self.storage_contracts.write().await;
            contracts.insert(contract_id.clone(), contract.clone());
        }

        // 5. Emit event
        let _ = self.event_sender.send(DfsEvent::StorageContractCreated {
            contract_id,
            duration: Duration::from_secs(duration_hours * 3600),
        });

        Ok(contract)
    }

    /// Parallel file assembly for high throughput
    async fn assemble_file_parallel(&self, dfs_file: &DfsFile) -> Result<Vec<u8>, DfsError> {
        let chunk_count = dfs_file.chunks.len();
        println!("🔧 Assembling {} chunks in parallel...", chunk_count);

        // Create semaphore for parallel workers
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.config.parallel_assembly_workers as usize
        ));

        // Prepare chunk assembly tasks
        let mut chunk_tasks = Vec::new();
        for chunk in &dfs_file.chunks {
            let chunk_clone = chunk.clone();
            let storage_coordinator = Arc::clone(&self.storage_coordinator);
            let semaphore_clone = Arc::clone(&semaphore);

            let task = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                // Try to retrieve chunk from available nodes
                for node_id in &chunk_clone.storage_nodes {
                    if let Some(data) = storage_coordinator.retrieve(chunk_clone.hash.clone()).await {
                        return Ok((chunk_clone.index, data));
                    }
                }
                
                Err(DfsError::AssemblyError(
                    format!("Chunk {} not available from any storage node", chunk_clone.index)
                ))
            });

            chunk_tasks.push(task);
        }

        // Await all chunks
        let mut chunk_results = Vec::new();
        for task in chunk_tasks {
            chunk_results.push(task.await.map_err(|e| {
                DfsError::AssemblyError(format!("Chunk assembly task failed: {}", e))
            })??);
        }

        // Sort chunks by index
        chunk_results.sort_by_key(|(index, _)| *index);

        // Concatenate chunk data
        let mut assembled_data = Vec::with_capacity(dfs_file.size as usize);
        for (_, chunk_data) in chunk_results {
            assembled_data.extend_from_slice(&chunk_data);
        }

        println!("✅ File assembled: {} bytes from {} chunks", 
                assembled_data.len(), chunk_count);

        Ok(assembled_data)
    }

    /// Check if user has access to file
    fn check_file_access(&self, file: &DfsFile, requester: &str) -> bool {
        match &file.visibility {
            FileVisibility::Public => true,
            FileVisibility::Private => file.owner == requester,
            FileVisibility::Shared(allowed_users) => {
                file.owner == requester || allowed_users.contains(&requester.to_string())
            }
        }
    }

    /// Calculate file hash using SHA-256
    fn calculate_file_hash(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Create data chunks for storage
    async fn create_chunks(&self, data: &[u8], file_hash: &str) -> Result<Vec<DataChunk>, DfsError> {
        let chunk_size = (self.config.default_chunk_size_mb * 1024 * 1024) as usize;
        let mut chunks = Vec::new();
        
        for (index, chunk_data) in data.chunks(chunk_size).enumerate() {
            let chunk = DataChunk::new(
                chunk_data.to_vec(),
                index as u32,
                crate::large_data_transfer::CompressionAlgorithm::Lz4,
            ).map_err(|e| DfsError::AssemblyError(format!("Chunk creation failed: {:?}", e)))?;
            
            chunks.push(chunk);
        }

        println!("📦 Created {} chunks of ~{}MB each", chunks.len(), self.config.default_chunk_size_mb);
        Ok(chunks)
    }

    /// Distribute chunks to storage nodes
    async fn distribute_chunks(
        &self,
        chunks: &[DataChunk],
        storage_nodes: &[String],
    ) -> Result<Vec<DfsChunk>, DfsError> {
        let mut dfs_chunks = Vec::new();

        for chunk in chunks {
            // For each chunk, store it on all replication nodes
            let mut chunk_storage_nodes = Vec::new();
            
            for node_id in storage_nodes {
                // Store chunk on node (simplified - in production would use network transfer)
                let result = self.storage_coordinator.store(
                    chunk.id().as_str().to_string(),
                    chunk.data.clone(),
                ).await;
                
                match result {
                    crate::distributed_storage::StorageResult::Success => {
                        chunk_storage_nodes.push(node_id.clone());
                    }
                    _ => {
                        println!("⚠️  Failed to store chunk {} on node {}", chunk.id().as_str(), node_id);
                    }
                }
            }

            let dfs_chunk = DfsChunk {
                index: chunk.info.index,
                hash: chunk.id().as_str().to_string(),
                size: chunk.info.original_size,
                storage_nodes: chunk_storage_nodes,
                verified_at: Utc::now(),
            };

            dfs_chunks.push(dfs_chunk);
        }

        Ok(dfs_chunks)
    }

    /// Process storage contract completions and release escrow
    pub async fn process_storage_contracts(&self) -> Result<(), DfsError> {
        let mut completed_contracts = Vec::new();
        
        // Find completed contracts
        {
            let contracts = self.storage_contracts.read().await;
            for contract in contracts.values() {
                if contract.status == StorageContractStatus::Active 
                   && Utc::now() >= contract.end_time {
                    completed_contracts.push(contract.clone());
                }
            }
        }

        // Process each completed contract
        for contract in completed_contracts {
            self.complete_storage_contract(contract).await?;
        }

        Ok(())
    }

    /// Complete storage contract and distribute payments
    async fn complete_storage_contract(&self, contract: StorageContract) -> Result<(), DfsError> {
        println!("💰 Completing storage contract: {}", contract.contract_id);

        // 1. Verify storage nodes still have the data
        let verified_nodes = self.verify_storage_availability(&contract).await;
        let availability_ratio = verified_nodes.len() as f32 / contract.storage_nodes.len() as f32;

        // 2. Calculate payments based on availability
        let mut total_payment = 0u64;
        
        if availability_ratio >= contract.required_availability {
            // Full payment for meeting requirements
            for node_id in &verified_nodes {
                let payment = contract.payment_per_node;
                
                // Transfer from escrow to storage node
                {
                    let mut ledger = self.token_ledger.write().await;
                    let escrow_account = format!("escrow_{}", contract.contract_id);
                    
                    if let Err(e) = ledger.transfer(&escrow_account, node_id, payment) {
                        println!("⚠️  Payment failed for node {}: {}", node_id, e);
                    } else {
                        total_payment += payment;
                        println!("💸 Paid {} BCAI to storage node {}", payment, node_id);
                    }
                }
            }
        } else {
            // Partial payment for poor availability
            println!("⚠️  Storage availability {}% below required {}%", 
                    availability_ratio * 100.0, contract.required_availability * 100.0);
            
            let penalty_factor = (availability_ratio / contract.required_availability).min(1.0);
            for node_id in &verified_nodes {
                let reduced_payment = (contract.payment_per_node as f32 * penalty_factor) as u64;
                
                {
                    let mut ledger = self.token_ledger.write().await;
                    let escrow_account = format!("escrow_{}", contract.contract_id);
                    
                    if let Err(e) = ledger.transfer(&escrow_account, node_id, reduced_payment) {
                        println!("⚠️  Reduced payment failed for node {}: {}", node_id, e);
                    } else {
                        total_payment += reduced_payment;
                    }
                }
            }
        }

        // 3. Return remaining escrow to client if any
        let remaining_escrow = contract.escrow_amount - total_payment;
        if remaining_escrow > 0 {
            let mut ledger = self.token_ledger.write().await;
            let escrow_account = format!("escrow_{}", contract.contract_id);
            let _ = ledger.transfer(&escrow_account, &contract.client, remaining_escrow);
        }

        // 4. Update contract status
        {
            let mut contracts = self.storage_contracts.write().await;
            if let Some(contract_mut) = contracts.get_mut(&contract.contract_id) {
                contract_mut.status = StorageContractStatus::Completed;
            }
        }

        // 5. Emit event
        let _ = self.event_sender.send(DfsEvent::StorageContractCompleted {
            contract_id: contract.contract_id,
            payment: total_payment,
        });

        println!("✅ Storage contract completed. Total payment: {} BCAI", total_payment);
        Ok(())
    }

    /// Verify storage availability for contract completion
    async fn verify_storage_availability(&self, contract: &StorageContract) -> Vec<String> {
        let mut verified_nodes = Vec::new();

        // Get file chunks to verify
        let chunks = {
            let file_index = self.file_index.read().await;
            if let Some(file) = file_index.get(&contract.file_hash) {
                file.chunks.clone()
            } else {
                return verified_nodes;
            }
        };

        // Check each storage node
        for node_id in &contract.storage_nodes {
            let mut node_verified = true;
            
            // Verify node has all required chunks
            for chunk in &chunks {
                if chunk.storage_nodes.contains(node_id) {
                    // Try to retrieve chunk
                    if self.storage_coordinator.retrieve(chunk.hash.clone()).await.is_none() {
                        node_verified = false;
                        break;
                    }
                }
            }
            
            if node_verified {
                verified_nodes.push(node_id.clone());
            }
        }

        verified_nodes
    }

    /// Add storage node metrics
    pub async fn add_storage_node_metrics(&self, metrics: StorageNodeMetrics) {
        let mut storage_metrics = self.storage_metrics.write().await;
        storage_metrics.insert(metrics.node_id.clone(), metrics);
    }

    /// Get storage metrics (public access)
    pub async fn get_storage_metrics(&self) -> HashMap<String, StorageNodeMetrics> {
        let storage_metrics = self.storage_metrics.read().await;
        storage_metrics.clone()
    }

    /// Update storage node earnings and performance metrics
    pub async fn update_node_earnings(&self, node_id: &str, earnings: u64, contracts_completed: u32) {
        let mut storage_metrics = self.storage_metrics.write().await;
        if let Some(metrics) = storage_metrics.get_mut(node_id) {
            metrics.total_earnings += earnings;
            metrics.active_contracts = metrics.active_contracts.saturating_sub(contracts_completed);
            metrics.last_heartbeat = Utc::now();
            
            // Update reliability based on successful contract completion
            if contracts_completed > 0 {
                metrics.reliability = (metrics.reliability * 0.9 + 0.98 * 0.1).min(1.0);
            }
        }
    }

    /// Force complete storage contracts (for demos and testing)
    pub async fn force_complete_storage_contracts(&self) -> Result<u64, DfsError> {
        let all_contracts = {
            let contracts = self.storage_contracts.read().await;
            contracts.values()
                .filter(|c| c.status == StorageContractStatus::Active)
                .cloned()
                .collect::<Vec<_>>()
        };

        let mut total_distributed = 0u64;
        for contract in all_contracts {
            // Simulate contract completion
            let payment = self.simulate_contract_completion(contract).await?;
            total_distributed += payment;
        }

        Ok(total_distributed)
    }

    /// Simulate contract completion with performance-based rewards
    async fn simulate_contract_completion(&self, contract: StorageContract) -> Result<u64, DfsError> {
        println!("💰 Force completing storage contract: {}", contract.contract_id);

        // Simulate storage verification (assume 95% success rate)
        let verified_nodes: Vec<String> = contract.storage_nodes.iter()
            .filter(|_| rand::random::<f32>() > 0.05) // 95% success rate
            .cloned()
            .collect();
        
        let availability_ratio = verified_nodes.len() as f32 / contract.storage_nodes.len() as f32;
        let mut total_payment = 0u64;

        // Calculate performance bonuses
        for node_id in &verified_nodes {
            let base_payment = contract.payment_per_node;
            
            // Get node performance metrics for bonus calculation
            let performance_bonus = {
                let metrics = self.storage_metrics.read().await;
                if let Some(node_metrics) = metrics.get(node_id) {
                    // Bonus based on reliability and response time
                    let reliability_bonus = (node_metrics.reliability - 0.9).max(0.0) * base_payment as f32;
                    let speed_bonus = if node_metrics.avg_response_time < 100 { 
                        base_payment / 10 // 10% bonus for fast response
                    } else { 0 };
                    (reliability_bonus as u64) + speed_bonus
                } else {
                    0
                }
            };

            let total_node_payment = base_payment + performance_bonus;

            // Transfer payment
            {
                let mut ledger = self.token_ledger.write().await;
                let escrow_account = format!("escrow_{}", contract.contract_id);
                
                if let Err(e) = ledger.transfer(&escrow_account, node_id, total_node_payment) {
                    println!("⚠️  Payment failed for node {}: {}", node_id, e);
                } else {
                    total_payment += total_node_payment;
                    println!("💸 Paid {} BCAI to {} (base: {}, bonus: {})", 
                            total_node_payment, node_id, base_payment, performance_bonus);
                    
                    // Update node earnings
                    self.update_node_earnings(node_id, total_node_payment, 1).await;
                }
            }
        }

        // Handle penalties for failed nodes
        let failed_nodes: Vec<String> = contract.storage_nodes.iter()
            .filter(|node_id| !verified_nodes.contains(node_id))
            .cloned()
            .collect();

        for node_id in &failed_nodes {
            println!("❌ Node {} failed availability check - no payment", node_id);
            
            // Apply reliability penalty
            let mut storage_metrics = self.storage_metrics.write().await;
            if let Some(metrics) = storage_metrics.get_mut(node_id) {
                metrics.reliability = (metrics.reliability * 0.8).max(0.1); // Significant penalty
            }
        }

        // Return remaining escrow to client
        let remaining_escrow = contract.escrow_amount - total_payment;
        if remaining_escrow > 0 {
            let mut ledger = self.token_ledger.write().await;
            let escrow_account = format!("escrow_{}", contract.contract_id);
            let _ = ledger.transfer(&escrow_account, &contract.client, remaining_escrow);
            println!("💰 Returned {} BCAI to client {}", remaining_escrow, contract.client);
        }

        // Update contract status
        {
            let mut contracts = self.storage_contracts.write().await;
            if let Some(contract_mut) = contracts.get_mut(&contract.contract_id) {
                contract_mut.status = StorageContractStatus::Completed;
            }
        }

        // Emit completion event
        let _ = self.event_sender.send(DfsEvent::StorageContractCompleted {
            contract_id: contract.contract_id,
            payment: total_payment,
        });

        println!("✅ Contract completed: {} BCAI distributed, {:.1}% availability", 
                total_payment, availability_ratio * 100.0);
        
        Ok(total_payment)
    }

    /// Get detailed node earnings report
    pub async fn get_node_earnings_report(&self) -> Vec<NodeEarningsReport> {
        let storage_metrics = self.storage_metrics.read().await;
        let mut reports = Vec::new();

        for (node_id, metrics) in storage_metrics.iter() {
            let report = NodeEarningsReport {
                node_id: node_id.clone(),
                total_earnings: metrics.total_earnings,
                contracts_completed: metrics.active_contracts,
                reliability_score: metrics.reliability,
                avg_response_time: metrics.avg_response_time,
                storage_capacity_gb: metrics.total_storage as f64 / (1024.0 * 1024.0 * 1024.0),
                earnings_per_gb: if metrics.total_storage > 0 {
                    metrics.total_earnings as f64 / (metrics.total_storage as f64 / (1024.0 * 1024.0 * 1024.0))
                } else {
                    0.0
                },
                performance_tier: if metrics.reliability > 0.98 && metrics.avg_response_time < 50 {
                    "Premium".to_string()
                } else if metrics.reliability > 0.95 && metrics.avg_response_time < 100 {
                    "Standard".to_string()
                } else {
                    "Basic".to_string()
                },
            };
            reports.push(report);
        }

        // Sort by total earnings
        reports.sort_by(|a, b| b.total_earnings.cmp(&a.total_earnings));
        reports
    }

    /// Calculate network-wide rewards distribution statistics
    pub async fn get_rewards_distribution_stats(&self) -> RewardsDistributionStats {
        let storage_metrics = self.storage_metrics.read().await;
        let contracts = self.storage_contracts.read().await;
        
        let total_earnings: u64 = storage_metrics.values().map(|m| m.total_earnings).sum();
        let active_storage_providers = storage_metrics.len();
        let completed_contracts = contracts.values()
            .filter(|c| c.status == StorageContractStatus::Completed)
            .count();
        let total_storage_gb: f64 = storage_metrics.values()
            .map(|m| m.total_storage as f64 / (1024.0 * 1024.0 * 1024.0))
            .sum();
        
        let avg_reliability: f32 = if storage_metrics.is_empty() {
            0.0
        } else {
            storage_metrics.values().map(|m| m.reliability).sum::<f32>() / storage_metrics.len() as f32
        };

        // Calculate earnings distribution tiers
        let mut premium_earnings = 0u64;
        let mut standard_earnings = 0u64;
        let mut basic_earnings = 0u64;
        
        for metrics in storage_metrics.values() {
            if metrics.reliability > 0.98 && metrics.avg_response_time < 50 {
                premium_earnings += metrics.total_earnings;
            } else if metrics.reliability > 0.95 && metrics.avg_response_time < 100 {
                standard_earnings += metrics.total_earnings;
            } else {
                basic_earnings += metrics.total_earnings;
            }
        }

        RewardsDistributionStats {
            total_earnings_distributed: total_earnings,
            active_storage_providers,
            completed_contracts,
            total_storage_capacity_gb: total_storage_gb,
            avg_earnings_per_provider: if active_storage_providers > 0 {
                total_earnings / active_storage_providers as u64
            } else {
                0
            },
            avg_reliability_score: avg_reliability,
            premium_tier_earnings: premium_earnings,
            standard_tier_earnings: standard_earnings,
            basic_tier_earnings: basic_earnings,
        }
    }

    /// ============ ENHANCED PERMISSION & ENCRYPTION SYSTEM ============

    /// Create a new permission group for file access control
    pub async fn create_permission_group(
        &self,
        group_id: String,
        name: String,
        description: String,
        owner: String,
        initial_members: Vec<String>,
        permissions: GroupPermissions,
    ) -> Result<(), DfsError> {
        // Generate a random group key for encryption
        let group_key = self.generate_group_key()?;
        
        let group = PermissionGroup {
            group_id: group_id.clone(),
            name,
            description,
            owner,
            members: initial_members,
            group_key,
            created_at: Utc::now(),
            last_modified: Utc::now(),
            permissions,
        };

        {
            let mut groups = self.permission_groups.write().await;
            groups.insert(group_id.clone(), group);
        }

        println!("🔐 Created permission group: {}", group_id);
        Ok(())
    }

    /// Generate a symmetric encryption key
    fn generate_symmetric_key(&self) -> Result<String, DfsError> {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        Ok(general_purpose::STANDARD.encode(&key))
    }

    /// Generate a group key
    fn generate_group_key(&self) -> Result<String, DfsError> {
        self.generate_symmetric_key()
    }

    /// Store file with enhanced permissions (NEW METHOD)
    pub async fn store_file_with_permissions(
        &self,
        filename: String,
        data: Vec<u8>,
        content_type: String,
        owner: String,
        permissions: FilePermissions,
        storage_duration_hours: u64,
        tags: Vec<String>,
    ) -> Result<String, DfsError> {
        println!("🔐 Storing file with enhanced permissions: {} ({} bytes)", filename, data.len());

        // 1. Encrypt data based on permissions
        let (encrypted_data, encryption_metadata) = self.encrypt_file_data(&data, &permissions, &owner).await?;

        // 2. Use encrypted data for storage (rest of process same as before)
        let file_hash = self.calculate_file_hash(&encrypted_data);
        let chunks = self.create_chunks(&encrypted_data, &file_hash).await?;
        
        // 3. Calculate storage costs
        let file_size_gb = encrypted_data.len() as f64 / (1024.0 * 1024.0 * 1024.0);
        let storage_cost = (file_size_gb * self.config.storage_price_per_gb_month as f64 
                           * storage_duration_hours as f64 / (24.0 * 30.0)) as u64;
        let total_cost = storage_cost * self.config.default_replication as u64;

        println!("💰 Storage cost: {} BCAI for {:.3} GB, {} hours", 
                total_cost, file_size_gb, storage_duration_hours);

        // 4. Check if owner has sufficient funds
        let owner_balance = {
            let ledger = self.token_ledger.read().await;
            ledger.balance(&owner)
        };

        if owner_balance < total_cost {
            return Err(DfsError::InsufficientFunds {
                required: total_cost,
                available: owner_balance,
            });
        }

        // 5. Select storage nodes
        let storage_nodes = self.select_storage_nodes(
            encrypted_data.len() as u64,
            self.config.default_replication,
        ).await?;

        // 6. Create storage contract
        let contract_id = format!("storage_{}_{}", file_hash, Utc::now().timestamp());
        let _storage_contract = self.create_storage_contract(
            contract_id.clone(),
            file_hash.clone(),
            storage_nodes.clone(),
            owner.clone(),
            total_cost,
            storage_duration_hours,
        ).await?;

        // 7. Store chunks
        let dfs_chunks = self.distribute_chunks(&chunks, &storage_nodes).await?;

        // 8. Create file metadata with encryption info
        let dfs_file = DfsFile {
            file_hash: file_hash.clone(),
            filename,
            size: data.len() as u64, // Original size, not encrypted size
            content_type,
            owner,
            storage_contracts: vec![contract_id.clone()],
            chunks: dfs_chunks,
            replication: self.config.default_replication,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            tags,
            visibility: FileVisibility::Private, // Deprecated field
            encryption_metadata,
        };

        // 9. Register file
        {
            let mut file_index = self.file_index.write().await;
            file_index.insert(file_hash.clone(), dfs_file);
        }

        println!("✅ File stored with encryption: {}", file_hash);
        
        Ok(file_hash)
    }

    /// Get filesystem statistics
    pub async fn get_statistics(&self) -> DfsStatistics {
        let file_index = self.file_index.read().await;
        let contracts = self.storage_contracts.read().await;
        let metrics = self.storage_metrics.read().await;
        let assembly_stats = self.assembly_stats.read().await;

        let total_files = file_index.len();
        let total_storage_bytes: u64 = file_index.values().map(|f| f.size).sum();
        let active_contracts = contracts.values()
            .filter(|c| c.status == StorageContractStatus::Active)
            .count();
        let storage_nodes = metrics.len();

        DfsStatistics {
            total_files,
            total_storage_bytes,
            active_contracts,
            storage_nodes,
            assembly_stats: assembly_stats.clone(),
            cache_hit_rate: assembly_stats.cache_hit_rate,
            avg_replication: if total_files > 0 {
                file_index.values().map(|f| f.replication).sum::<u32>() as f32 / total_files as f32
            } else {
                0.0
            },
        }
    }

    /// Encrypt file data based on permissions
    async fn encrypt_file_data(
        &self, 
        data: &[u8], 
        permissions: &FilePermissions, 
        _owner: &str
    ) -> Result<(Vec<u8>, EncryptionMetadata), DfsError> {
        match permissions {
            FilePermissions::Public => {
                // No encryption for public files
                Ok((data.to_vec(), EncryptionMetadata {
                    is_encrypted: false,
                    encryption_algorithm: "none".to_string(),
                    nonce: None,
                    permissions: permissions.clone(),
                }))
            },
            
            FilePermissions::OwnerOnly { owner: perm_owner, .. } => {
                // Encrypt with owner's key
                let key = self.generate_symmetric_key()?;
                let (encrypted_data, nonce) = self.encrypt_with_symmetric_key(data, &key)?;
                
                Ok((encrypted_data, EncryptionMetadata {
                    is_encrypted: true,
                    encryption_algorithm: "AES-256-GCM".to_string(),
                    nonce: Some(general_purpose::STANDARD.encode(&nonce)),
                    permissions: FilePermissions::OwnerOnly {
                        owner: perm_owner.clone(),
                        encrypted_key: key, // Store the key for demo
                    },
                }))
            },
            
            _ => {
                // For demo, treat other permissions as encrypted with a simple key
                let key = self.generate_symmetric_key()?;
                let (encrypted_data, nonce) = self.encrypt_with_symmetric_key(data, &key)?;
                
                Ok((encrypted_data, EncryptionMetadata {
                    is_encrypted: true,
                    encryption_algorithm: "AES-256-GCM".to_string(),
                    nonce: Some(general_purpose::STANDARD.encode(&nonce)),
                    permissions: permissions.clone(),
                }))
            }
        }
    }

    /// Encrypt data with symmetric key
    fn encrypt_with_symmetric_key(&self, data: &[u8], key_b64: &str) -> Result<(Vec<u8>, Vec<u8>), DfsError> {
        let key_bytes = general_purpose::STANDARD.decode(key_b64)
            .map_err(|e| DfsError::EncryptionError(format!("Invalid key: {}", e)))?;
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| DfsError::EncryptionError(format!("Encryption failed: {}", e)))?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Add a member to a permission group
    pub async fn add_group_member(
        &self,
        group_id: String,
        new_member: String,
        requester: String,
    ) -> Result<(), DfsError> {
        let mut groups = self.permission_groups.write().await;
        
        let group = groups.get_mut(&group_id)
            .ok_or_else(|| DfsError::GroupNotFound(group_id.clone()))?;

        // Check if requester has admin permissions
        if group.owner != requester {
            return Err(DfsError::AccessDenied(
                "Only group owner can add members".to_string()
            ));
        }

        if !group.members.contains(&new_member) {
            group.members.push(new_member.clone());
            group.last_modified = Utc::now();
            println!("👥 Added {} to group {}", new_member, group_id);
        }

        Ok(())
    }

    /// Remove a member from a permission group
    pub async fn remove_group_member(
        &self,
        group_id: String,
        member_to_remove: String,
        requester: String,
    ) -> Result<(), DfsError> {
        let mut groups = self.permission_groups.write().await;
        
        let group = groups.get_mut(&group_id)
            .ok_or_else(|| DfsError::GroupNotFound(group_id.clone()))?;

        // Check if requester has admin permissions
        if group.owner != requester {
            return Err(DfsError::AccessDenied(
                "Only group owner can remove members".to_string()
            ));
        }

        group.members.retain(|member| member != &member_to_remove);
        group.last_modified = Utc::now();
        println!("❌ Removed {} from group {}", member_to_remove, group_id);

        Ok(())
    }
}

/// Filesystem statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsStatistics {
    pub total_files: usize,
    pub total_storage_bytes: u64,
    pub active_contracts: usize,
    pub storage_nodes: usize,
    pub assembly_stats: AssemblyStats,
    pub cache_hit_rate: f32,
    pub avg_replication: f32,
}

/// Individual node earnings report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEarningsReport {
    pub node_id: String,
    pub total_earnings: u64,
    pub contracts_completed: u32,
    pub reliability_score: f32,
    pub avg_response_time: u32,
    pub storage_capacity_gb: f64,
    pub earnings_per_gb: f64,
    pub performance_tier: String,
}

/// Network-wide rewards distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsDistributionStats {
    pub total_earnings_distributed: u64,
    pub active_storage_providers: usize,
    pub completed_contracts: usize,
    pub total_storage_capacity_gb: f64,
    pub avg_earnings_per_provider: u64,
    pub avg_reliability_score: f32,
    pub premium_tier_earnings: u64,
    pub standard_tier_earnings: u64,
    pub basic_tier_earnings: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::CapabilityType;

    #[tokio::test]
    async fn test_dfs_creation() {
        let config = DfsConfig::default();
        assert_eq!(config.default_replication, 3);
        assert_eq!(config.storage_price_per_gb_month, 10);
    }

    #[tokio::test]
    async fn test_file_hash_calculation() {
        let config = DfsConfig::default();
        
        // Create mock dependencies (simplified for test)
        let local_node = Arc::new(AsyncRwLock::new(UnifiedNode::new(
            "test_node".to_string(),
            NodeCapability {
                cpus: 4,
                gpus: 1,
                gpu_memory_gb: 8,
                available_stake: 1000,
                reputation: 50,
                capability_types: vec![CapabilityType::Storage],
            },
            1000,
        )));
        
        // This test would require more infrastructure setup in a real implementation
        // For now, just verify the hash calculation function works
        let test_data = b"Hello, BCAI DFS!";
        
        // Create a minimal DFS instance just for testing hash calculation
        use crate::distributed_storage::{DistributedStorage, StorageConfig};
        use crate::network::NetworkCoordinator;
        
        let storage_config = StorageConfig::default();
        let storage_coordinator = Arc::new(DistributedStorage::new(storage_config, "test".to_string()));
        let network_coordinator = Arc::new(AsyncRwLock::new(NetworkCoordinator::new(
            local_node.read().await.clone()
        )));
        let token_ledger = Arc::new(AsyncRwLock::new(TokenLedger::new()));
        let contract_engine = Arc::new(AsyncRwLock::new(crate::smart_contracts::SmartContractEngine::new()));
        
        let dfs = DecentralizedFilesystem::new(
            config,
            local_node,
            network_coordinator,
            storage_coordinator,
            None,
            token_ledger,
            contract_engine,
        );
        
        let hash = dfs.calculate_file_hash(test_data);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 produces 64-character hex string
        
        println!("✅ File hash calculated: {}", hash);
    }
}