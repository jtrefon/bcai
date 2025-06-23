use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{chain::BlockchainError, transaction::{Transaction, StorageTx}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    /// Mapping from a public key (hex-encoded) to an account balance.
    pub balances: HashMap<String, u64>,
    /// Mapping from a public key (hex-encoded) to the next valid nonce.
    pub nonces: HashMap<String, u64>,
    /// Latest metrics for known storage nodes.
    pub node_metrics: HashMap<String, crate::distributed_storage::allocation::NodeMetrics>,
}

impl State {
    /// Creates a new, empty state.
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            nonces: HashMap::new(),
            node_metrics: HashMap::new(),
        }
    }

    /// Applies a transaction to the state, updating balances and nonces.
    /// This function assumes the transaction has already been validated.
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), BlockchainError> {
        // Determine total cost depending on payload type.
        let total_cost: u128 = if let Some(ref payload) = tx.storage {
            match payload {
                crate::blockchain::transaction::StorageTx::StoreFile { price, .. } => {
                    (*price as u128) + tx.fee as u128
                }
                crate::blockchain::transaction::StorageTx::RewardHolding { .. } => {
                    // Reward payouts are negative cost to sender (node). For now treat as zero cost.
                    tx.fee as u128
                }
                crate::blockchain::transaction::StorageTx::UpdateMetrics { metrics } => {
                    // Apply metrics update without cost; update node_metrics map.
                    for m in metrics {
                        self.node_metrics.insert(m.node_id.clone(), m.clone());
                    }
                    0u128
                }
            }
        } else {
            (tx.amount as u128) + tx.fee as u128
        };
        let total_cost_u64 = total_cost as u64;

        // Credit receiver for value transfers only
        if tx.storage.is_none() {
            let receiver_balance = self.balances.entry(tx.to.clone()).or_insert(0);
            *receiver_balance = receiver_balance.checked_add(tx.amount).ok_or_else(|| {
                BlockchainError::TransactionValidationError("Balance overflow on receiver".to_string())
            })?;
        }

        // Update sender's nonce
        let nonce = self.nonces.entry(tx.from.clone()).or_insert(0);
        *nonce = tx.nonce; // The new nonce is the one from the applied transaction

        if total_cost_u64 > 0 {
            let sender_balance = self
                .balances
                .get_mut(&tx.from)
                .ok_or_else(|| BlockchainError::TransactionValidationError("Sender not found".to_string()))?;

            if *sender_balance < total_cost_u64 {
                return Err(BlockchainError::TransactionValidationError(
                    "Insufficient funds".to_string(),
                ));
            }
            *sender_balance -= total_cost_u64;
        }

        Ok(())
    }

    /// Gets the balance for a given public key.
    pub fn get_balance(&self, pubkey: &str) -> u64 {
        self.balances.get(pubkey).cloned().unwrap_or(0)
    }

    /// Gets the current nonce for a given public key.
    pub fn get_nonce(&self, pubkey: &str) -> u64 {
        self.nonces.get(pubkey).cloned().unwrap_or(0)
    }

    /// A special function to directly set a balance, used for genesis block creation.
    pub fn set_balance(&mut self, pubkey: &str, amount: u64) {
        self.balances.insert(pubkey.to_string(), amount);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Public alias exposed to other modules for convenience.
pub type BlockchainState = State; 