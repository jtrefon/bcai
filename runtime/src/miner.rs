//! The logic for creating and solving a new block.

use crate::blockchain::{
    block::Block, chain::Blockchain, transaction::Transaction, validation, BlockchainError,
};
use crate::pouw::PoUWTask;
use std::sync::{Arc, Mutex};

/// Creates a new block, solving the PoUW challenge.
pub fn mine_block(
    miner_pubkey: String,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
) -> Result<Block, BlockchainError> {
    let mut chain = blockchain.lock().unwrap();
    let mempool_guard = mempool.lock().unwrap();

    let prev_block = chain
        .get_last_block()
        .ok_or(BlockchainError::NoBlocksInChain)?;

    // Select valid transactions from the mempool
    let mut transactions_to_include = Vec::new();
    let mut temp_state = chain.state.clone(); // Create a temporary state for validation

    for tx in mempool_guard.iter() {
        if validation::validate_transaction_stateful(tx, &temp_state).is_ok() {
            // If valid, apply it to the temp state and add to our list
            temp_state.apply_transaction(tx)?;
            transactions_to_include.push(tx.clone());
        }
    }
    
    // Drop the locks early
    drop(mempool_guard);
    drop(chain);

    let tx_root = Block::calculate_merkle_root(&transactions_to_include);
    let new_block_index = (prev_block.index + 1) as u32;

    // TODO: The PoUW task should come from a job queue.
    let pouw_task = PoUWTask::new("model_1".to_string(), "dataset_1".to_string(), 10);
    let pouw_solution = pouw_task.solve();

    let new_block = Block::new(
        new_block_index,
        prev_block.hash.clone(),
        transactions_to_include,
        0, // Difficulty is part of the task now
        miner_pubkey,
        pouw_task,
        pouw_solution,
    );

    Ok(new_block)
} 