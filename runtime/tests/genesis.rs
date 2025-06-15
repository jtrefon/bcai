use runtime::pouw::{Task, generate_task, solve, verify, solve_enhanced, verify_enhanced};
use runtime::token::{LedgerError, TokenLedger};

#[test]
fn genesis_and_transfer_with_pouw() -> Result<(), LedgerError> {
    println!("🧪 Testing genesis and transfer with progressive PoUW complexity");
    
    // generate two mock wallets (using simple strings for public keys)
    let wallet1_public = "wallet1_public_key".to_string();
    let wallet2_public = "wallet2_public_key".to_string();

    // create genesis ledger with one token for wallet1
    let mut ledger = TokenLedger::new();
    let _ = ledger.mint(&wallet1_public, 1000); // Mint more for realistic testing
    assert_eq!(ledger.balance(&wallet1_public), 1000);

    // Phase 1: Simple PoUW for genesis block (fast for CI)
    println!("📋 Phase 1: Simple Genesis PoUW");
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
        println!("✅ Genesis PoUW completed (simple)");
    }

    // Phase 2: Enhanced PoUW with actual computation
    println!("📋 Phase 2: Enhanced Genesis PoUW with Real Computation");
    let enhanced_genesis_task = generate_task(2); // Small matrix for CI
    let enhanced_solution = solve_enhanced(&enhanced_genesis_task, 0x0000ffff);
    assert!(verify_enhanced(&enhanced_genesis_task, &enhanced_solution, 0x0000ffff));
    println!("✅ Enhanced Genesis PoUW completed (real computation)");

    // transfer token to wallet2
    ledger.transfer(&wallet1_public, &wallet2_public, 100)?;
    assert_eq!(ledger.balance(&wallet1_public), 900);
    assert_eq!(ledger.balance(&wallet2_public), 100);

    // Phase 3: Progressive complexity for transaction blocks
    println!("📋 Phase 3: Progressive Transaction PoUW");
    
    // Simple transfer PoUW
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
    
    if let Some(block_solution) = runtime::pouw::solve(&block_task) {
        assert!(runtime::pouw::verify(&block_task, block_solution));
        println!("✅ Transfer PoUW completed (simple)");
    }

    // Enhanced transfer PoUW
    let enhanced_transfer_task = generate_task(3); // Slightly larger for transfer
    let enhanced_transfer_solution = solve_enhanced(&enhanced_transfer_task, 0x0000ffff);
    assert!(verify_enhanced(&enhanced_transfer_task, &enhanced_transfer_solution, 0x0000ffff));
    println!("✅ Enhanced Transfer PoUW completed (real computation)");

    // Phase 4: Demonstrate scalability with multiple small transfers
    println!("📋 Phase 4: Multiple Transaction Processing");
    for i in 1..=5 {
        let transfer_amount = 10;
        if ledger.balance(&wallet1_public) >= transfer_amount {
            ledger.transfer(&wallet1_public, &wallet2_public, transfer_amount)?;
            
            // Simple PoUW for each transaction
            let tx_task = Task {
                difficulty: 1,
                data: vec![i as u8; 4],
                target: format!("tx_{}", i),
                a: vec![],
                b: vec![],
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                challenge: vec![i as u8; 4],
            };
            
            if let Some(tx_solution) = runtime::pouw::solve(&tx_task) {
                assert!(runtime::pouw::verify(&tx_task, tx_solution));
            }
        }
    }
    
    println!("✅ Processed {} micro-transactions", 5);

    // Final verification
    let final_wallet1_balance = ledger.balance(&wallet1_public);
    let final_wallet2_balance = ledger.balance(&wallet2_public);
    let total_balance = final_wallet1_balance + final_wallet2_balance;
    
    assert_eq!(total_balance, 1000, "Token conservation violated");
    println!("✅ Token conservation verified: {} total tokens", total_balance);
    
    println!("🎉 Genesis and progressive PoUW test completed successfully");
    println!("📊 Final state: wallet1({}), wallet2({})", final_wallet1_balance, final_wallet2_balance);
    
    Ok(())
} 