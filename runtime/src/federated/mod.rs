pub mod learning;
pub mod aggregation;
pub mod model;

// Re-export commonly used types
pub use aggregation::{AggregationStrategy, FederatedConfig};
pub use learning::{FederatedParticipant, FederatedRound, FederatedStats};
pub use model::{FederatedError, ModelMetadata, ModelParameters}; 