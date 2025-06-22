pub mod storage;
pub mod replication;
pub mod reward;
pub mod daemon;

// Re-export commonly used types
pub use storage::{StorageConfig, ConsistencyLevel, StorageEntry, StorageResult, StorageStats};
pub use replication::{StorageNode, ReplicationManager};
pub use reward::{RewardPolicy, calculate_reward};
pub use daemon::run_auto_heal;
