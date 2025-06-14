//! Unified node architecture integrating P2P networking, job management, and runtime execution.
//!
//! This module provides the integration layer that connects all BCAI components:
//! - P2P networking for distributed communication
//! - Job management with persistent state
//! - Runtime execution with PoUW verification
//! - Token operations and economic incentives

use crate::{
    evaluator::Evaluator,
    job_manager::{JobManager, JobManagerError},
    pouw::{generate_task, Solution},
    token::{LedgerError, TokenLedger},
    trainer::Trainer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Node capability types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NodeCapability {
    BasicCompute,
    GpuAccelerated,
    HighMemory,
    Storage,
    Network,
}

/// Training job with distributed coordination
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Posted,
    WorkersAssigned,
    Training,
    EvaluationPending,
    Completed,
    Failed,
}

/// Results from distributed training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingResult {
    pub job_id: u64,
    pub model_hash: String,
    pub accuracy_metrics: HashMap<String, f64>,
    pub pouw_solution: Solution,
    pub worker_signatures: Vec<String>,
}

/// Errors that can occur during node operations
#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Job management error: {0}")]
    JobManager(#[from] JobManagerError),
    #[error("Token ledger error: {0}")]
    Ledger(#[from] LedgerError),
    #[error("Network error: {0}")]
    Network(#[from] crate::network::NetworkError),
    #[error("Insufficient stake for operation")]
    InsufficientStake,
    #[error("Node capability mismatch")]
    CapabilityMismatch,
    #[error("Training verification failed")]
    TrainingVerificationFailed,
    #[error("Network communication error: {0}")]
    NetworkCommunication(String),
    #[error("Job not found: {0}")]
    JobNotFound(u64),
    #[error("Invalid job state transition")]
    InvalidStateTransition,
}

/// Unified node implementation
#[derive(Debug, Clone)]
pub struct UnifiedNode {
    pub node_id: String,
    pub capability: NodeCapability,
    pub status: NodeStatus,
    job_manager: JobManager,
    trainer: Trainer,
    evaluator: Evaluator,
    distributed_jobs: HashMap<u64, DistributedJob>,
    pending_results: HashMap<u64, TrainingResult>,
    current_block: u64,
}

/// Node status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NodeStatus {
    Active,
    Idle,
    Busy,
    Offline,
}

impl UnifiedNode {
    /// Create a new unified node with the given capabilities
    pub fn new(node_id: String, capability: NodeCapability, initial_tokens: u64) -> Self {
        let mut ledger = TokenLedger::new();
        ledger.mint(&node_id, initial_tokens);

        let job_manager = JobManager::new(ledger);
        let trainer = Trainer::new(&node_id);
        let evaluator = Evaluator::new(&node_id);

        Self {
            node_id,
            capability,
            status: NodeStatus::Idle,
            job_manager,
            trainer,
            evaluator,
            distributed_jobs: HashMap::new(),
            pending_results: HashMap::new(),
            current_block: 1,
        }
    }

    /// Get current token balance
    pub fn balance(&self) -> u64 {
        self.job_manager.ledger().balance(&self.node_id)
    }

    /// Get current staked amount
    pub fn staked(&self) -> u64 {
        self.job_manager.ledger().staked(&self.node_id)
    }

    /// Stake tokens to participate in network
    pub fn stake_tokens(&mut self, amount: u64) -> Result<(), NodeError> {
        self.job_manager.ledger_mut().stake(&self.node_id, amount)?;
        self.capability.available_stake = self.staked();
        Ok(())
    }

    /// Post a distributed training job
    pub fn post_distributed_job(
        &mut self,
        description: String,
        reward: u64,
        required_capability: NodeCapability,
        data_hash: String,
        model_spec: String,
        deadline_blocks: u64,
    ) -> Result<u64, NodeError> {
        // Check sufficient balance
        if self.balance() < reward {
            return Err(NodeError::JobManager(JobManagerError::InsufficientBalance));
        }

        // Reserve tokens in escrow
        self.job_manager.ledger_mut().transfer(&self.node_id, "escrow", reward)?;

        let job_id = self.distributed_jobs.len() as u64 + 1;
        let job = DistributedJob {
            id: job_id,
            description,
            reward,
            required_capability,
            data_hash,
            model_spec,
            assigned_workers: Vec::new(),
            evaluators: Vec::new(),
            status: JobStatus::Posted,
            created_block: self.current_block,
            completion_deadline: self.current_block + deadline_blocks,
        };

        self.distributed_jobs.insert(job_id, job);
        Ok(job_id)
    }

    /// Volunteer to work on a distributed job
    pub fn volunteer_for_job(&mut self, job_id: u64, min_stake: u64) -> Result<(), NodeError> {
        // Check stake requirements first (no borrow needed)
        if self.staked() < min_stake {
            return Err(NodeError::InsufficientStake);
        }

        // Get job and check requirements
        let job = self.distributed_jobs.get(&job_id).ok_or(NodeError::JobNotFound(job_id))?;

        // Check capability requirements
        if !self.meets_capability_requirements(&job.required_capability) {
            return Err(NodeError::CapabilityMismatch);
        }

        // Check job status
        if job.status != JobStatus::Posted {
            return Err(NodeError::InvalidStateTransition);
        }

        // Now get mutable reference and update
        let job = self.distributed_jobs.get_mut(&job_id).unwrap();
        job.assigned_workers.push(self.node_id.clone());
        if job.assigned_workers.len() >= 3 {
            // Assume we need 3 workers
            job.status = JobStatus::WorkersAssigned;
        }

        Ok(())
    }

    /// Execute training task with PoUW verification
    pub fn execute_training(
        &mut self,
        job_id: u64,
        difficulty: u32,
    ) -> Result<TrainingResult, NodeError> {
        let job = self.distributed_jobs.get_mut(&job_id).ok_or(NodeError::JobNotFound(job_id))?;

        if job.status != JobStatus::WorkersAssigned {
            return Err(NodeError::InvalidStateTransition);
        }

        job.status = JobStatus::Training;

        // Generate PoUW task based on job parameters
        let task = generate_task(4, job.id); // Use job ID as seed for deterministic task

        // Execute training computation (simplified for this integration)
        let solution = self.trainer.train(&task, difficulty);

        // Verify our own solution (use same verification as evaluator for consistency)
        if !self.evaluator.evaluate(&task, &solution, difficulty) {
            return Err(NodeError::TrainingVerificationFailed);
        }

        // Create training result
        let mut accuracy_metrics = HashMap::new();
        accuracy_metrics.insert("accuracy".to_string(), 0.95); // Placeholder metric

        let result = TrainingResult {
            job_id,
            model_hash: format!("model_hash_{}", job_id),
            accuracy_metrics,
            pouw_solution: solution,
            worker_signatures: vec![self.node_id.clone()],
        };

        job.status = JobStatus::EvaluationPending;
        self.pending_results.insert(job_id, result.clone());

        Ok(result)
    }

    /// Evaluate training results from other nodes
    pub fn evaluate_training_result(
        &mut self,
        job_id: u64,
        result: &TrainingResult,
    ) -> Result<bool, NodeError> {
        let job = self.distributed_jobs.get(&job_id).ok_or(NodeError::JobNotFound(job_id))?;

        // Generate the same task that should have been used for training
        let task = generate_task(4, job.id);

        // Verify the PoUW solution
        let is_valid = self.evaluator.evaluate(&task, &result.pouw_solution, 0x0000ffff);

        if is_valid {
            // Update reputation for successful work
            self.job_manager.ledger_mut().adjust_reputation(&self.node_id, 1);
            self.capability.reputation = self.job_manager.ledger().reputation(&self.node_id);
        }

        Ok(is_valid)
    }

    /// Complete a distributed job and distribute rewards
    pub fn complete_distributed_job(&mut self, job_id: u64) -> Result<(), NodeError> {
        let job = self.distributed_jobs.get_mut(&job_id).ok_or(NodeError::JobNotFound(job_id))?;

        if job.status != JobStatus::EvaluationPending {
            return Err(NodeError::InvalidStateTransition);
        }

        // Distribute rewards among workers
        let reward_per_worker = job.reward / job.assigned_workers.len() as u64;
        for worker in &job.assigned_workers {
            self.job_manager.ledger_mut().transfer("escrow", worker, reward_per_worker)?;
        }

        job.status = JobStatus::Completed;
        self.pending_results.remove(&job_id);

        Ok(())
    }

    /// Check if node meets capability requirements
    fn meets_capability_requirements(&self, required: &NodeCapability) -> bool {
        self.capability.cpus >= required.cpus
            && self.capability.gpus >= required.gpus
            && self.capability.gpu_memory_gb >= required.gpu_memory_gb
            && self.capability.available_stake >= required.available_stake
    }

    /// Get all distributed jobs
    pub fn distributed_jobs(&self) -> &HashMap<u64, DistributedJob> {
        &self.distributed_jobs
    }

    /// Get mutable reference to distributed jobs (for testing)
    pub fn distributed_jobs_mut(&mut self) -> &mut HashMap<u64, DistributedJob> {
        &mut self.distributed_jobs
    }

    /// Get pending training results
    pub fn pending_results(&self) -> &HashMap<u64, TrainingResult> {
        &self.pending_results
    }

    /// Update current block height
    pub fn update_block_height(&mut self, block: u64) {
        self.current_block = block;
    }

    /// Get jobs available for this node's capabilities
    pub fn get_available_jobs(&self) -> Vec<&DistributedJob> {
        self.distributed_jobs
            .values()
            .filter(|job| {
                job.status == JobStatus::Posted
                    && self.meets_capability_requirements(&job.required_capability)
                    && self.current_block < job.completion_deadline
            })
            .collect()
    }

    /// Get node statistics
    pub fn get_stats(&self) -> NodeStats {
        NodeStats {
            node_id: self.node_id.clone(),
            balance: self.balance(),
            staked: self.staked(),
            reputation: self.capability.reputation,
            jobs_completed: self
                .distributed_jobs
                .values()
                .filter(|job| {
                    job.status == JobStatus::Completed
                        && job.assigned_workers.contains(&self.node_id)
                })
                .count(),
            jobs_active: self
                .distributed_jobs
                .values()
                .filter(|job| {
                    matches!(job.status, JobStatus::Training | JobStatus::EvaluationPending)
                        && job.assigned_workers.contains(&self.node_id)
                })
                .count(),
        }
    }

    pub fn set_status(&mut self, status: NodeStatus) {
        self.status = status;
    }
    
    pub fn is_available(&self) -> bool {
        matches!(self.status, NodeStatus::Idle | NodeStatus::Active)
    }
}

/// Node statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub node_id: String,
    pub balance: u64,
    pub staked: u64,
    pub reputation: i32,
    pub jobs_completed: usize,
    pub jobs_active: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_node_creation() {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 0,
            reputation: 0,
        };

        let node = UnifiedNode::new("test_node".to_string(), capability, 1000);
        assert_eq!(node.balance(), 1000);
        assert_eq!(node.staked(), 0);
    }

    #[test]
    fn stake_and_post_job_flow() -> Result<(), NodeError> {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 0,
            reputation: 0,
        };

        let mut node = UnifiedNode::new("test_node".to_string(), capability.clone(), 1000);

        // Stake tokens
        node.stake_tokens(100)?;
        assert_eq!(node.staked(), 100);

        // Post job
        let job_id = node.post_distributed_job(
            "Test training job".to_string(),
            500,
            capability,
            "data_hash_123".to_string(),
            "model_spec_456".to_string(),
            100,
        )?;

        assert_eq!(job_id, 1);
        assert_eq!(node.balance(), 400); // 1000 - 100 (staked) - 500 (escrowed)

        Ok(())
    }

    #[test]
    fn volunteer_and_execute_training() -> Result<(), NodeError> {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 100,
            reputation: 0,
        };

        let mut node = UnifiedNode::new("worker_node".to_string(), capability.clone(), 1000);
        node.stake_tokens(150)?;

        // Create another node to post job
        let mut job_poster = UnifiedNode::new("job_poster".to_string(), capability.clone(), 1000);
        let job_id = job_poster.post_distributed_job(
            "Test training job".to_string(),
            500,
            capability,
            "data_hash_123".to_string(),
            "model_spec_456".to_string(),
            100,
        )?;

        // Transfer job to worker node for testing
        let mut job = job_poster.distributed_jobs.get(&job_id).unwrap().clone();
        // Add enough workers to make job ready for training
        job.assigned_workers =
            vec!["worker1".to_string(), "worker2".to_string(), "worker3".to_string()];
        job.status = JobStatus::WorkersAssigned;
        node.distributed_jobs.insert(job_id, job);

        // Volunteer for job (already assigned above)
        // node.volunteer_for_job(job_id, 100)?;

        // Execute training
        let result = node.execute_training(job_id, 0x0000ffff)?;
        assert_eq!(result.job_id, job_id);

        // Evaluate result
        let is_valid = node.evaluate_training_result(job_id, &result)?;
        assert!(is_valid);

        Ok(())
    }
}
