use crate::blockchain::state::BlockchainState;
use std::collections::HashMap;

pub struct AccountManager;

impl AccountManager {
    /// Gets the current nonce for an account
    pub fn get_nonce(account_nonces: &HashMap<String, u64>, pubkey_hex: &str) -> u64 {
        *account_nonces.get(pubkey_hex).unwrap_or(&0)
    }

    /// Gets the current balance for an account
    pub fn get_balance(state: &BlockchainState, pubkey_hex: &str) -> u64 {
        *state.balances.get(pubkey_hex).unwrap_or(&0)
    }
} 