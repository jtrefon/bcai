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
