use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    business::BusinessMetrics,
    data_quality::DataQualityMetrics,
    model_quality::ModelQualityMetrics,
    performance::PerformanceMetrics,
    system::SystemMetrics,
};

/// Top-level container aggregating all metric groups for a single ML model version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLMetrics {
    pub model_id: Uuid,
    pub model_version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub performance: PerformanceMetrics,
    pub data_quality: DataQualityMetrics,
    pub model_quality: ModelQualityMetrics,
    pub system: SystemMetrics,
    pub business: Option<BusinessMetrics>,
} 