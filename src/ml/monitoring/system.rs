use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub request_count: u64,
    pub error_count: u64,
    pub active_connections: u32,
    pub disk_usage_gb: f64,
    pub network_io_mbps: f64,
    pub model_load_time_ms: f64,
} 