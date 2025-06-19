//! Implements the job lifecycle management logic for the `UnifiedNode`.

mod job_posting;
mod job_volunteering;
mod job_completion;
mod capability_validator;

// Re-export the functionality through the separated modules
pub use job_posting::*;
pub use job_volunteering::*;
pub use job_completion::*;
pub use capability_validator::*; 