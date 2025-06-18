#[derive(Debug, thiserror::Error)]
pub enum PerformanceError {
    #[error("Cache is full")]
    CacheFull,
    #[error("Bandwidth limit exceeded")]
    BandwidthLimitExceeded,
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
    #[error("Optimization error: {0}")]
    OptimizationError(String),
} 