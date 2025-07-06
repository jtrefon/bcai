use runtime::pouw::{solve, verify, task as pouw_task, types as pouw_types};

const TEST_DIFFICULTY_TARGET: u32 = 0x0FFFFFFF; // Easy difficulty for testing
const TEST_SEED_1: u64 = 12345;
const TEST_SEED_2: u64 = 67890;
const TEST_SEED_3: u64 = 13579;

#[test]
fn generate_solve_verify() {
    // The first argument to generate_task is a proxy for epochs/complexity of "useful work"
    // The second is a seed for RNG to make task generation deterministic.
    let task = pouw_task::generate_task(10, TEST_SEED_1);
    let config = pouw_types::PoUWConfig::default();

    let solution = solve(&task, TEST_DIFFICULTY_TARGET);
    assert!(verify(&task, &solution, TEST_DIFFICULTY_TARGET, &config));
}

#[test]
fn profile_execution_time() {
    let task = pouw_task::generate_task(5, TEST_SEED_2); // Lower "useful work" complexity
    let config = pouw_types::PoUWConfig::default();

    let start_time = std::time::Instant::now();
    let solution = solve(&task, TEST_DIFFICULTY_TARGET);
    let duration = start_time.elapsed();

    assert!(verify(&task, &solution, TEST_DIFFICULTY_TARGET, &config));
    println!("Profile execution time: {:?}", duration);
    // Test completed successfully (actual time depends on machine, just ensure it runs)
}

#[test]
fn security_enhanced_verification() {
    // This test name might be a misnomer now, it's similar to generate_solve_verify
    // but uses different parameters.
    let task = pouw_task::generate_task(8, TEST_SEED_3);
    let config = pouw_types::PoUWConfig::default();

    let solution = solve(&task, TEST_DIFFICULTY_TARGET);
    
    // Basic verification test
    assert!(verify(&task, &solution, TEST_DIFFICULTY_TARGET, &config));
}
