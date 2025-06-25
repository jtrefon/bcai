use thiserror::Error;

/// Errors that can occur when operating on the token ledger.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    /// Account has insufficient spendable balance for the requested operation.
    #[error("insufficient balance")]
    InsufficientBalance,
    /// Account has insufficient staked balance.
    #[error("insufficient staked amount")]
    InsufficientStaked,
}

/// Minimal token ledger stub sufficient for compilation.
#[derive(Debug, Clone)]
pub struct TokenLedger {
    balances: std::collections::HashMap<String, u64>,
    staked: std::collections::HashMap<String, u64>,
}

impl TokenLedger {
    pub fn new() -> Self {
        Self {
            balances: std::collections::HashMap::new(),
            staked: std::collections::HashMap::new(),
        }
    }

    /// Transfers spendable balance between two accounts.
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), LedgerError> {
        let from_balance = self.balances.entry(from.to_string()).or_default();
        if *from_balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        *from_balance -= amount;
        *self.balances.entry(to.to_string()).or_default() += amount;
        Ok(())
    }

    /// Stakes a given amount of tokens from the account's balance.
    pub fn stake(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let balance = self.balances.entry(account.to_string()).or_default();
        if *balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        *balance -= amount;
        *self.staked.entry(account.to_string()).or_default() += amount;
        Ok(())
    }

    /// Unstakes tokens, returning them to the account's balance.
    pub fn unstake(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let staked = self.staked.entry(account.to_string()).or_default();
        if *staked < amount {
            return Err(LedgerError::InsufficientStaked);
        }
        *staked -= amount;
        *self.balances.entry(account.to_string()).or_default() += amount;
        Ok(())
    }

    /// Slashes staked tokens as a penalty.
    pub fn slash(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let staked = self.staked.entry(account.to_string()).or_default();
        if *staked < amount {
            *staked = 0;
        } else {
            *staked -= amount;
        }
        Ok(())
    }

    pub fn mint(&mut self, account: &str, amount: u64) {
        *self.balances.entry(account.to_string()).or_default() += amount;
    }

    pub fn balance(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }

    pub fn staked(&self, account: &str) -> u64 {
        *self.staked.get(account).unwrap_or(&0)
    }

    pub fn penalize(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let balance = self.balances.entry(account.to_string()).or_default();
        if *balance < amount {
            *balance = 0;
        } else {
            *balance -= amount;
        }
        Ok(())
    }
}
