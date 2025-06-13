use devnet::*;
use runtime::token::TokenLedger;

#[test]
fn mint_and_stake_flow() {
    let mut ledger = TokenLedger::new();
    mint(&mut ledger, "alice", 100);
    assert_eq!(ledger.balance("alice"), 100);
    stake(&mut ledger, "alice", 40).unwrap();
    assert_eq!(ledger.balance("alice"), 60);
    assert_eq!(ledger.staked("alice"), 40);
    unstake(&mut ledger, "alice", 10).unwrap();
    assert_eq!(ledger.balance("alice"), 70);
    assert_eq!(ledger.staked("alice"), 30);
}

#[test]
fn train_and_verify_flow() {
    assert!(train_and_verify(2, 1, 0x0000ffff));
}

#[test]
fn job_flow() {
    let mut ledger = TokenLedger::new();
    mint(&mut ledger, "alice", 100);
    let mut jobs = Vec::new();
    post_job(&mut jobs, &mut ledger, "alice", "task".into(), 50).unwrap();
    assert_eq!(ledger.balance("alice"), 50);
    assign_job(&mut jobs, 1, "bob").unwrap();
    complete_job(&mut jobs, &mut ledger, 1).unwrap();
    assert_eq!(ledger.balance("bob"), 50);
}
