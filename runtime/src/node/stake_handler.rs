//! Implements the staking and token management logic for the `UnifiedNode`.

use super::{error::NodeError, node::UnifiedNode};

impl UnifiedNode {
    /// Stakes a specified amount of tokens for the node.
    pub fn stake_tokens(&mut self, amount: u64) -> Result<(), NodeError> {
        self.job_manager
            .ledger_mut()
            .stake(&self.node_id, amount)?;
        // Update the node's own capability view of its stake.
        self.capability.available_stake = self.staked();
        Ok(())
    }
} 