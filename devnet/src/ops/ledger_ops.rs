use crate::ledger::{actions as ledger_actions, TokenLedger};
use crate::persistence::{load_ledger, save_ledger};
use crate::error::DevnetError;

pub fn init_ledger() -> Result<(), DevnetError> {
    save_ledger(&TokenLedger::new())
}

pub fn mint(account: &str, amount: u64) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    ledger_actions::mint(&mut ledger, account, amount);
    save_ledger(&ledger)
}

pub fn transfer(from: &str, to: &str, amount: u64) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    ledger_actions::transfer(&mut ledger, from, to, amount)?;
    save_ledger(&ledger)
}

pub fn stake(account: &str, amount: u64) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    let _ = ledger_actions::stake(&mut ledger, account, amount);
    save_ledger(&ledger)
}

pub fn unstake(account: &str, amount: u64) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    let _ = ledger_actions::unstake(&mut ledger, account, amount);
    save_ledger(&ledger)
}

pub fn slash(account: &str, amount: u64) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    let _ = ledger_actions::slash(&mut ledger, account, amount);
    save_ledger(&ledger)
}

pub fn burn(account: &str, amount: u64) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    let _ = ledger_actions::burn(&mut ledger, account, amount);
    save_ledger(&ledger)
}

pub fn balance(account: &str) -> Result<(), DevnetError> {
    let ledger = load_ledger()?;
    println!("balance: {} staked: {}", ledger.balance(account), ledger.balance(account));
    Ok(())
}

pub fn reputation(account: &str) -> Result<(), DevnetError> {
    let ledger = load_ledger()?;
    println!("reputation: {}", ledger_actions::reputation(&ledger, account));
    Ok(())
}

pub fn adjust_reputation(account: &str, delta: i32) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    ledger_actions::adjust_reputation(&mut ledger, account, delta);
    save_ledger(&ledger)
} 