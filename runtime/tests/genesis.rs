use keygen_lib::generate_keypair;
use runtime::pouw::{generate_task, solve, verify};
use runtime::token::{LedgerError, TokenLedger};

#[test]
fn genesis_and_transfer_with_pouw() -> Result<(), LedgerError> {
    // generate two wallets
    let wallet1 = generate_keypair();
    let wallet2 = generate_keypair();

    // create genesis ledger with one token for wallet1
    let mut ledger = TokenLedger::new();
    ledger.mint(&wallet1.public, 1);
    assert_eq!(ledger.balance(&wallet1.public), 1);

    // PoUW for genesis block
    let genesis_task = generate_task(2, 1);
    let genesis_solution = solve(&genesis_task, 0x0000ffff);
    assert!(verify(&genesis_task, &genesis_solution, 0x0000ffff));

    // transfer token to wallet2
    ledger.transfer(&wallet1.public, &wallet2.public, 1)?;
    assert_eq!(ledger.balance(&wallet1.public), 0);
    assert_eq!(ledger.balance(&wallet2.public), 1);

    // PoUW for transfer block
    let block_task = generate_task(2, 2);
    let block_solution = solve(&block_task, 0x0000ffff);
    assert!(verify(&block_task, &block_solution, 0x0000ffff));

    Ok(())
}
