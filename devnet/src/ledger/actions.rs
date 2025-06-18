// This file will contain the ledger actions.

use super::{LedgerError, TokenLedger, TREASURY};

pub fn mint(ledger: &mut TokenLedger, account: &str, amount: u64) {
    ledger.balances.insert(account.to_string(), amount);
}

pub fn transfer(
    ledger: &mut TokenLedger,
    from: &str,
    to: &str,
    amount: u64,
) -> Result<(), LedgerError> {
    if ledger.balances.get(from).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(from.to_string(), ledger.balances[from] - amount);
    ledger.balances
        .insert(to.to_string(), ledger.balances.get(to).copied().unwrap_or(0) + amount);
    Ok(())
}

pub fn stake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(account).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(account.to_string(), ledger.balances[account] - amount);
    Ok(())
}

pub fn unstake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(account).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(account.to_string(), ledger.balances[account] + amount);
    Ok(())
}

pub fn slash(ledger: &mut TokenLedger, offender: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(offender).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(offender.to_string(), ledger.balances[offender] - amount);
    ledger.balances.insert(
        TREASURY.to_string(),
        ledger.balances.get(TREASURY).copied().unwrap_or(0) + amount,
    );
    Ok(())
}

pub fn reputation(ledger: &TokenLedger, account: &str) -> i32 {
    ledger.balances.get(account).copied().unwrap_or(0) as i32
}

pub fn adjust_reputation(ledger: &mut TokenLedger, account: &str, delta: i32) {
    let current = ledger.balances.get(account).copied().unwrap_or(0) as i64;
    let new_balance = std::cmp::max(0, current + delta as i64) as u64;
    ledger.balances.insert(account.to_string(), new_balance);
}

pub fn burn(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(account).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(account.to_string(), ledger.balances[account] - amount);
    Ok(())
} 