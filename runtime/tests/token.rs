use runtime::{LedgerError, TokenLedger};

#[test]
fn mint_and_transfer() -> Result<(), LedgerError> {
    let mut ledger = TokenLedger::new();
    ledger.mint("alice", 100)?;
    assert_eq!(ledger.balance("alice"), 100);
    ledger.transfer("alice", "bob", 40)?;
    assert_eq!(ledger.balance("alice"), 60);
    assert_eq!(ledger.balance("bob"), 40);
    Ok(())
}
