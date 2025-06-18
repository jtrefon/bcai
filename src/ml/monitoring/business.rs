use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub revenue_impact: f64,
    pub cost_savings: f64,
    pub user_satisfaction: f64,
    pub conversion_rate: f64,
    pub custom_kpis: HashMap<String, f64>,
}