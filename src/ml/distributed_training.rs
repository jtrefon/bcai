use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// --- Core Data Structures for Distributed Training ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    pub id: Uuid,
    pub model_id: Uuid,
    pub dataset_id: Uuid,
    pub config: TrainingConfig,
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub strategy: DistributionStrategy,
    pub num_workers: u32,
    pub epochs: u32,
    pub batch_size: u32,
    pub learning_rate: f64,
    pub optimizer: String,
    pub hyperparameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionStrategy {
    DataParallelism,
    ModelParallelism,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    pub id: Uuid,
    pub job_id: Uuid,
    pub rank: u32,
    pub status: WorkerStatus,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub capabilities: WorkerCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Running,
    Failed,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_count: u32,
}

// NOTE: Removed placeholder implementation structs:
// - DistributedTrainingCoordinator
// - JobScheduler
// - ResourceManager
// - FaultDetector
// This file now only defines the data models for distributed training. 