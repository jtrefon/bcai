pub mod optimizer;
pub mod metrics;
pub mod error;

// Re-export commonly used types
pub use error::PerformanceError;
pub use metrics::{PerformanceStats, ResourceMetrics};
pub use optimizer::PerformanceConfig; 