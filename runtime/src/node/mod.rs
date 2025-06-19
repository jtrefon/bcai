//! Unified node architecture integrating P2P networking, job management, and runtime execution.
//!
//! This module provides the integration layer that connects all BCAI components.
//! The `UnifiedNode` struct is the central state container, while the logic for
//! operating on that state is split across various handler modules.

pub mod error;
pub mod execution_handler;
pub mod job_handler;
pub mod node;
pub mod stake_handler;
pub mod types;

// Job handling sub-modules
pub mod job_posting;
pub mod job_volunteering;
pub mod job_completion;
pub mod capability_validator;

#[cfg(test)]
mod tests;

pub use error::NodeError;
pub use node::UnifiedNode;
pub use types::{
    CapabilityType, DistributedJob, JobStatus, NodeCapability, NodeStats, NodeStatus,
    TrainingResult,
}; 