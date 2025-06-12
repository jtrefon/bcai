use runtime::job_manager::{JobManager, JobManagerError};
use runtime::token::TokenLedger;

#[test]
fn posting_and_completing_job() {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 100);
    let mut jm = JobManager::new(ledger);
    // post job
    jm.post_job("alice", "test".into(), 50).unwrap();
    assert_eq!(jm.jobs().len(), 1);
    assert_eq!(jm.ledger().balance("alice"), 50);
    assert_eq!(jm.ledger().balance("escrow"), 50);

    // assign and complete
    jm.assign_job(1, "bob").unwrap();
    jm.complete_job(1).unwrap();
    assert_eq!(jm.ledger().balance("bob"), 50);
    assert_eq!(jm.ledger().balance("escrow"), 0);
}

#[test]
fn insufficient_balance_fails() {
    let jm = &mut JobManager::default();
    assert_eq!(
        jm.post_job("alice", "test".into(), 10).unwrap_err(),
        JobManagerError::InsufficientBalance
    );
}
