//! Defines the `UnifiedNode`, the central state container for a BCAI node.

use super::types::{DistributedJob, NodeCapability, NodeStatus, TrainingResult};
use crate::{
    evaluator::Evaluator, job_manager::JobManager, token::TokenLedger, trainer::Trainer,
};
use std::collections::HashMap;

/// The central state-holding struct for a node.
///
/// This struct is intended to be a simple container for the various components
/// and state maps. The complex business logic for operating on this state is

/// implemented in separate service/handler modules.
#[derive(Debug, Clone)]
pub struct UnifiedNode {
    pub node_id: String,
    pub capability: NodeCapability,
    pub status: NodeStatus,
    pub(crate) job_manager: JobManager,
    pub(crate) trainer: Trainer,
    pub(crate) evaluator: Evaluator,
    pub(crate) distributed_jobs: HashMap<u64, DistributedJob>,
    pub(crate) pending_results: HashMap<u64, TrainingResult>,
    pub(crate) current_block: u64,
}

impl UnifiedNode {
    /// Create a new unified node with the given capabilities and initial token balance.
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
    
    /// Returns the current token balance of the node.
    pub fn balance(&self) -> u64 {
        self.job_manager.ledger().balance(&self.node_id)
    }

    /// Returns the current staked token amount of the node.
    pub fn staked(&self) -> u64 {
        self.job_manager.ledger().staked(&self.node_id)
    }

    /// Returns a list of jobs that are currently posted and available to be worked on.
    pub fn get_available_jobs(&self) -> Vec<&DistributedJob> {
        self.distributed_jobs
            .values()
            .filter(|job| job.status == super::types::JobStatus::Posted)
            .collect()
    }
    
    /// Updates the node's current block height.
    pub fn update_block_height(&mut self, block: u64) {
        self.current_block = block;
    }

    /// Sets the node's operational status.
    pub fn set_status(&mut self, status: NodeStatus) {
        self.status = status;
    }

    /// A simple check to see if the node is in a state where it can accept new work.
    pub fn is_available(&self) -> bool {
        matches!(self.status, NodeStatus::Active | NodeStatus::Idle)
    }
} 