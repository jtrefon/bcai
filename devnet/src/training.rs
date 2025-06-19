use runtime::{
    evaluator::Evaluator,
    mnist,
    neural_network::{generate_synthetic_data, NeuralNetwork, TrainingMetrics},
    pouw,
    trainer::Trainer,
};

/// Generate a PoUW task, train a solution and verify it.
pub fn train_and_verify(size: usize, seed: u64, difficulty: u32) -> bool {
    let task = pouw::generate_task(size, seed);
    let trainer = Trainer::new("alice");
    let solution = trainer.train(&task, difficulty);
    let evaluator = Evaluator::new("bob");
    evaluator.evaluate(&task, &solution, difficulty)
}

/// Train a logistic regression model on the MNIST digits dataset.
pub fn train_mnist() -> Result<f32, String> {
    mnist::train_digits().map_err(|e| e.to_string())
}

/// Train a simple neural network on synthetic data and return per-epoch metrics.
pub fn train_neural_network(
    layers: Vec<usize>,
    epochs: usize,
    samples: usize,
) -> Result<Vec<TrainingMetrics>, String> {
    if layers.len() < 2 {
        return Err("Neural network must have at least 2 layers (input and output)".into());
    }

    let mut network = NeuralNetwork::new(&layers, 0.01);
    let input_size = layers[0];
    let output_size = *layers.last().unwrap();
    let data = generate_synthetic_data(samples, input_size, output_size);
    Ok(network.train(&data, epochs as u32))
} 