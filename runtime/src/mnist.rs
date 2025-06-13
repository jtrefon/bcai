use smartcore::dataset::digits::load_dataset;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::linear::logistic_regression::LogisticRegression;

/// Train a logistic regression model on the built-in digits dataset.
/// Returns the training accuracy as a value between 0.0 and 1.0.
pub fn train_digits() -> Result<f32, String> {
    let dataset = load_dataset();
    let mut rows = Vec::with_capacity(dataset.num_samples);
    for r in 0..dataset.num_samples {
        let start = r * dataset.num_features;
        rows.push(dataset.data[start..start + dataset.num_features].to_vec());
    }
    let x = DenseMatrix::from_2d_vec(&rows).map_err(|e| format!("matrix: {e}"))?;
    let y: Vec<i32> = dataset.target.iter().map(|v| *v as i32).collect();
    let lr = LogisticRegression::fit(&x, &y, Default::default())
        .map_err(|e| format!("fit failed: {e}"))?;
    let preds = lr.predict(&x).map_err(|e| format!("predict failed: {e}"))?;
    let correct = preds.iter().zip(&y).filter(|(a, b)| a == b).count();
    Ok(correct as f32 / y.len() as f32)
}
