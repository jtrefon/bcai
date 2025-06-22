use thiserror::Error;

/// Placeholder ledger error until full token module implemented.
#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Stub ledger error: {0}")]
    Stub(String),
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

    pub fn transfer(&self, _from: &str, _to: &str, _amount: u64) -> Result<(), LedgerError> {
        // TODO: Implement real ledger logic.
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