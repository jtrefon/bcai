use runtime::mnist::train_digits;

#[test]
fn mnist_training_accuracy() {
    let acc = train_digits().expect("training");
    assert!(acc > 0.8, "accuracy too low: {acc}");
}
