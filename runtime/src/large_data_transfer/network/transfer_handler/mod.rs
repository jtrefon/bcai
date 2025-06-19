//! Network transfer handler – split into focused sub-modules.
//!
//! Each sub-module extends `NetworkTransferCoordinator` with a specific set of
//! responsibilities while keeping every file ≤ 100 LOC.

mod announce;
mod request;
mod orchestrator;
mod process;
mod internal;

// No public items are declared here; everything is attached via `impl` blocks
// on `NetworkTransferCoordinator` defined in the sub-modules. 