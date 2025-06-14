use runtime::pouw::{generate_task, solve, solve_profile, verify};

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
