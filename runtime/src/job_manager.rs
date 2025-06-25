use thiserror::Error;

/// Placeholder error for job manager ops.
#[derive(Debug, Error)]
pub enum JobManagerError {
    #[error("operation failed: {0}")]
    Stub(String),
    #[error("insufficient balance for job action")]
    InsufficientBalance,
}

/// Minimal stub for job manager, tracks jobs by id.
#[derive(Debug, Clone)]
pub struct JobManager {
    ledger: crate::token::TokenLedger,
}

impl JobManager {
    pub fn new(ledger: crate::token::TokenLedger) -> Self {
        Self { ledger }
    }

    pub fn submit_job(&self, _job_id: u64) -> Result<(), JobManagerError> {
        // TODO: Implement real submit.
        Ok(())
    }

    pub fn ledger(&self) -> &crate::token::TokenLedger {
        &self.ledger
    }

    pub fn ledger_mut(&mut self) -> &mut crate::token::TokenLedger {
        &mut self.ledger
    }
} 