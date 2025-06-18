//! BCAI - Blockchain AI Infrastructure
//! 
//! A comprehensive blockchain-based AI infrastructure with advanced ML capabilities

pub mod ml;

/// Main BCAI library
pub struct BCAI {
    pub ml_orchestrator: Option<ml::pipeline_orchestrator::Pipeline>,
    pub model_registry: Option<ml::model_registry::ModelMetadata>,
    pub inference_engine: Option<ml::inference_engine::ModelEndpoint>,
    pub monitoring_system: Option<ml::monitoring::MLMetrics>,
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
} 