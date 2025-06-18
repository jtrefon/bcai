//! Implements the job lifecycle management logic for the `UnifiedNode`.

use super::{
    error::NodeError,
    node::UnifiedNode,
    types::{DistributedJob, JobStatus, NodeCapability},
};
use crate::job_manager::JobManagerError;

impl UnifiedNode {
    /// Posts a new distributed training job to the network.
    pub fn post_distributed_job(
        &mut self,
        description: String,
        reward: u64,
        required_capability: NodeCapability,
        data_hash: String,
        model_spec: String,
        deadline_blocks: u64,
    ) -> Result<u64, NodeError> {
        if self.balance() < reward {
            return Err(NodeError::JobManager(JobManagerError::InsufficientBalance));
        }

        // In a real system, this would be a transaction sent to the blockchain
        // to be processed by the consensus mechanism.
        self.job_manager
            .ledger_mut()
            .transfer(&self.node_id, "escrow", reward)?;

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

    /// Allows the node to volunteer for an available job.
    pub fn volunteer_for_job(&mut self, job_id: u64, min_stake: u64) -> Result<(), NodeError> {
        if self.staked() < min_stake {
            return Err(NodeError::InsufficientStake);
        }

        let job = self
            .distributed_jobs
            .get_mut(&job_id)
            .ok_or(NodeError::JobNotFound(job_id))?;

        if !self.meets_capability_requirements(&job.required_capability) {
            return Err(NodeError::CapabilityMismatch);
        }

        if job.status != JobStatus::Posted {
            return Err(NodeError::InvalidStateTransition);
        }

        job.assigned_workers.push(self.node_id.clone());
        // Simple logic: if enough workers have volunteered, start the job.
        if job.assigned_workers.len() >= 3 {
            job.status = JobStatus::WorkersAssigned;
        }

        Ok(())
    }

    /// Completes a job, distributing the reward from escrow to the workers.
    pub fn complete_distributed_job(&mut self, job_id: u64) -> Result<(), NodeError> {
        let job = self
            .distributed_jobs
            .get_mut(&job_id)
            .ok_or(NodeError::JobNotFound(job_id))?;

        if job.status != JobStatus::EvaluationPending {
            return Err(NodeError::InvalidStateTransition);
        }

        // Pay out the reward to the assigned workers.
        let reward_per_worker = job.reward / job.assigned_workers.len() as u64;
        for worker_id in &job.assigned_workers {
            self.job_manager
                .ledger_mut()
                .transfer("escrow", worker_id, reward_per_worker)?;
        }

        job.status = JobStatus::Completed;
        Ok(())
    }

    /// Checks if the node's capabilities meet the job's requirements.
    fn meets_capability_requirements(&self, required: &NodeCapability) -> bool {
        self.capability.cpus >= required.cpus
            && self.capability.gpus >= required.gpus
            && self.capability.gpu_memory_gb >= required.gpu_memory_gb
            && self.capability.available_stake >= required.available_stake
    }
} 