use super::{
    error::NodeError,
    node::UnifiedNode,
    types::JobStatus,
};

impl UnifiedNode {
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
} 