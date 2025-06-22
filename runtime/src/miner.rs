//! The logic for creating and solving a new block.

use crate::blockchain::{
    block::Block, chain::Blockchain, transaction::Transaction, validation, BlockchainError,
};
use crate::job::Job;
use crate::pouw::PoUWTask;
use std::collections::{HashSet, VecDeque};
use tokio::sync::Mutex;
use std::sync::Arc;

/// Creates a new block, solving the PoUW challenge.
pub async fn mine_block(
    miner_pubkey: String,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<HashSet<Transaction>>>,
    job_queue: Arc<Mutex<VecDeque<Job>>>,
) -> Result<Block, BlockchainError> {
    let mut chain = blockchain.lock().await;
    let mempool_guard = mempool.lock().await;

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
    
    // We still need data from prev_block before releasing the chain lock.
    let prev_block_hash = prev_block.hash.clone();
    let new_block_index = (prev_block.index + 1) as u32;

    // Drop the locks early (after capturing necessary values)
    drop(mempool_guard);
    drop(chain);

    let tx_root = Block::calculate_merkle_root(&transactions_to_include);

    // Get a job from the queue for the PoUW task.
    let job = job_queue.lock().await.pop_front();

    // TODO: The PoUW task should come from a job queue.
    let pouw_task = if let Some(job) = job {
        PoUWTask::new(job.model_id, job.dataset_id, job.iterations)
    } else {
        // Fallback to a dummy task if the job queue is empty
        PoUWTask::new("default_model".to_string(), "default_dataset".to_string(), 1)
    };
    let pouw_solution = crate::pouw::types::PoUWSolution {
        trained_model_hash: "0".repeat(64),
        accuracy: 0,
        nonce: 0,
        computation_time_ms: 0,
    };

    let new_block = Block::new(
        new_block_index,
        prev_block_hash,
        transactions_to_include,
        0, // Difficulty is part of the task now
        miner_pubkey,
        pouw_task,
        pouw_solution,
    );

    Ok(new_block)
} 