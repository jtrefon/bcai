pub mod engine;
pub mod messages;
pub mod state;

// Re-export commonly used types
pub use engine::{ConsensusAlgorithm, ConsensusConfig, Validator};
pub use messages::{ConsensusProposal, ConsensusResult, Vote, VoteType};
pub use state::ConsensusStats; 