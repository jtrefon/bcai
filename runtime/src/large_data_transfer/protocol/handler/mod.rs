//! Transfer protocol handler – split into focused sub-modules.

mod core;
mod transfer_request;
mod chunk_request;
mod chunk_data;
mod maintenance;

pub use core::ProtocolHandler; 