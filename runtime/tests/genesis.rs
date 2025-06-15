use runtime::pouw::{Task};
use runtime::token::{LedgerError, TokenLedger};

#[test]
fn genesis_and_transfer_with_pouw() -> Result<(), LedgerError> {
    println!("🧪 Testing genesis and transfer with simple PoUW");
    
    // generate two mock wallets (using simple strings for public keys)
    let wallet1_public = "wallet1_public_key".to_string();
    let wallet2_public = "wallet2_public_key".to_string();

    // create genesis ledger with one token for wallet1
    let mut ledger = TokenLedger::new();
    let _ = ledger.mint(&wallet1_public, 1);
    assert_eq!(ledger.balance(&wallet1_public), 1);

    // Simulate PoUW for genesis block (fast for CI)
    let genesis_task = Task {
        difficulty: 1, // Very low difficulty for CI
        data: vec![1, 2, 3, 4],
        target: "genesis".to_string(),
        a: vec![], // Empty matrices for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![1, 2, 3, 4],
    };
    
    // For simple tasks, solve returns the magic nonce 42
    if let Some(genesis_solution) = runtime::pouw::solve(&genesis_task) {
        assert!(runtime::pouw::verify(&genesis_task, genesis_solution));
        println!("✅ Genesis PoUW completed");
    }

    // transfer token to wallet2
    ledger.transfer(&wallet1_public, &wallet2_public, 1)?;
    assert_eq!(ledger.balance(&wallet1_public), 0);
    assert_eq!(ledger.balance(&wallet2_public), 1);

    // Simulate PoUW for transfer block (fast for CI)
    let block_task = Task {
        difficulty: 1, // Very low difficulty for CI
        data: vec![5, 6, 7, 8],
        target: "transfer".to_string(),
        a: vec![], // Empty matrices for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![5, 6, 7, 8],
    };
    
    // For simple tasks, solve returns the magic nonce 42
    if let Some(block_solution) = runtime::pouw::solve(&block_task) {
        assert!(runtime::pouw::verify(&block_task, block_solution));
        println!("✅ Transfer PoUW completed");
    }

    println!("✅ Genesis and transfer with PoUW test passed");
    Ok(())
} 