use runtime::token::{LedgerError, TokenLedger};
use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DevnetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
#[derive(Debug, Serialize, Deserialize)]
struct LedgerWrapper {
    ledger: TokenLedger,
}

pub const LEDGER_FILE: &str = "ledger.json";

pub fn load_ledger() -> Result<TokenLedger, DevnetError> {
    if !std::path::Path::new(LEDGER_FILE).exists() {
        return Ok(TokenLedger::new());
    }
    let data = fs::read_to_string(LEDGER_FILE)?;
    let wrapper: LedgerWrapper = serde_json::from_str(&data)?;
    Ok(wrapper.ledger)
}

pub fn save_ledger(ledger: &TokenLedger) -> Result<(), DevnetError> {
    let data = serde_json::to_string_pretty(&LedgerWrapper { ledger: ledger.clone() })?;
    fs::write(LEDGER_FILE, data)?;
    Ok(())
}

pub fn mint(ledger: &mut TokenLedger, account: &str, amount: u64) {
    ledger.mint(account, amount);
}

pub fn transfer(
    ledger: &mut TokenLedger,
    from: &str,
    to: &str,
    amount: u64,
) -> Result<(), LedgerError> {
    ledger.transfer(from, to, amount)
}

pub fn stake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    ledger.stake(account, amount)
}

pub fn unstake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    ledger.unstake(account, amount)
}
