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

        // Use the job manager to escrow the reward and record the submission.
        let job_id = self.distributed_jobs.len() as u64 + 1;
        self.job_manager
            .submit_job(&self.node_id, job_id, reward)?;

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
} 