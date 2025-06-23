pub mod storage;
pub mod replication;
pub mod reward;
pub mod allocation;
pub mod daemon;

// Re-export commonly used items so callers can simply `use distributed_storage::*`.
pub use storage::{StorageConfig, ConsistencyLevel, StorageEntry, StorageResult, StorageStats};
pub use replication::{StorageNode, ReplicationManager};
pub use reward::{RewardPolicy, calculate_reward};
pub use allocation::{StoragePolicy, NodeMetrics, allocate_nodes};
pub use daemon::run_auto_heal;
