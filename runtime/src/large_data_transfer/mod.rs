//! Large Data Transfer Module
//!
//! This module provides efficient transfer of large data blocks (GB to TB scale)
//! through chunking, content addressing, compression, and streaming protocols.
//!
//! Key features:
//! - Content-addressed chunking with integrity verification
//! - Streaming transfer protocol with flow control
//! - Bandwidth management and QoS controls
//! - Local caching with deduplication
//! - Encryption and security features

pub mod cache;
pub mod chunk;
pub mod compression;
pub mod config;
pub mod crypto;
pub mod descriptor;
pub mod error;
pub mod manager;
pub mod network;
pub mod protocol;
pub mod types;

// Re-export core types
pub use chunk::{ChunkId, ChunkInfo, DataChunk};
pub use config::{
    CacheConfig, CompressionConfig, EncryptionConfig, LargeDataConfig, RetryConfig,
};
pub use descriptor::{LargeDataDescriptor, TransferMetadata};
pub use error::{LargeDataError, LargeDataResult};
pub use manager::{ChunkManager, ChunkManagerConfig};
pub use network::{NetworkTransferCoordinator, NetworkPeerInfo, NetworkStats};
pub use protocol::{TransferError, TransferMessage, TransferState};
pub use types::{TransferPriority, TransferStats}; 