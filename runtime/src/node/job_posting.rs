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
} 