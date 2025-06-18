pub mod error;
pub mod config;
pub mod job;


// Re-export commonly used types
pub use config::FederatedNetworkConfig;
pub use error::FederatedNetworkError;
pub use job::{
    FederatedJobStatus, FederatedTrainingConfig, FederatedTrainingJob, ModelArchitecture,
    ParticipantInfo, ParticipantStatus,
}; 