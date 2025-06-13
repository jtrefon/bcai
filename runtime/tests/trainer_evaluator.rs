use runtime::evaluator::Evaluator;
use runtime::pouw::generate_task;
use runtime::trainer::Trainer;

#[test]
fn trainer_and_evaluator_flow() {
    let task = generate_task(2, 1);
    let trainer = Trainer::new("alice");
    let solution = trainer.train(&task, 0x0000ffff);
    let evaluator = Evaluator::new("bob");
    assert!(evaluator.evaluate(&task, &solution, 0x0000ffff));
}
