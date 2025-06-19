use super::{
    session::TransferSession,
    state::TransferState,
};
use std::time::{Duration, Instant};

impl TransferSession {
    /// Update session state and refresh the activity timer.
    pub fn set_state(&mut self, state: TransferState) {
        self.state = state;
        self.last_activity = Instant::now();
    }

    /// Check if the session has been inactive for longer than the timeout duration.
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
} 