//! Data structures for the Decentralized Filesystem (DFS).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use chrono::{DateTime, Utc};

// --- Error Types ---

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

// --- Configuration ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsConfig {
    pub default_replication: u32,
    pub default_chunk_size_mb: u32,
    pub storage_price_per_gb_month: u64,
    pub bandwidth_price_per_gb: u64,
    pub min_storage_duration_hours: u64,
    pub max_storage_duration_hours: u64,
    pub parallel_assembly_workers: u32,
    pub escrow_confirmation_blocks: u64,
}

impl Default for DfsConfig {
    fn default() -> Self {
        Self {
            default_replication: 3,
            default_chunk_size_mb: 4,
            storage_price_per_gb_month: 10,
            bandwidth_price_per_gb: 1,
            min_storage_duration_hours: 24,
            max_storage_duration_hours: 8760,
            parallel_assembly_workers: 8,
            escrow_confirmation_blocks: 6,
        }
    }
}

// --- Core File and Chunk Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsFile {
    pub file_hash: String,
    pub filename: String,
    pub size: u64,
    pub content_type: String,
    pub owner: String,
    pub storage_contracts: Vec<String>,
    pub chunks: Vec<DfsChunk>,
    pub replication: u32,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub tags: Vec<String>,
    pub encryption_metadata: EncryptionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsChunk {
    pub index: u32,
    pub hash: String,
    pub size: u32,
    pub storage_nodes: Vec<String>,
    pub verified_at: DateTime<Utc>,
}

// --- Permissions and Encryption Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilePermissions {
    Public,
    OwnerOnly {
        owner: String,
        encrypted_key: String,
    },
    Group {
        group_id: String,
        encrypted_key: String,
        members: Vec<String>,
    },
    Custom {
        access_list: HashMap<String, String>,
    },
    TimeBound {
        base_permissions: Box<FilePermissions>,
        access_grants: Vec<TemporaryAccess>,
        default_expiry: Option<DateTime<Utc>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryAccess {
    pub user_id: String,
    pub encrypted_key: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub access_type: TemporaryAccessType,
    pub granted_by: String,
    pub usage_count: u64,
    pub max_usage: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TemporaryAccessType {
    ReadOnly,
    ReadWrite,
    Trial,
    Emergency,
    Subscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub members: Vec<String>,
    pub group_key: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub permissions: GroupPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupPermissions {
    Read,
    ReadWrite,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKeyPair {
    pub user_id: String,
    pub public_key: String,
    pub private_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    pub is_encrypted: bool,
    pub encryption_algorithm: String,
    pub nonce: Option<String>,
    pub permissions: FilePermissions,
}

// --- Contract and Node Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageContract {
    pub contract_id: String,
    pub file_hash: String,
    pub storage_nodes: Vec<String>,
    pub client: String,
    pub escrow_amount: u64,
    pub payment_per_node: u64,
    pub duration: Duration,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: StorageContractStatus,
    pub last_verified: DateTime<Utc>,
    pub required_availability: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StorageContractStatus {
    Active,
    Expired,
    Completed,
    Breached,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNodeMetrics {
    pub node_id: String,
    pub total_storage: u64,
    pub available_storage: u64,
    pub reliability: f32,
    pub avg_response_time: u32,
    pub bandwidth_capacity: u64,
    pub total_earnings: u64,
    pub active_contracts: u32,
    pub last_heartbeat: DateTime<Utc>,
    pub region: String,
}

// --- Assembly and Statistics ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssemblyPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyRequest {
    pub file_hash: String,
    pub requester: String,
    pub priority: AssemblyPriority,
    pub max_assembly_time: Duration,
    pub bandwidth_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssemblyStats {
    pub files_assembled: u64,
    pub bytes_assembled: u64,
    pub avg_assembly_time: f64,
    pub cache_hit_rate: f32,
    pub parallel_efficiency: f32,
}

// --- Events and Logging ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    pub file_hash: String,
    pub requester: String,
    pub access_time: DateTime<Utc>,
    pub was_allowed: bool,
    pub reason: String,
}

pub enum DfsEvent {
    FileStored { file_hash: String, size: u64, replication: u32 },
    FileRetrieved { file_hash: String, assembly_time: Duration },
    StorageContractCreated { contract_id: String, duration: Duration },
    StorageContractCompleted { contract_id: String, payment: u64 },
    ChunkVerified { chunk_hash: String, node_id: String },
    EscrowReleased { contract_id: String, amount: u64 },
} 