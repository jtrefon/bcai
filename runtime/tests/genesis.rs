use runtime::pouw::{generate_task, solve, verify};
use runtime::token::{LedgerError, TokenLedger};

#[test]
fn genesis_and_transfer_with_pouw() -> Result<(), LedgerError> {
    // generate two mock wallets (using simple strings for public keys)
    let wallet1_public = "wallet1_public_key".to_string();
    let wallet2_public = "wallet2_public_key".to_string();

    // create genesis ledger with one token for wallet1
    let mut ledger = TokenLedger::new();
    let _ = ledger.mint(&wallet1_public, 1);
    assert_eq!(ledger.balance(&wallet1_public), 1);

    // PoUW for genesis block
    let genesis_task = generate_task(2);
    if let Some(genesis_solution) = solve(&genesis_task) {
        assert!(verify(&genesis_task, genesis_solution));
    }

    // transfer token to wallet2
    ledger.transfer(&wallet1_public, &wallet2_public, 1)?;
    assert_eq!(ledger.balance(&wallet1_public), 0);
    assert_eq!(ledger.balance(&wallet2_public), 1);

    // PoUW for transfer block
    let block_task = generate_task(2);
    if let Some(block_solution) = solve(&block_task) {
        assert!(verify(&block_task, block_solution));
    }

    Ok(())
} 