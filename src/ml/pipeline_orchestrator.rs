use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::time::Duration;

// --- Core Data Structures for Pipeline Orchestration ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tasks: Vec<PipelineTask>,
    pub trigger: PipelineTrigger,
    pub owner: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTask {
    pub id: Uuid,
    pub name: String,
    pub task_type: TaskType,
    pub dependencies: Vec<Uuid>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    DataIngestion,
    DataValidation,
    DataPreprocessing,
    ModelTraining,
    ModelEvaluation,
    ModelDeployment,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTrigger {
    pub trigger_type: TriggerType,
    pub schedule: Option<String>, // e.g., cron schedule
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Manual,
    Scheduled,
    Webhook,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecution {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: ExecutionStatus,
    pub task_statuses: HashMap<Uuid, TaskExecutionStatus>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub triggered_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskExecutionStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

// NOTE: Removed placeholder implementation structs:
// - PipelineOrchestrator
// - ResourceManager
// - PipelineScheduler
// This file now only defines the data models for pipeline orchestration. 