//! BCAI - Blockchain AI Infrastructure
//! 
//! A comprehensive blockchain-based AI infrastructure with advanced ML capabilities

use std::sync::Arc;

pub mod ml;

/// Main BCAI library
pub struct BCAI {
    pub ml_orchestrator: Option<ml::pipeline_orchestrator::PipelineOrchestrator>,
    pub model_registry: Option<ml::model_registry::ModelRegistry>,
    pub inference_engine: Option<std::sync::Arc<ml::inference_engine::InferenceEngine>>,
    pub monitoring_system: Option<ml::ml_monitoring::MLMonitoringSystem>,
}

impl BCAI {
    pub fn new() -> Self {
        Self {
            ml_orchestrator: None,
            model_registry: None,
            inference_engine: None,
            monitoring_system: None,
        }
    }
    
    pub fn version() -> &'static str {
        "0.1.0"
    }

    pub fn with_ml_orchestrator(mut self, max_concurrent_executions: usize, max_concurrent_tasks: usize) -> Self {
        let total_resources = ml::pipeline_orchestrator::ResourceRequirements {
            cpu_cores: 16,
            memory_gb: 64,
            gpu_count: 4,
            storage_gb: 1000,
            network_bandwidth_mbps: 1000,
        };
        
        self.ml_orchestrator = Some(ml::pipeline_orchestrator::PipelineOrchestrator::new(
            max_concurrent_executions,
            max_concurrent_tasks,
            total_resources,
        ));
        self
    }

    pub fn with_model_registry(mut self) -> Self {
        let storage_backend = Arc::new(ml::model_registry::LocalStorageBackend::new("./models".to_string()));
        self.model_registry = Some(ml::model_registry::ModelRegistry::new(storage_backend));
        self
    }

    pub fn with_monitoring_system(mut self) -> Self {
        self.monitoring_system = Some(ml::ml_monitoring::MLMonitoringSystem::new());
        self
    }
}

impl Default for BCAI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcai_creation() {
        let _bcai = BCAI::new();
        assert_eq!(BCAI::version(), "0.1.0");
    }

    #[test]
    fn test_bcai_with_components() {
        let bcai = BCAI::new()
            .with_ml_orchestrator(10, 50)
            .with_model_registry()
            .with_monitoring_system();
        
        assert!(bcai.ml_orchestrator.is_some());
        assert!(bcai.model_registry.is_some());
        assert!(bcai.monitoring_system.is_some());
    }
} 