use runtime::pouw::{generate_task, solve, solve_profile, verify, PoUWConfig};

#[test]
fn generate_solve_verify() {
    let task = generate_task(4, 42);
    let solution = solve(&task, 0x0000ffff);
    assert!(verify(&task, &solution, 0x0000ffff));
}

#[test]
fn profile_execution_time() {
    let task = generate_task(2, 1);
    let (solution, dur) = solve_profile(&task, 0x0000ffff);
    assert!(verify(&task, &solution, 0x0000ffff));
    // duration should be non-zero
    assert!(dur.as_nanos() > 0);
}

#[test]
fn security_enhanced_verification() {
    let task = generate_task(2, 42);
    let solution = solve(&task, 0x0000ffff);
    
    // Test with relaxed config for testing
    let config = PoUWConfig {
        base_difficulty: 0x0000ffff,
        time_window_secs: 3600, // 1 hour for testing
        max_precompute_advantage: 0, // Allow instant for testing
    };
    
    assert!(runtime::pouw::verify_with_config(&task, &solution, 0x0000ffff, &config));
}
