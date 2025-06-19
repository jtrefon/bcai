use super::{
    error::NodeError,
    node::UnifiedNode,
    types::JobStatus,
};

impl UnifiedNode {
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
} 