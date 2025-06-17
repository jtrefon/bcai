//! The core data structures and logic for the blockchain itself.
//!
//! This module is the result of refactoring a single, monolithic `blockchain.rs`
//! file into a collection of single-responsibility modules, adhering to SOLID principles.

// 1. Declare the sub-modules. The code for each is in a file with the same name.
pub mod block;
pub mod chain;
pub mod config;
pub mod constants;
pub mod error;
pub mod state;
pub mod transaction;
pub mod validation;

// 2. Re-export the most important public types for easier access.
pub use block::Block;
pub use chain::Blockchain;
pub use config::BlockchainConfig;
pub use error::BlockchainError;
pub use transaction::Transaction; 