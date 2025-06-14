use devnet::*;

#[test]
fn mint_and_stake_flow() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    mint(&mut ledger, "alice", 100);
    assert_eq!(ledger.balance("alice"), 100);
    stake(&mut ledger, "alice", 40)?;
    assert_eq!(ledger.balance("alice"), 60);
    // staking simply deducts balance in this simplified model
    unstake(&mut ledger, "alice", 10)?;
    assert_eq!(ledger.balance("alice"), 70);
    Ok(())
}

#[test]
fn train_and_verify_flow() {
    assert!(train_and_verify(2, 1, 0x0000ffff));
}

#[test]
fn job_flow() -> Result<(), JobManagerError> {
    let mut ledger = TokenLedger::new();
    mint(&mut ledger, "alice", 100);
    let mut jobs = Vec::new();
    post_job(&mut jobs, &mut ledger, "alice", "task".into(), 50)?;
    assert_eq!(ledger.balance("alice"), 50);
    assign_job(&mut jobs, "1", "bob")?;
    complete_job(&mut jobs, &mut ledger, "1")?;
    assert_eq!(ledger.balance("bob"), 50);
    Ok(())
}

#[test]
fn slash_and_reputation_flow() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    mint(&mut ledger, "off", 100);
    stake(&mut ledger, "off", 40)?;
    adjust_reputation(&mut ledger, "off", 3);
    assert_eq!(reputation(&ledger, "off"), 3);
    slash(&mut ledger, "off", 25)?;
    assert_eq!(ledger.balance(TREASURY), 25);
    Ok(())
}

#[test]
fn burn_flow() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    mint(&mut ledger, "alice", 60);
    burn(&mut ledger, "alice", 20)?;
    assert_eq!(ledger.balance("alice"), 40);
    assert!(burn(&mut ledger, "alice", 50).is_err());
    Ok(())
}
