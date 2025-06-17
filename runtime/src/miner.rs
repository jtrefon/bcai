//! The logic for creating and solving a new block.

use crate::blockchain::{block::Block, chain::Blockchain, transaction::Transaction};
use crate::pouw::PoUWTask;
use std::sync::{Arc, Mutex};

/// Mines a new block by taking transactions from the mempool, solving a PoW puzzle,
/// and creating a valid `Block`.
///
/// # Arguments
/// * `miner_pubkey` - The public key of the miner who will receive the block reward.
/// * `blockchain` - A thread-safe reference to the main blockchain instance.
/// * `mempool` - A thread-safe reference to the transaction mempool.
///
/// # Returns
/// A `Result` containing the newly created `Block` or an error.
pub fn mine_block(
    miner_pubkey: String,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
) -> Result<Block, &'static str> {
    let transactions_to_include;
    {
        let mempool = mempool.lock().unwrap();
        let bc_guard = blockchain.lock().unwrap();

        // Validate transactions from mempool against the current state before including them.
        let valid_txs: Vec<Transaction> = mempool
            .iter()
            .filter(|tx| bc_guard.validate_transaction(tx).is_ok())
            .cloned()
            .collect();

        let tx_limit = bc_guard.config.max_transactions_per_block;
        transactions_to_include = valid_txs.into_iter().take(tx_limit).collect();
    }

    let bc = blockchain.lock().unwrap();
    let prev_block = bc.blocks.last().ok_or("Blockchain has no blocks")?;
    let tx_root = Transaction::merkle_root(&transactions_to_include);
    let new_block_index = (prev_block.index + 1) as u32;
    
    // TODO: This should come from a job queue or similar mechanism.
    let pouw_task = PoUWTask::new(
        "model_1".to_string(),
        "dataset_1".to_string(),
        10
    );
    let pouw_solution = pouw_task.solve();

    let new_block = Block::new(
        new_block_index,
        prev_block.hash.clone(),
        transactions_to_include,
        0, // Difficulty is part of the task now, not the block
        miner_pubkey,
        pouw_task,
        pouw_solution,
    );

    Ok(new_block)
} 