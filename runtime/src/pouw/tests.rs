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
    let bad_solution = types::Solution { nonce: solution.nonce + 1, ..solution };
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

#[test]
fn validator_selection_respects_weights() {
    let validators =
        vec![("alice".to_string(), 100), ("bob".to_string(), 50), ("carol".to_string(), 25)];
    let seed = [1u8; 32];
    let cfg = ValidatorSelectionConfig { min_stake: 10, subset_size: 2 };
    let selected = select_validators(validators.clone(), seed, &cfg);
    assert!(selected.len() <= 2);
    // Ensure that every selected validator is from the original set
    for id in selected {
        assert!(validators.iter().any(|(v, _)| v == &id));
    }
}

#[test]
fn evaluation_sign_and_verify() {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    let key = SigningKey::generate(&mut OsRng);
    let eval = sign_evaluation("task1", 9000, &key);
    assert!(verify_evaluation(&eval));
}

#[test]
fn outlier_detection() {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    let k1 = SigningKey::generate(&mut OsRng);
    let k2 = SigningKey::generate(&mut OsRng);
    let k3 = SigningKey::generate(&mut OsRng);
    let e1 = sign_evaluation("t", 9000, &k1);
    let e2 = sign_evaluation("t", 100, &k2); // extreme outlier
    let e3 = sign_evaluation("t", 8900, &k3);
    let offenders = detect_outliers(&[e1, e2, e3]);
    assert_eq!(offenders.len(), 1);
}

#[test]
fn evaluation_hash_and_tx() {
    use crate::blockchain::transaction::{StorageTx, Transaction};
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use schnorrkel::SecretKey;
    let key = SigningKey::generate(&mut OsRng);
    let sk = SecretKey::generate_with(&mut rand::rngs::OsRng);
    let eval = sign_evaluation("task42", 7777, &key);
    let hash = evaluation_hash(&eval);
    let tx = Transaction::new_pouw_evaluation_signed(&sk, "task42".to_string(), hash.clone(), 0);
    assert!(tx.verify_signature());
    match tx.storage {
        Some(StorageTx::PoUWEvaluationHash { task_id, evaluation_hash }) => {
            assert_eq!(task_id, "task42");
            assert_eq!(evaluation_hash, hash);
        }
        _ => panic!("wrong tx"),
    }
}

#[test]
fn deterministic_algorithms() {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    // Validator selection determinism
    let validators =
        vec![("alice".to_string(), 100), ("bob".to_string(), 50), ("carol".to_string(), 25)];
    let seed = [9u8; 32];
    let cfg = ValidatorSelectionConfig { min_stake: 10, subset_size: 2 };
    let first = select_validators(validators.clone(), seed, &cfg);
    let second = select_validators(validators, seed, &cfg);
    assert_eq!(first, second);

    // Signing determinism
    let key = SigningKey::generate(&mut OsRng);
    let eval1 = sign_evaluation("t", 8000, &key);
    let eval2 = sign_evaluation("t", 8000, &key);
    assert_eq!(eval1, eval2);

    // Solver determinism (nonce and model hash)
    let task = task::generate_task(1, 42);
    let diff = 0x0FFFFFFF;
    let sol1 = solver::solve(&task, diff);
    let sol2 = solver::solve(&task, diff);
    assert_eq!(sol1.nonce, sol2.nonce);
    assert_eq!(sol1.trained_model_hash, sol2.trained_model_hash);
    assert_eq!(sol1.accuracy, sol2.accuracy);
}
