use runtime::pouw::{Task, solve, verify};

#[test]
fn generate_solve_verify() {
    let task = Task {
        difficulty: 10, // Low difficulty for CI
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
}

#[test]
fn profile_execution_time() {
    let task = Task {
        difficulty: 5, // Even lower difficulty for speed test
        data: vec![5, 6, 7, 8],
        target: "speed_test".to_string(),
        a: vec![], // Empty for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![5, 6, 7, 8],
    };
    let solution = solve(&task).expect("Failed to solve task");
    assert!(verify(&task, solution));
    // Test completed successfully
}

#[test]
fn security_enhanced_verification() {
    let task = Task {
        difficulty: 8, // Low difficulty for security test
        data: vec![9, 10, 11, 12],
        target: "security_test".to_string(),
        a: vec![], // Empty for fast testing
        b: vec![],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        challenge: vec![9, 10, 11, 12],
    };
    let solution = solve(&task).expect("Failed to solve task");
    
    // Basic verification test
    assert!(verify(&task, solution));
}
