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
    jobs: std::collections::HashMap<u64, (String, u64)>,
}

impl JobManager {
    pub fn new(ledger: crate::token::TokenLedger) -> Self {
        Self {
            ledger,
            jobs: std::collections::HashMap::new(),
        }
    }

    /// Submits a new job, escrowing the reward from the poster.
    pub fn submit_job(
        &mut self,
        poster: &str,
        job_id: u64,
        reward: u64,
    ) -> Result<(), JobManagerError> {
        if self.ledger.balance(poster) < reward {
            return Err(JobManagerError::InsufficientBalance);
        }

        self
            .ledger
            .transfer(poster, "escrow", reward)
            .map_err(|e| JobManagerError::Stub(e.to_string()))?;
        self.jobs.insert(job_id, (poster.to_string(), reward));
        Ok(())
    }

    pub fn ledger(&self) -> &crate::token::TokenLedger {
        &self.ledger
    }

    pub fn ledger_mut(&mut self) -> &mut crate::token::TokenLedger {
        &mut self.ledger
    }
} 