//! Central configuration types for the large-data-transfer subsystem.
//!
//! This module was split into smaller single-responsibility files so that each
//! config group remains ≤100 LOC.

mod core;
mod cache;
mod compression;
mod encryption;
mod retry;

pub use core::LargeDataConfig;
pub use cache::CacheConfig;
pub use compression::{CompressionConfig, CompressionAlgorithm};
pub use encryption::{EncryptionConfig, EncryptionAlgorithm};
pub use retry::RetryConfig; 