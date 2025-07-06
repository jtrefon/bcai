//! Simple integration test for BCAI Runtime
//! Tests only the modules that are actually available and working

use runtime::{
    token::{TokenLedger, LedgerError},
    pouw::{task::generate_task, solve, verify, types::{PoUWConfig, PoUWTask, Solution as PoUWSolution}},
    tensor_ops::{TensorId, DataType}, // Corrected path
    vm::VmConfig, // Corrected path, assuming VmConfig is in vm
    blockchain::Blockchain, // Corrected path
    blockchain::Transaction, // Corrected path
};

const TEST_POW_DIFFICULTY_SIMPLE: u32 = 0x0FFFFFFF; // Easy for testing

#[test]
fn test_token_ledger_operations() -> Result<(), LedgerError> {
    println!("🧪 Testing token ledger operations");
    
    let mut ledger = TokenLedger::new();
    
    // Test minting
    ledger.mint("alice", 1000);
    ledger.mint("bob", 500);
    
    assert_eq!(ledger.balance("alice"), 1000);
    assert_eq!(ledger.balance("bob"), 500);
    assert_eq!(ledger.balance("charlie"), 0);
    
    // Test transfers
    ledger.transfer("alice", "bob", 100)?;
    assert_eq!(ledger.balance("alice"), 900);
    assert_eq!(ledger.balance("bob"), 600);
    
    // Test insufficient balance
    let result = ledger.transfer("alice", "bob", 2000);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), LedgerError::InsufficientBalance);
    
    println!("✅ Token ledger operations test passed");
    Ok(())
}

#[test]
fn test_pouw_operations() {
    println!("🧪 Testing PoUW operations");
    
    // Use very simple tasks for CI (no matrix multiplication)
    let pouw_config = PoUWConfig::default();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let task = generate_task(10, timestamp); // Use generate_task, 10 as complexity proxy
    let solution = solve(&task, TEST_POW_DIFFICULTY_SIMPLE);
    assert!(verify(&task, &solution, TEST_POW_DIFFICULTY_SIMPLE, &pouw_config));
    
    // Test with different task
    let small_task = generate_task(5, timestamp + 1); // 5 as complexity proxy
    let small_solution = solve(&small_task, TEST_POW_DIFFICULTY_SIMPLE);
    assert!(verify(&small_task, &small_solution, TEST_POW_DIFFICULTY_SIMPLE, &pouw_config));
    
    println!("✅ PoUW operations test passed");
}

#[test]
fn test_basic_types() {
    println!("🧪 Testing basic types");
    
    // Test TensorId
    let tensor_id = TensorId::new(42);
    assert_eq!(tensor_id.as_u64(), 42);
    
    // Test VmConfig
    let config = VmConfig::default();
    assert!(config.max_stack_size > 0);
    assert!(config.max_memory_size > 0);
    
    // Test DataType enum
    let data_types = vec![
        DataType::Float32,
        DataType::Float64,
        DataType::Int32,
        DataType::Int64,
        DataType::Bool,
        DataType::String,
    ];
    
    for dtype in data_types {
        // Serialize and deserialize to test that it works
        let json = serde_json::to_string(&dtype).expect("Failed to serialize DataType");
        let deserialized: DataType = serde_json::from_str(&json).expect("Failed to deserialize DataType");
        assert_eq!(std::mem::discriminant(&dtype), std::mem::discriminant(&deserialized));
    }
    
    println!("✅ Basic types test passed");
}

