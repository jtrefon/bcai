use runtime::pouw::{generate_task, solve, verify};

#[test]
fn generate_solve_verify() {
    let task = generate_task(4, 42);
    let solution = solve(&task, 0x0000ffff);
    assert!(verify(&task, &solution, 0x0000ffff));
}
