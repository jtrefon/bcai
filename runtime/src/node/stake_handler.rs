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

    /// Unstakes previously staked tokens back to the node's balance.
    pub fn unstake_tokens(&mut self, amount: u64) -> Result<(), NodeError> {
        self.job_manager
            .ledger_mut()
            .unstake(&self.node_id, amount)?;
        self.capability.available_stake = self.staked();
        Ok(())
    }

    /// Slashes the node's stake as a penalty.
    pub fn slash_stake(&mut self, amount: u64) -> Result<(), NodeError> {
        self.job_manager.ledger_mut().slash(&self.node_id, amount)?;
        self.capability.available_stake = self.staked();
        Ok(())
    }
}
