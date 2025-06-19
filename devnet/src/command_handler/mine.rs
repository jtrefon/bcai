use super::core::{CommandHandler, Mempool};
use runtime::{
    blockchain::{self, validation, Transaction},
    miner,
    p2p_service::WireMessage,
};
use libp2p::gossipsub::IdentTopic;
use std::collections::HashSet;
use std::error::Error;
use tracing::{error, info};

impl CommandHandler {
    /// Mine a new block locally and broadcast it to the network.
    pub async fn mine(&mut self) -> Result<String, Box<dyn Error>> {
        info!("Received 'mine' command.");
        let miner_pubkey = blockchain::constants::DEV_PUBLIC_KEY.to_string();

        let new_block = match miner::mine_block(
            miner_pubkey,
            self.blockchain.clone(),
            self.mempool.clone(),
            self.job_queue.clone(),
        )
        .await
        {
            Ok(block) => block,
            Err(e) => return Ok(format!("Error creating block: {}", e)),
        };
        let block_hash = new_block.hash.clone();
        let num_txs = new_block.transactions.len();
        let total_fees: u64 = new_block.transactions.iter().map(|tx| tx.fee).sum();
        let miner_reward = blockchain::constants::BLOCK_REWARD.saturating_add(total_fees);

        let included_txs = new_block.transactions.clone();
        let block_to_broadcast = new_block.clone();

        match self.blockchain.lock().await.add_block(new_block) {
            Ok(_) => {
                info!("Successfully added locally mined block: {}", block_hash);
                self.prune_mempool(&included_txs).await;
            }
            Err(e) => {
                let err_msg = format!("Failed to add locally mined block: {}", e);
                error!("{}", err_msg);
                return Ok(err_msg);
            }
        }

        let message = WireMessage::Block(block_to_broadcast);
        let topic = IdentTopic::new("bcai_global");
        self.p2p_handle
            .send_message(topic, serde_json::to_vec(&message)?)
            .await?;

        Ok(format!(
            "Success! Mined and broadcast new block #{}:\n  Hash: {}\n  Transactions: {}\n  Miner Reward: {} ({} base + {} fees)",
            self.blockchain.lock().await.blocks.len() - 1,
            block_hash,
            num_txs,
            miner_reward,
            blockchain::constants::BLOCK_REWARD,
            total_fees
        ))
    }

    /// Remove included or invalid transactions from the mempool.
    async fn prune_mempool(&self, included_txs: &[Transaction]) {
        let mut mempool_guard = self.mempool.lock().await;
        let chain_guard = self.blockchain.lock().await;

        let included_hashes: HashSet<_> = included_txs.iter().map(|tx| tx.hash()).collect();
        mempool_guard.retain(|tx| !included_hashes.contains(&tx.hash()));

        mempool_guard.retain(|tx| {
            validation::validate_transaction_stateful(tx, &chain_guard.state).is_ok()
        });

        info!(
            "Mempool pruned. Included: {}. Remaining: {}.",
            included_txs.len(),
            mempool_guard.len()
        );
    }
} 