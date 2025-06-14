use runtime::token::{LedgerError, TokenLedger};

#[test]
fn mint_and_transfer() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 100);
    assert_eq!(ledger.balance("alice"), 100);
    ledger.transfer("alice", "bob", 40)?;
    assert_eq!(ledger.balance("alice"), 60);
    assert_eq!(ledger.balance("bob"), 40);
    Ok(())
}

#[test]
fn stake_and_unstake() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 50);
    ledger.stake("alice", 30)?;
    assert_eq!(ledger.balance("alice"), 20);
    assert_eq!(ledger.staked("alice"), 30);
    ledger.unstake("alice", 10)?;
    assert_eq!(ledger.balance("alice"), 30);
    assert_eq!(ledger.staked("alice"), 20);
    Ok(())
}

#[test]
fn staking_errors() {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 10);
    assert_eq!(ledger.stake("alice", 20).unwrap_err(), LedgerError::InsufficientBalance);
    ledger.stake("alice", 10).expect("stake succeeds");
    assert_eq!(ledger.unstake("alice", 20).unwrap_err(), LedgerError::InsufficientStake);
}

#[test]
fn slashing_and_reputation() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    ledger.mint("offender", 50);
    ledger.stake("offender", 30)?;
    assert_eq!(ledger.staked("offender"), 30);
    ledger.adjust_reputation("offender", 5);
    assert_eq!(ledger.reputation("offender"), 5);
    ledger.slash("offender", "treasury", 20)?;
    assert_eq!(ledger.staked("offender"), 10);
    assert_eq!(ledger.balance("treasury"), 20);
    ledger.adjust_reputation("offender", -3);
    assert_eq!(ledger.reputation("offender"), 2);
    Ok(())
}

#[test]
fn burn_tokens() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 50);
    ledger.burn("alice", 20)?;
    assert_eq!(ledger.balance("alice"), 30);
    assert_eq!(ledger.burn("alice", 40).unwrap_err(), LedgerError::InsufficientBalance);
    Ok(())
}
