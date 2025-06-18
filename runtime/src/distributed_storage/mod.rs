pub mod storage;
pub mod replication;

// Re-export commonly used types
pub use storage::{StorageConfig, ConsistencyLevel, StorageEntry, StorageResult, StorageStats};
pub use replication::StorageNode; 