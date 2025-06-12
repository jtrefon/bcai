use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("insufficient stake")]
    InsufficientStake,
}

/// Simple in-memory ledger for balances and staking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenLedger {
    balances: HashMap<String, u64>,
    stakes: HashMap<String, u64>,
}

impl TokenLedger {
    /// Create a new empty ledger.
    pub fn new() -> Self {
        Self { balances: HashMap::new(), stakes: HashMap::new() }
    }

    /// Return the free token balance for `account`.
    pub fn balance(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }

    /// Return the staked token balance for `account`.
    pub fn staked(&self, account: &str) -> u64 {
        *self.stakes.get(account).unwrap_or(&0)
    }

    /// Mint new tokens into `account`.
    pub fn mint(&mut self, account: &str, amount: u64) {
        let entry = self.balances.entry(account.to_string()).or_default();
        *entry += amount;
    }

    /// Transfer tokens between accounts.
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), LedgerError> {
        let from_bal = self.balances.entry(from.to_string()).or_default();
        if *from_bal < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        *from_bal -= amount;
        let to_bal = self.balances.entry(to.to_string()).or_default();
        *to_bal += amount;
        Ok(())
    }

    /// Stake tokens from the caller's balance.
    pub fn stake(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let bal = self.balances.entry(account.to_string()).or_default();
        if *bal < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        *bal -= amount;
        let st = self.stakes.entry(account.to_string()).or_default();
        *st += amount;
        Ok(())
    }

    /// Unstake tokens back to the caller's balance.
    pub fn unstake(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let st = self.stakes.entry(account.to_string()).or_default();
        if *st < amount {
            return Err(LedgerError::InsufficientStake);
        }
        *st -= amount;
        let bal = self.balances.entry(account.to_string()).or_default();
        *bal += amount;
        Ok(())
    }
}
