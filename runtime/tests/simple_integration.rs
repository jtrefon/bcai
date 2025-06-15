//! Simple integration test for BCAI Runtime
//! Tests only the modules that are actually available and working

use runtime::{
    token::{TokenLedger, LedgerError},
    pouw::{Task, solve, verify},
    TensorId, VmConfig, DataType, Blockchain, Transaction,
};

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
    let task = Task {
        difficulty: 10, // Low difficulty
        data: vec![1, 2, 3, 4],
        target: "test".to_string(),
        a: vec![], // Empty for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![1, 2, 3, 4],
    };
    let solution = solve(&task).expect("Failed to solve task");
    assert!(verify(&task, solution));
    
    // Test with different task
    let small_task = Task {
        difficulty: 5, // Even lower difficulty
        data: vec![5, 6, 7, 8],
        target: "small_test".to_string(),
        a: vec![], // Empty for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![5, 6, 7, 8],
    };
    let small_solution = solve(&small_task).expect("Failed to solve small task");
    assert!(verify(&small_task, small_solution));
    
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
    
    let transaction = Transaction {
        from: "alice".to_string(),
        to: "bob".to_string(),
        amount: 100,
        timestamp: 1234567890,
    };
    
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
    let task1 = Task {
        difficulty: 10,
        data: vec![1, 2, 3, 4],
        target: "alice_task".to_string(),
        a: vec![], // Empty for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![1, 2, 3, 4],
    };
    let solution1 = solve(&task1).expect("Failed to solve task1");
    assert!(verify(&task1, solution1));
    
    let task2 = Task {
        difficulty: 15,
        data: vec![5, 6, 7, 8],
        target: "bob_task".to_string(),
        a: vec![], // Empty for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![5, 6, 7, 8],
    };
    let solution2 = solve(&task2).expect("Failed to solve task2");
    assert!(verify(&task2, solution2));
    
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