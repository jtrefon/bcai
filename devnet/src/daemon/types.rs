//! Shared constants and simple type aliases for the devnet daemon.

use runtime::blockchain::Transaction;
use runtime::job::Job;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

// --- IPC ---------------------------------------------------------------------

/// Filesystem path to the UNIX domain socket used for CLI ↔ daemon IPC.
pub const SOCKET_PATH: &str = "/tmp/bcai_devnet.sock";
/// PID file written on daemon startup so external scripts can manage it.
pub const PID_FILE: &str = "/tmp/bcai_devnet.pid";

// --- Shared state ------------------------------------------------------------

/// Pending transactions forwarded from the CLI to the P2P layer.
pub type Mempool = Arc<Mutex<HashSet<Transaction>>>;
/// Queue of compute jobs awaiting miners.
pub type JobQueue = Arc<Mutex<VecDeque<Job>>>; 