//! Chunk management for large data transfers
//!
//! This module provides the core types and functionality for handling 
//! data chunks in the large data transfer system.

pub mod data;
pub mod error;
pub mod id;
pub mod info;
pub mod stats;
pub mod utils;

#[cfg(test)]
mod tests;

pub use data::DataChunk;
pub use error::ChunkError;
pub use id::ChunkId;
pub use info::ChunkInfo;
pub use stats::ChunkStats;
pub use utils::*; 