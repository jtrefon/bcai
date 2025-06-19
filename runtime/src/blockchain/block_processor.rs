use crate::blockchain::{
    block::Block,
    constants::BLOCK_REWARD,
    error::BlockchainError,
    state::BlockchainState,
    validation,
};

pub struct BlockProcessor;

impl BlockProcessor {
    /// Processes and validates a new block, applying transactions and rewarding the miner
    pub fn process_block(
        block: &Block,
        prev_block: &Block,
        state: &mut BlockchainState,
    ) -> Result<(), BlockchainError> {
        // Validate the block
        validation::validate_block(block, prev_block, state)?;

        // Apply transactions and calculate total fees
        let mut total_fees = 0;
        for tx in &block.transactions {
            state.apply_transaction(tx)?;
            total_fees += tx.fee;
        }

        // Reward the miner
        Self::reward_miner(block, total_fees, state)?;

        Ok(())
    }

    /// Rewards the miner with block reward plus transaction fees
    fn reward_miner(
        block: &Block,
        total_fees: u64,
        state: &mut BlockchainState,
    ) -> Result<(), BlockchainError> {
        let miner_reward = BLOCK_REWARD
            .checked_add(total_fees)
            .ok_or(BlockchainError::TransactionValidationError(
                "Miner reward overflow".to_string(),
            ))?;

        let miner_balance = state.balances.entry(block.miner.clone()).or_insert(0);
        *miner_balance = miner_balance
            .checked_add(miner_reward)
            .ok_or(BlockchainError::TransactionValidationError(
                "Miner balance overflow".to_string(),
            ))?;

        Ok(())
    }
} 