use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Token ledger error types
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("insufficient stake")]
    InsufficientStake,
    #[error("invalid account: {0}")]
    InvalidAccount(String),
    #[error("transaction failed: {0}")]
    TransactionFailed(String),
}

/// Simple in-memory ledger for balances and staking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenLedger {
    balances: HashMap<String, u64>,
    stakes: HashMap<String, u64>,
    reputations: HashMap<String, i32>,
}

impl TokenLedger {
    /// Create a new empty ledger.
    pub fn new() -> Self {
        Self { balances: HashMap::new(), stakes: HashMap::new(), reputations: HashMap::new() }
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
    pub fn mint(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let balance = self.balances.entry(account.to_string()).or_insert(0);
        *balance = balance.checked_add(amount)
            .ok_or_else(|| LedgerError::TransactionFailed("overflow".to_string()))?;
        Ok(())
    }

    /// Transfer tokens between accounts.
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), LedgerError> {
        let from_balance = self.balances.get(from).copied().unwrap_or(0);
        if from_balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        
        self.balances.insert(from.to_string(), from_balance - amount);
        let to_balance = self.balances.entry(to.to_string()).or_insert(0);
        *to_balance = to_balance.checked_add(amount)
            .ok_or_else(|| LedgerError::TransactionFailed("overflow".to_string()))?;
        
        Ok(())
    }

    /// Permanently remove tokens from an account, reducing total supply.
    pub fn burn(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let bal = self.balances.entry(account.to_string()).or_default();
        if *bal < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        *bal -= amount;
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

    /// Slash staked tokens from `offender` and transfer them to `recipient`.
    pub fn slash(
        &mut self,
        offender: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<(), LedgerError> {
        let off = self.stakes.entry(offender.to_string()).or_default();
        if *off < amount {
            return Err(LedgerError::InsufficientStake);
        }
        *off -= amount;
        let to = self.balances.entry(recipient.to_string()).or_default();
        *to += amount;
        Ok(())
    }

    /// Return the reputation score for `account`.
    pub fn reputation(&self, account: &str) -> i32 {
        *self.reputations.get(account).unwrap_or(&0)
    }

    /// Adjust the reputation for `account` by `delta`.
    pub fn adjust_reputation(&mut self, account: &str, delta: i32) {
        let rep = self.reputations.entry(account.to_string()).or_default();
        *rep += delta;
    }
}
