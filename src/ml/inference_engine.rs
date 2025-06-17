use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::time::Duration;

// --- Core Data Structures for Inference ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEndpoint {
    pub id: Uuid,
    pub model_id: Uuid,
    pub model_version: String,
    pub status: EndpointStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub replicas: u32,
    pub resources: ResourceRequirements,
    pub load_balancing_strategy: LoadBalancingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub input_data: serde_json::Value,
    pub parameters: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: Uuid,
    pub output_data: serde_json::Value,
    pub error: Option<String>,
    pub performance_metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu: f64,
    pub memory_gb: f64,
    pub gpu: Option<GpuRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRequirement {
    pub gpu_type: String,
    pub memory_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointStatus {
    Creating,
    Active,
    Updating,
    Inactive,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub request_count: u64,
    pub error_count: u64,
    pub average_latency_ms: f64,
    pub throughput_rps: f64,
}

// NOTE: Removed placeholder implementation structs:
// - InferenceEngine
// - LoadBalancer
// - BatchProcessor
// - MetricsCollector
// This file now only defines the data models for the inference engine. 