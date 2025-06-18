//! Manages the lifecycle of data chunks including storage, retrieval, and cleanup.
//!
//! The `ChunkManager` acts as a central in-memory cache for `DataChunk` objects,
//! enforcing memory limits and eviction policies like LRU and TTL.

mod config;
mod entry;
mod eviction;
mod info;
mod manager;
mod stats;

#[cfg(test)]
mod tests;

pub use config::ChunkManagerConfig;
pub use manager::ChunkManager;
pub use stats::ChunkManagerStats; 