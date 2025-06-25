use super::types::SignedEvaluation;

/// Detects outlier evaluators based on median absolute deviation.
pub fn detect_outliers(results: &[SignedEvaluation]) -> Vec<String> {
    if results.is_empty() {
        return vec![];
    }
    let mut accuracies: Vec<u32> = results.iter().map(|r| r.accuracy).collect();
    accuracies.sort_unstable();
    let median = accuracies[accuracies.len() / 2];
    let deviations: Vec<i64> = accuracies.iter().map(|&a| (a as i64 - median as i64).abs()).collect();
    let median_dev = deviations[deviations.len() / 2];
    results
        .iter()
        .filter(|r| ((r.accuracy as i64 - median as i64).abs()) > median_dev * 3)
        .map(|r| r.validator.clone())
        .collect()
}
