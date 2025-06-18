use super::*;

#[test]
fn adaptive_difficulty_adjustment() {
    let initial_difficulty = 1000;
    let target_time = 60;

    // Slower than target -> decrease difficulty (higher number)
    let new_diff_slower = calculate_adaptive_difficulty(initial_difficulty, target_time, 90);
    assert!(new_diff_slower > initial_difficulty);

    // Faster than target -> increase difficulty (lower number)
    let new_diff_faster = calculate_adaptive_difficulty(initial_difficulty, target_time, 30);
    assert!(new_diff_faster < initial_difficulty);

    // At target -> no change
    let new_diff_same = calculate_adaptive_difficulty(initial_difficulty, target_time, 60);
    assert_eq!(new_diff_same, initial_difficulty);
}

#[test]
fn solve_and_verify_flow() {
    let config = types::PoUWConfig::default();
    let task = task::generate_task(1, 12345);
    let difficulty = 0x0FFFFFFF; // Relatively easy difficulty for testing

    let solution = solver::solve(&task, difficulty);

    // Verify the solution
    assert!(verifier::verify(&task, &solution, difficulty, &config));

    // A solution with a wrong nonce should fail
    let bad_solution = types::Solution {
        nonce: solution.nonce + 1,
        ..solution
    };
    assert!(!verifier::verify(&task, &bad_solution, difficulty, &config));
}

#[test]
fn timestamp_validation() {
    let config = types::PoUWConfig::default();
    let mut task = task::generate_task(1, 12345);
    let difficulty = 0x0FFFFFFF;
    
    // Stale timestamp
    task.timestamp = 0;
    let solution = solver::solve(&task, difficulty);
    assert!(!verifier::verify(&task, &solution, difficulty, &config));
} 