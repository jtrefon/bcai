//! Defines the core data structures used by the `UnifiedNode` and its services.

use crate::pouw::Solution;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specifies the hardware and economic parameters of a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapability {
    pub cpus: u32,
    pub gpus: u32,
    pub gpu_memory_gb: u32,
    pub available_stake: u64,
    pub reputation: i32,
    pub capability_types: Vec<CapabilityType>,
}

/// The different types of capabilities a node can advertise.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityType {
    BasicCompute,
    GpuAccelerated,
    HighMemory,
    Storage,
    Network,
    Training,
    Inference,
}

/// Represents a distributed training job posted to the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedJob {
    pub id: u64,
    pub description: String,
    pub reward: u64,
    pub required_capability: NodeCapability,
    pub data_hash: String,
    pub model_spec: String,
    pub assigned_workers: Vec<String>,
    pub evaluators: Vec<String>,
    pub status: JobStatus,
    pub created_block: u64,
    pub completion_deadline: u64,
}

/// The lifecycle status of a `DistributedJob`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Posted,
    WorkersAssigned,
    Training,
    EvaluationPending,
    Completed,
    Failed,
}

/// The result submitted by a worker after completing a training job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingResult {
    pub job_id: u64,
    pub model_hash: String,
    pub accuracy_metrics: HashMap<String, f64>,
    pub pouw_solution: Solution,
    pub worker_signatures: Vec<String>,
}

/// The operational status of a `UnifiedNode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Active,
    Idle,
    Busy,
    Offline,
}

/// A snapshot of key statistics for a node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeStats {
    pub node_id: String,
    pub balance: u64,
    pub staked: u64,
    pub reputation: i32,
    pub jobs_completed: usize,
    pub jobs_active: usize,
} 