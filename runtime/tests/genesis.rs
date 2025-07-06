use runtime::pouw::{
    task::generate_task,
    solve,
    verify,
    types::{PoUWConfig, PoUWTask, Solution as PoUWSolution}
};
use runtime::token::{LedgerError, TokenLedger};

const TEST_POW_DIFFICULTY_GENESIS: u32 = 0x0FFFFFFF; // Easy for testing
const TEST_POW_DIFFICULTY_TX: u32 = 0x00FFFFFF;    // Slightly harder for tx

#[test]
fn genesis_and_transfer_with_pouw() -> Result<(), LedgerError> {
    println!("🧪 Testing genesis and transfer with progressive PoUW complexity");
    let pouw_config = PoUWConfig::default();
    
    // generate two mock wallets (using simple strings for public keys)
    let wallet1_public = "wallet1_public_key".to_string();
    let wallet2_public = "wallet2_public_key".to_string();

    // create genesis ledger with one token for wallet1
    let mut ledger = TokenLedger::new();
    ledger.mint(&wallet1_public, 1000); // Mint more for realistic testing
    assert_eq!(ledger.balance(&wallet1_public), 1000);

    // Phase 1: Simple PoUW for genesis block (fast for CI)
    println!("📋 Phase 1: Simple Genesis PoUW");
    let task_complexity_genesis = 1; // This is the 'epochs' proxy
    let seed_genesis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let genesis_task = generate_task(task_complexity_genesis, seed_genesis);

    let genesis_solution = solve(&genesis_task, TEST_POW_DIFFICULTY_GENESIS);
    assert!(verify(&genesis_task, &genesis_solution, TEST_POW_DIFFICULTY_GENESIS, &pouw_config));
    println!("✅ Genesis PoUW completed (simple)");
    

    // Phase 2: Enhanced PoUW (just means using generate_task with a different complexity)
    println!("📋 Phase 2: Enhanced Genesis PoUW with Real Computation");
    let task_complexity_enhanced_genesis = 2;
    let seed_enhanced_genesis = seed_genesis + 1;
    let enhanced_genesis_task = generate_task(task_complexity_enhanced_genesis, seed_enhanced_genesis);
    let enhanced_solution = solve(&enhanced_genesis_task, TEST_POW_DIFFICULTY_GENESIS);
    assert!(verify(&enhanced_genesis_task, &enhanced_solution, TEST_POW_DIFFICULTY_GENESIS, &pouw_config));
    println!("✅ Enhanced Genesis PoUW completed (real computation)");

    // transfer token to wallet2
    ledger.transfer(&wallet1_public, &wallet2_public, 100)?;
    assert_eq!(ledger.balance(&wallet1_public), 900);
    assert_eq!(ledger.balance(&wallet2_public), 100);

    // Phase 3: Progressive complexity for transaction blocks
    println!("📋 Phase 3: Progressive Transaction PoUW");
    
    // Simple transfer PoUW
    let task_complexity_transfer_simple = 1;
    let seed_transfer_simple = seed_genesis + 2;
    let block_task = generate_task(task_complexity_transfer_simple, seed_transfer_simple);

    let block_solution = solve(&block_task, TEST_POW_DIFFICULTY_TX);
    assert!(verify(&block_task, &block_solution, TEST_POW_DIFFICULTY_TX, &pouw_config));
    println!("✅ Transfer PoUW completed (simple)");
    

    // Enhanced transfer PoUW
    let task_complexity_transfer_enhanced = 3;
    let seed_transfer_enhanced = seed_genesis + 3;
    let enhanced_transfer_task = generate_task(task_complexity_transfer_enhanced, seed_transfer_enhanced);
    let enhanced_transfer_solution = solve(&enhanced_transfer_task, TEST_POW_DIFFICULTY_TX);
    assert!(verify(&enhanced_transfer_task, &enhanced_transfer_solution, TEST_POW_DIFFICULTY_TX, &pouw_config));
    println!("✅ Enhanced Transfer PoUW completed (real computation)");

    // Phase 4: Demonstrate scalability with multiple small transfers
    println!("📋 Phase 4: Multiple Transaction Processing");
    for i in 1..=5 {
        let transfer_amount = 10;
        if ledger.balance(&wallet1_public) >= transfer_amount {
            ledger.transfer(&wallet1_public, &wallet2_public, transfer_amount)?;
            
            // Simple PoUW for each transaction
            let task_complexity_tx = 1;
            let seed_tx = seed_genesis + 3 + i as u64;
            let tx_task = generate_task(task_complexity_tx, seed_tx);
            
            let tx_solution = solve(&tx_task, TEST_POW_DIFFICULTY_TX);
            assert!(verify(&tx_task, &tx_solution, TEST_POW_DIFFICULTY_TX, &pouw_config));
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