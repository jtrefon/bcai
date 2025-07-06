pub mod storage;
pub mod replication;
pub mod reward;
pub mod allocation;
pub mod daemon;

// Re-export commonly used items so callers can simply `use distributed_storage::*`.
pub use storage::{StorageConfig, ConsistencyLevel, StorageEntry, StorageResult, StorageStats};
pub use replication::{StorageNode, ReplicationManager};
pub use reward::{RewardPolicy, calculate_reward};
// pub use allocation::{StoragePolicy, NodeMetrics, allocate_nodes}; // allocate_nodes was commented out
pub use allocation::{StoragePolicy, NodeMetrics}; // Keep StoragePolicy and NodeMetrics
// pub use daemon::run_auto_heal; // run_auto_heal was commented out
