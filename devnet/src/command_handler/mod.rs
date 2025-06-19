//! Command handler public facade – delegates to specialized sub-modules.

pub mod core;
mod info;
mod mine;
mod tx;
mod account;
mod job;

pub use core::CommandHandler; 