#[test]
fn test_blockchain_operations() {
    println!("🧪 Testing blockchain operations");
    
    let mut blockchain = Blockchain::new();
    assert_eq!(blockchain.blocks.len(), 0);
    
    // Add some blocks
    blockchain.add_block("Genesis block".to_string());
    blockchain.add_block("Second block".to_string());
    blockchain.add_block("Third block".to_string());
    
    assert_eq!(blockchain.blocks.len(), 3);
    assert_eq!(blockchain.blocks[0].index, 0);
    assert_eq!(blockchain.blocks[1].index, 1);
    assert_eq!(blockchain.blocks[2].index, 2);
    
    assert_eq!(blockchain.blocks[0].data, "Genesis block");
    assert_eq!(blockchain.blocks[1].data, "Second block");
    assert_eq!(blockchain.blocks[2].data, "Third block");
    
    // Check that previous hashes are linked correctly
    assert_eq!(blockchain.blocks[0].previous_hash, "0");
    assert_eq!(blockchain.blocks[1].previous_hash, blockchain.blocks[0].hash);
    assert_eq!(blockchain.blocks[2].previous_hash, blockchain.blocks[1].hash);
    
    println!("✅ Blockchain operations test passed");
}

#[test]
fn test_transaction_serialization() {
    println!("🧪 Testing transaction serialization");
    
    let transaction = Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        0, // Placeholder fee
        0, // Placeholder nonce
    );
    
    // Test JSON serialization
    let json = serde_json::to_string(&transaction).expect("Failed to serialize transaction");
    let deserialized: Transaction = serde_json::from_str(&json).expect("Failed to deserialize transaction");
    
    assert_eq!(transaction.from, deserialized.from);
    assert_eq!(transaction.to, deserialized.to);
    assert_eq!(transaction.amount, deserialized.amount);
    assert_eq!(transaction.timestamp, deserialized.timestamp);
    
    println!("✅ Transaction serialization test passed");
}

#[tokio::test]
async fn test_async_operations() {
    println!("🧪 Testing async operations");
    
    // Test that async operations work in the test environment
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    
    // Test concurrent token operations (simulating multiple nodes)
    let mut ledger1 = TokenLedger::new();
    let mut ledger2 = TokenLedger::new();
    
    ledger1.mint("node1", 1000);
    ledger2.mint("node2", 1000);
    
    assert_eq!(ledger1.balance("node1"), 1000);
    assert_eq!(ledger2.balance("node2"), 1000);
    
    println!("✅ Async operations test passed");
}

#[test]
fn test_comprehensive_integration() {
    println!("🧪 Running comprehensive integration test");
    
    // Simulate a complete workflow
    let mut ledger = TokenLedger::new();
    let mut blockchain = Blockchain::new();
    
    // 1. Initialize system with genesis
    blockchain.add_block("Genesis: System initialized".to_string());
    ledger.mint("system", 10000);
    
    // 2. Create some users and distribute tokens
    ledger.transfer("system", "alice", 1000).expect("Transfer failed");
    ledger.transfer("system", "bob", 1000).expect("Transfer failed");
    ledger.transfer("system", "charlie", 1000).expect("Transfer failed");
    
    // 3. Generate PoUW tasks and solutions (simple for CI)
    let pouw_config_integ = PoUWConfig::default();
    let timestamp_integ = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

    let task1 = generate_task(10, timestamp_integ);
    let solution1 = solve(&task1, TEST_POW_DIFFICULTY_SIMPLE);
    assert!(verify(&task1, &solution1, TEST_POW_DIFFICULTY_SIMPLE, &pouw_config_integ));

    let task2 = generate_task(15, timestamp_integ + 1);
    let solution2 = solve(&task2, TEST_POW_DIFFICULTY_SIMPLE);
    assert!(verify(&task2, &solution2, TEST_POW_DIFFICULTY_SIMPLE, &pouw_config_integ));
    
    // 4. Record transactions on blockchain
    blockchain.add_block(format!("PoUW task completed by alice: {:?}", task1));
    blockchain.add_block(format!("PoUW task completed by bob: {:?}", task2));
    
    // 5. Reward completion
    ledger.transfer("system", "alice", 100).expect("Reward failed");
    ledger.transfer("system", "bob", 100).expect("Reward failed");
    
    // 6. Verify final state
    assert_eq!(ledger.balance("alice"), 1100);
    assert_eq!(ledger.balance("bob"), 1100);
    assert_eq!(ledger.balance("charlie"), 1000);
    assert_eq!(ledger.balance("system"), 6800);
    
    assert_eq!(blockchain.blocks.len(), 4);
    
    println!("✅ Comprehensive integration test passed");
} 