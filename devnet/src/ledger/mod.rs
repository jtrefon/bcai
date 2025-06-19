use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod actions;

pub use actions::*;

pub const TREASURY: &str = "treasury";

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Account not found: {0}")]
    AccountNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLedger {
    pub balances: HashMap<String, u64>,
}

impl TokenLedger {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    pub fn balance(&self, account: &str) -> u64 {
        self.balances.get(account).copied().unwrap_or(0)
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), LedgerError> {
        if self.balances.get(from).copied().unwrap_or(0) < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        let from_balance = self.balances.get(from).copied().unwrap_or(0);
        let to_balance = self.balances.get(to).copied().unwrap_or(0);
        self.balances.insert(from.to_string(), from_balance - amount);
        self.balances.insert(to.to_string(), to_balance + amount);
        Ok(())
    }
} 