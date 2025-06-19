use crate::error::DevnetError;
use runtime::gpu;
use crate::training;

pub fn mine() -> Result<(), DevnetError> {
    let input = vec![1.0f32, 2.0, 3.0, 4.0];
    match gpu::double_numbers(&input) {
        Ok(res) => println!("mined block with result: {:?}", res),
        Err(e) => println!("gpu task failed: {e}"),
    }
    Ok(())
}

pub fn train_pouw(size: usize, seed: u64, difficulty: u32) -> Result<(), DevnetError> {
    if training::train_and_verify(size, seed, difficulty) {
        println!("training succeeded");
    } else {
        println!("training failed");
    }
    Ok(())
}

pub fn train_mnist() -> Result<(), DevnetError> {
    match training::train_mnist() {
        Ok(acc) => println!("digits training accuracy: {:.2}", acc),
        Err(e) => println!("training failed: {e}"),
    }
    Ok(())
}

pub fn train_neural(layers: Vec<usize>, epochs: usize, samples: usize) -> Result<(), DevnetError> {
    match training::train_neural_network(layers.clone(), epochs, samples) {
        Ok(metrics) => {
            println!("Neural Network Training Results:");
            println!("Architecture: {:?}", layers);
            for metric in metrics {
                println!(
                    "  Epoch {}: loss={:.4}, accuracy={:.3}, time={}ms",
                    metric.epoch, metric.loss, metric.accuracy, metric.training_time_ms
                );
            }
        }
        Err(e) => println!("neural network training failed: {e}"),
    }
    Ok(())
} 