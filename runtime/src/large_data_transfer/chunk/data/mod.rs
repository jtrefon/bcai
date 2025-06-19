//! Data chunk representation split into focused sub-modules.

mod core;
mod compression;
mod verify;

pub use core::DataChunk; 