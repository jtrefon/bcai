use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, Semaphore};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: Uuid,
    pub model_id: Uuid,
    pub model_version: Option<String>,
    pub input_data: serde_json::Value,
    pub preprocessing_config: Option<PreprocessingConfig>,
    pub postprocessing_config: Option<PostprocessingConfig>,
    pub priority: RequestPriority,
    pub timeout: Duration,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: Uuid,
    pub model_id: Uuid,
    pub model_version: String,
    pub predictions: serde_json::Value,
    pub confidence_scores: Option<Vec<f64>>,
    pub processing_time_ms: f64,
    pub metadata: HashMap<String, String>,
    pub status: InferenceStatus,
    pub error_message: Option<String>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InferenceStatus {
    Success,
    Failed,
    Timeout,
    ModelNotFound,
    InvalidInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingConfig {
    pub steps: Vec<PreprocessingStep>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostprocessingConfig {
    pub steps: Vec<PostprocessingStep>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreprocessingStep {
    Normalize,
    Standardize,
    Resize,
    Tokenize,
    Encode,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostprocessingStep {
    Softmax,
    Argmax,
    Threshold,
    Decode,
    Format,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEndpoint {
    pub id: Uuid,
    pub model_id: Uuid,
    pub model_version: String,
    pub endpoint_url: String,
    pub status: EndpointStatus,
    pub deployment_config: DeploymentConfig,
    pub performance_config: PerformanceConfig,
    pub health_check_config: HealthCheckConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub metrics: EndpointMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EndpointStatus {
    Deploying,
    Active,
    Inactive,
    Failed,
    Scaling,
    Updating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub replicas: u32,
    pub resource_requirements: ResourceRequirements,
    pub auto_scaling: AutoScalingConfig,
    pub load_balancing: LoadBalancingConfig,
    pub canary_deployment: Option<CanaryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f64,
    pub memory_gb: f64,
    pub gpu_count: u32,
    pub gpu_memory_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    pub enabled: bool,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
    pub target_memory_utilization: f64,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub cooldown_period: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub strategy: LoadBalancingStrategy,
    pub health_check_enabled: bool,
    pub sticky_sessions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    IPHash,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub enabled: bool,
    pub traffic_percentage: f64,
    pub success_criteria: SuccessCriteria,
    pub rollback_criteria: RollbackCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub min_success_rate: f64,
    pub max_latency_ms: f64,
    pub min_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackCriteria {
    pub max_error_rate: f64,
    pub max_latency_ms: f64,
    pub min_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub batch_size: u32,
    pub max_batch_delay_ms: u32,
    pub timeout_ms: u32,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u32,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Custom(HashMap<String, serde_json::Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub requests_per_second: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub error_rate: f64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub gpu_utilization: f64,
    pub active_connections: u32,
    pub queue_size: u32,
    pub cache_hit_rate: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct InferenceEngine {
    endpoints: Arc<RwLock<HashMap<Uuid, ModelEndpoint>>>,
    request_queue: Arc<RwLock<Vec<InferenceRequest>>>,
    response_cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    model_registry: Arc<dyn ModelRegistry + Send + Sync>,
    load_balancer: Arc<LoadBalancer>,
    batch_processor: Arc<BatchProcessor>,
    metrics_collector: Arc<MetricsCollector>,
    rate_limiter: Arc<Semaphore>,
    event_bus: mpsc::UnboundedSender<InferenceEvent>,
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub response: InferenceResponse,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait ModelRegistry {
    async fn get_model(&self, model_id: Uuid, version: Option<String>) -> Result<ModelInfo, String>;
    async fn load_model(&self, model_id: Uuid, version: String) -> Result<LoadedModel, String>;
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub framework: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub preprocessing_config: Option<PreprocessingConfig>,
    pub postprocessing_config: Option<PostprocessingConfig>,
}

pub struct LoadedModel {
    pub info: ModelInfo,
    pub model_data: Vec<u8>, // Simplified - in practice would be framework-specific
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum InferenceEvent {
    RequestReceived(Uuid),
    RequestProcessed(Uuid),
    RequestFailed(Uuid, String),
    EndpointDeployed(Uuid),
    EndpointScaled(Uuid, u32),
    ModelLoaded(Uuid, String),
    HealthCheckFailed(Uuid),
}

pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    endpoint_health: Arc<RwLock<HashMap<Uuid, bool>>>,
}

pub struct BatchProcessor {
    max_batch_size: usize,
    max_wait_time: Duration,
    pending_requests: Arc<RwLock<Vec<InferenceRequest>>>,
}

pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<Uuid, EndpointMetrics>>>,
    collection_interval: Duration,
}

impl InferenceEngine {
    pub fn new(
        model_registry: Arc<dyn ModelRegistry + Send + Sync>,
        max_concurrent_requests: usize,
    ) -> Self {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        
        Self {
            endpoints: Arc::new(RwLock::new(HashMap::new())),
            request_queue: Arc::new(RwLock::new(Vec::new())),
            response_cache: Arc::new(RwLock::new(HashMap::new())),
            model_registry,
            load_balancer: Arc::new(LoadBalancer::new(LoadBalancingStrategy::RoundRobin)),
            batch_processor: Arc::new(BatchProcessor::new(32, Duration::from_millis(10))),
            metrics_collector: Arc::new(MetricsCollector::new(Duration::from_secs(1))),
            rate_limiter: Arc::new(Semaphore::new(max_concurrent_requests)),
            event_bus: event_tx,
        }
    }

    pub async fn deploy_model(&self, mut endpoint: ModelEndpoint) -> Result<Uuid, String> {
        let endpoint_id = endpoint.id;
        
        // Validate deployment configuration
        self.validate_deployment_config(&endpoint.deployment_config)?;
        
        // Load model from registry
        let model_info = self.model_registry.get_model(endpoint.model_id, Some(endpoint.model_version.clone())).await?;
        
        endpoint.status = EndpointStatus::Deploying;
        endpoint.created_at = chrono::Utc::now();
        endpoint.last_updated = chrono::Utc::now();

        // Store endpoint
        {
            let mut endpoints = self.endpoints.write().await;
            endpoints.insert(endpoint_id, endpoint);
        }

        // Simulate deployment process
        self.perform_deployment(endpoint_id).await?;

        // Update status to active
        {
            let mut endpoints = self.endpoints.write().await;
            if let Some(endpoint) = endpoints.get_mut(&endpoint_id) {
                endpoint.status = EndpointStatus::Active;
                endpoint.last_updated = chrono::Utc::now();
            }
        }

        let _ = self.event_bus.send(InferenceEvent::EndpointDeployed(endpoint_id));
        Ok(endpoint_id)
    }

    pub async fn predict(&self, request: InferenceRequest) -> Result<InferenceResponse, String> {
        let start_time = Instant::now();
        let request_id = request.id;

        // Acquire rate limiting permit
        let _permit = self.rate_limiter.acquire().await.map_err(|_| "Rate limit exceeded")?;

        let _ = self.event_bus.send(InferenceEvent::RequestReceived(request_id));

        // Check cache first
        if let Some(cached_response) = self.check_cache(&request).await {
            return Ok(cached_response);
        }

        // Find appropriate endpoint
        let endpoint = self.select_endpoint(&request).await?;

        // Validate input
        self.validate_input(&request, &endpoint).await?;

        // Preprocess input
        let preprocessed_input = self.preprocess_input(&request).await?;

        // Perform inference
        let raw_output = self.perform_inference(&endpoint, &preprocessed_input).await?;

        // Postprocess output
        let processed_output = self.postprocess_output(&request, &raw_output).await?;

        let processing_time = start_time.elapsed().as_millis() as f64;

        let response = InferenceResponse {
            request_id,
            model_id: endpoint.model_id,
            model_version: endpoint.model_version.clone(),
            predictions: processed_output,
            confidence_scores: None, // Would be computed based on model output
            processing_time_ms: processing_time,
            metadata: HashMap::new(),
            status: InferenceStatus::Success,
            error_message: None,
            completed_at: chrono::Utc::now(),
        };

        // Cache response if enabled
        self.cache_response(&request, &response).await;

        // Update metrics
        self.update_endpoint_metrics(endpoint.id, processing_time).await;

        let _ = self.event_bus.send(InferenceEvent::RequestProcessed(request_id));
        Ok(response)
    }

    async fn perform_deployment(&self, endpoint_id: Uuid) -> Result<(), String> {
        // Simulate deployment process
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn select_endpoint(&self, request: &InferenceRequest) -> Result<ModelEndpoint, String> {
        let endpoints = self.endpoints.read().await;
        
        // Find endpoints for the requested model
        let matching_endpoints: Vec<&ModelEndpoint> = endpoints.values()
            .filter(|e| e.model_id == request.model_id && e.status == EndpointStatus::Active)
            .filter(|e| {
                if let Some(version) = &request.model_version {
                    &e.model_version == version
                } else {
                    true // Use any version if not specified
                }
            })
            .collect();

        if matching_endpoints.is_empty() {
            return Err("No active endpoints found for model".to_string());
        }

        // Use load balancer to select endpoint
        let selected_endpoint = self.load_balancer.select_endpoint(&matching_endpoints).await?;
        Ok(selected_endpoint.clone())
    }

    async fn check_cache(&self, request: &InferenceRequest) -> Option<InferenceResponse> {
        let cache_key = self.generate_cache_key(request);
        let cache = self.response_cache.read().await;
        
        if let Some(cached) = cache.get(&cache_key) {
            if cached.expires_at > chrono::Utc::now() {
                return Some(cached.response.clone());
            }
        }
        
        None
    }

    async fn cache_response(&self, request: &InferenceRequest, response: &InferenceResponse) {
        let cache_key = self.generate_cache_key(request);
        let cached_response = CachedResponse {
            response: response.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(300), // 5 minutes TTL
        };
        
        let mut cache = self.response_cache.write().await;
        cache.insert(cache_key, cached_response);
    }

    fn generate_cache_key(&self, request: &InferenceRequest) -> String {
        // Simple cache key generation - in practice would use proper hashing
        format!("{}:{}:{}", request.model_id, 
                request.model_version.as_deref().unwrap_or("latest"),
                serde_json::to_string(&request.input_data).unwrap_or_default())
    }

    async fn validate_input(&self, request: &InferenceRequest, endpoint: &ModelEndpoint) -> Result<(), String> {
        // Simulate input validation
        if request.input_data.is_null() {
            return Err("Input data cannot be null".to_string());
        }
        Ok(())
    }

    async fn preprocess_input(&self, request: &InferenceRequest) -> Result<serde_json::Value, String> {
        // Simulate preprocessing
        if let Some(config) = &request.preprocessing_config {
            // Apply preprocessing steps
            let mut processed_data = request.input_data.clone();
            for step in &config.steps {
                processed_data = self.apply_preprocessing_step(step, processed_data, &config.parameters).await?;
            }
            Ok(processed_data)
        } else {
            Ok(request.input_data.clone())
        }
    }

    async fn apply_preprocessing_step(
        &self,
        step: &PreprocessingStep,
        data: serde_json::Value,
        parameters: &HashMap<String, serde_json::Value>
    ) -> Result<serde_json::Value, String> {
        // Simulate preprocessing step application
        match step {
            PreprocessingStep::Normalize => {
                // Simulate normalization
                Ok(data)
            }
            PreprocessingStep::Tokenize => {
                // Simulate tokenization
                Ok(serde_json::json!({"tokens": [1, 2, 3, 4, 5]}))
            }
            _ => Ok(data)
        }
    }

    async fn perform_inference(&self, endpoint: &ModelEndpoint, input: &serde_json::Value) -> Result<serde_json::Value, String> {
        // Simulate model inference
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Mock prediction result
        Ok(serde_json::json!({
            "predictions": [0.1, 0.7, 0.2],
            "class": "class_1"
        }))
    }

    async fn postprocess_output(&self, request: &InferenceRequest, output: &serde_json::Value) -> Result<serde_json::Value, String> {
        // Simulate postprocessing
        if let Some(config) = &request.postprocessing_config {
            let mut processed_output = output.clone();
            for step in &config.steps {
                processed_output = self.apply_postprocessing_step(step, processed_output, &config.parameters).await?;
            }
            Ok(processed_output)
        } else {
            Ok(output.clone())
        }
    }

    async fn apply_postprocessing_step(
        &self,
        step: &PostprocessingStep,
        data: serde_json::Value,
        parameters: &HashMap<String, serde_json::Value>
    ) -> Result<serde_json::Value, String> {
        // Simulate postprocessing step application
        match step {
            PostprocessingStep::Softmax => {
                // Simulate softmax application
                Ok(data)
            }
            PostprocessingStep::Argmax => {
                // Simulate argmax
                Ok(serde_json::json!({"predicted_class": 1}))
            }
            _ => Ok(data)
        }
    }

    async fn update_endpoint_metrics(&self, endpoint_id: Uuid, processing_time_ms: f64) {
        // Update endpoint metrics
        let mut endpoints = self.endpoints.write().await;
        if let Some(endpoint) = endpoints.get_mut(&endpoint_id) {
            endpoint.metrics.requests_per_second += 1.0; // Simplified
            endpoint.metrics.average_latency_ms = 
                (endpoint.metrics.average_latency_ms + processing_time_ms) / 2.0;
            endpoint.metrics.last_updated = chrono::Utc::now();
        }
    }

    pub async fn get_endpoint(&self, endpoint_id: Uuid) -> Option<ModelEndpoint> {
        let endpoints = self.endpoints.read().await;
        endpoints.get(&endpoint_id).cloned()
    }

    pub async fn list_endpoints(&self) -> Vec<ModelEndpoint> {
        let endpoints = self.endpoints.read().await;
        endpoints.values().cloned().collect()
    }

    pub async fn scale_endpoint(&self, endpoint_id: Uuid, replicas: u32) -> Result<(), String> {
        let mut endpoints = self.endpoints.write().await;
        if let Some(endpoint) = endpoints.get_mut(&endpoint_id) {
            endpoint.deployment_config.replicas = replicas;
            endpoint.status = EndpointStatus::Scaling;
            endpoint.last_updated = chrono::Utc::now();
            
            // Simulate scaling process
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            endpoint.status = EndpointStatus::Active;
            let _ = self.event_bus.send(InferenceEvent::EndpointScaled(endpoint_id, replicas));
            Ok(())
        } else {
            Err("Endpoint not found".to_string())
        }
    }

    fn validate_deployment_config(&self, config: &DeploymentConfig) -> Result<(), String> {
        if config.replicas == 0 {
            return Err("Number of replicas must be greater than 0".to_string());
        }
        
        if config.resource_requirements.cpu_cores <= 0.0 {
            return Err("CPU cores must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            endpoint_health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn select_endpoint<'a>(&self, endpoints: &[&'a ModelEndpoint]) -> Result<&'a ModelEndpoint, String> {
        if endpoints.is_empty() {
            return Err("No endpoints available".to_string());
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                // Simplified round-robin - in practice would maintain state
                Ok(endpoints[0])
            }
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..endpoints.len());
                Ok(endpoints[index])
            }
            _ => Ok(endpoints[0]) // Default to first endpoint
        }
    }
}

impl BatchProcessor {
    pub fn new(max_batch_size: usize, max_wait_time: Duration) -> Self {
        Self {
            max_batch_size,
            max_wait_time,
            pending_requests: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl MetricsCollector {
    pub fn new(collection_interval: Duration) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockModelRegistry;

    #[async_trait::async_trait]
    impl ModelRegistry for MockModelRegistry {
        async fn get_model(&self, model_id: Uuid, version: Option<String>) -> Result<ModelInfo, String> {
            Ok(ModelInfo {
                id: model_id,
                name: "test_model".to_string(),
                version: version.unwrap_or("1.0.0".to_string()),
                framework: "pytorch".to_string(),
                input_schema: serde_json::json!({"type": "array"}),
                output_schema: serde_json::json!({"type": "array"}),
                preprocessing_config: None,
                postprocessing_config: None,
            })
        }

        async fn load_model(&self, model_id: Uuid, version: String) -> Result<LoadedModel, String> {
            Ok(LoadedModel {
                info: ModelInfo {
                    id: model_id,
                    name: "test_model".to_string(),
                    version,
                    framework: "pytorch".to_string(),
                    input_schema: serde_json::json!({"type": "array"}),
                    output_schema: serde_json::json!({"type": "array"}),
                    preprocessing_config: None,
                    postprocessing_config: None,
                },
                model_data: vec![0u8; 1024],
                loaded_at: chrono::Utc::now(),
            })
        }
    }

    #[tokio::test]
    async fn test_model_deployment() {
        let model_registry = Arc::new(MockModelRegistry);
        let engine = InferenceEngine::new(model_registry, 100);

        let endpoint = ModelEndpoint {
            id: Uuid::new_v4(),
            model_id: Uuid::new_v4(),
            model_version: "1.0.0".to_string(),
            endpoint_url: "http://localhost:8080/predict".to_string(),
            status: EndpointStatus::Deploying,
            deployment_config: DeploymentConfig {
                replicas: 2,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2.0,
                    memory_gb: 4.0,
                    gpu_count: 0,
                    gpu_memory_gb: 0.0,
                },
                auto_scaling: AutoScalingConfig {
                    enabled: true,
                    min_replicas: 1,
                    max_replicas: 10,
                    target_cpu_utilization: 70.0,
                    target_memory_utilization: 80.0,
                    scale_up_threshold: 80.0,
                    scale_down_threshold: 30.0,
                    cooldown_period: Duration::from_secs(300),
                },
                load_balancing: LoadBalancingConfig {
                    strategy: LoadBalancingStrategy::RoundRobin,
                    health_check_enabled: true,
                    sticky_sessions: false,
                },
                canary_deployment: None,
            },
            performance_config: PerformanceConfig {
                batch_size: 32,
                max_batch_delay_ms: 10,
                timeout_ms: 5000,
                cache_enabled: true,
                cache_ttl_seconds: 300,
                optimization_level: OptimizationLevel::Basic,
            },
            health_check_config: HealthCheckConfig {
                enabled: true,
                interval_seconds: 30,
                timeout_seconds: 5,
                healthy_threshold: 2,
                unhealthy_threshold: 3,
                path: "/health".to_string(),
            },
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            metrics: EndpointMetrics {
                requests_per_second: 0.0,
                average_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                error_rate: 0.0,
                cpu_utilization: 0.0,
                memory_utilization: 0.0,
                gpu_utilization: 0.0,
                active_connections: 0,
                queue_size: 0,
                cache_hit_rate: 0.0,
                last_updated: chrono::Utc::now(),
            },
        };

        let endpoint_id = engine.deploy_model(endpoint.clone()).await.unwrap();
        assert_eq!(endpoint_id, endpoint.id);

        let deployed_endpoint = engine.get_endpoint(endpoint_id).await.unwrap();
        assert_eq!(deployed_endpoint.status, EndpointStatus::Active);
    }

    #[tokio::test]
    async fn test_inference_request() {
        let model_registry = Arc::new(MockModelRegistry);
        let engine = InferenceEngine::new(model_registry, 100);

        // First deploy a model
        let model_id = Uuid::new_v4();
        let endpoint = ModelEndpoint {
            id: Uuid::new_v4(),
            model_id,
            model_version: "1.0.0".to_string(),
            endpoint_url: "http://localhost:8080/predict".to_string(),
            status: EndpointStatus::Deploying,
            deployment_config: DeploymentConfig {
                replicas: 1,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1.0,
                    memory_gb: 2.0,
                    gpu_count: 0,
                    gpu_memory_gb: 0.0,
                },
                auto_scaling: AutoScalingConfig {
                    enabled: false,
                    min_replicas: 1,
                    max_replicas: 1,
                    target_cpu_utilization: 70.0,
                    target_memory_utilization: 80.0,
                    scale_up_threshold: 80.0,
                    scale_down_threshold: 30.0,
                    cooldown_period: Duration::from_secs(300),
                },
                load_balancing: LoadBalancingConfig {
                    strategy: LoadBalancingStrategy::RoundRobin,
                    health_check_enabled: false,
                    sticky_sessions: false,
                },
                canary_deployment: None,
            },
            performance_config: PerformanceConfig {
                batch_size: 1,
                max_batch_delay_ms: 0,
                timeout_ms: 1000,
                cache_enabled: false,
                cache_ttl_seconds: 0,
                optimization_level: OptimizationLevel::None,
            },
            health_check_config: HealthCheckConfig {
                enabled: false,
                interval_seconds: 30,
                timeout_seconds: 5,
                healthy_threshold: 2,
                unhealthy_threshold: 3,
                path: "/health".to_string(),
            },
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            metrics: EndpointMetrics {
                requests_per_second: 0.0,
                average_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                error_rate: 0.0,
                cpu_utilization: 0.0,
                memory_utilization: 0.0,
                gpu_utilization: 0.0,
                active_connections: 0,
                queue_size: 0,
                cache_hit_rate: 0.0,
                last_updated: chrono::Utc::now(),
            },
        };

        engine.deploy_model(endpoint).await.unwrap();

        // Now make an inference request
        let request = InferenceRequest {
            id: Uuid::new_v4(),
            model_id,
            model_version: Some("1.0.0".to_string()),
            input_data: serde_json::json!([1.0, 2.0, 3.0, 4.0]),
            preprocessing_config: None,
            postprocessing_config: None,
            priority: RequestPriority::Normal,
            timeout: Duration::from_secs(5),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        let response = engine.predict(request.clone()).await.unwrap();
        assert_eq!(response.request_id, request.id);
        assert_eq!(response.model_id, model_id);
        assert_eq!(response.status, InferenceStatus::Success);
        assert!(response.processing_time_ms > 0.0);
    }
} 