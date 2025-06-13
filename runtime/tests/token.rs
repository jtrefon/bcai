use runtime::token::{LedgerError, TokenLedger};

#[test]
fn mint_and_transfer() {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 100);
    assert_eq!(ledger.balance("alice"), 100);
    ledger.transfer("alice", "bob", 40).unwrap();
    assert_eq!(ledger.balance("alice"), 60);
    assert_eq!(ledger.balance("bob"), 40);
}

#[test]
fn stake_and_unstake() {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 50);
    ledger.stake("alice", 30).unwrap();
    assert_eq!(ledger.balance("alice"), 20);
    assert_eq!(ledger.staked("alice"), 30);
    ledger.unstake("alice", 10).unwrap();
    assert_eq!(ledger.balance("alice"), 30);
    assert_eq!(ledger.staked("alice"), 20);
}

#[test]
fn staking_errors() {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 10);
    assert_eq!(ledger.stake("alice", 20).unwrap_err(), LedgerError::InsufficientBalance);
    ledger.stake("alice", 10).unwrap();
    assert_eq!(ledger.unstake("alice", 20).unwrap_err(), LedgerError::InsufficientStake);
}

#[test]
fn slashing_and_reputation() {
    let mut ledger = TokenLedger::new();
    ledger.mint("offender", 50);
    ledger.stake("offender", 30).unwrap();
    assert_eq!(ledger.staked("offender"), 30);
    ledger.adjust_reputation("offender", 5);
    assert_eq!(ledger.reputation("offender"), 5);
    ledger.slash("offender", "treasury", 20).unwrap();
    assert_eq!(ledger.staked("offender"), 10);
    assert_eq!(ledger.balance("treasury"), 20);
    ledger.adjust_reputation("offender", -3);
    assert_eq!(ledger.reputation("offender"), 2);
}
