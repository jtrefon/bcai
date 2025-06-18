use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityMetrics {
    pub completeness: f64,
    pub consistency: f64,
    pub validity: f64,
    pub uniqueness: f64,
    pub timeliness: f64,
    pub accuracy: f64,
    pub drift_score: f64,
    pub anomaly_score: f64,
} 