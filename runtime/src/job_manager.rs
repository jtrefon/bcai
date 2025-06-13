use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::token::TokenLedger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub description: String,
    pub reward: u64,
    pub assigned_to: Option<String>,
    pub completed: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JobManagerError {
    #[error("job not found")]
    JobNotFound,
    #[error("job already assigned")]
    AlreadyAssigned,
    #[error("job already completed")]
    AlreadyCompleted,
    #[error("insufficient balance for reward")]
    InsufficientBalance,
}

/// Minimal in-memory job manager.
pub struct JobManager {
    jobs: Vec<Job>,
    ledger: TokenLedger,
}

impl JobManager {
    /// Create a new job manager with the given ledger.
    pub fn new(ledger: TokenLedger) -> Self {
        Self { jobs: Vec::new(), ledger }
    }

    /// Access the underlying ledger.
    pub fn ledger(&self) -> &TokenLedger {
        &self.ledger
    }

    /// Mutable access to the ledger.
    pub fn ledger_mut(&mut self) -> &mut TokenLedger {
        &mut self.ledger
    }

    /// Post a new job and reserve the reward from the poster's balance.
    pub fn post_job(
        &mut self,
        poster: &str,
        description: String,
        reward: u64,
    ) -> Result<&Job, JobManagerError> {
        if self.ledger.balance(poster) < reward {
            return Err(JobManagerError::InsufficientBalance);
        }
        // reserve tokens by transferring to a temporary holding account
        self.ledger.transfer(poster, "escrow", reward).unwrap();
        let id = self.jobs.last().map(|j| j.id + 1).unwrap_or(1);
        let job = Job { id, description, reward, assigned_to: None, completed: false };
        self.jobs.push(job);
        Ok(self.jobs.last().unwrap())
    }

    /// Assign a worker to an open job.
    pub fn assign_job(&mut self, job_id: u64, worker: &str) -> Result<(), JobManagerError> {
        let job =
            self.jobs.iter_mut().find(|j| j.id == job_id).ok_or(JobManagerError::JobNotFound)?;
        if job.completed {
            return Err(JobManagerError::AlreadyCompleted);
        }
        if job.assigned_to.is_some() {
            return Err(JobManagerError::AlreadyAssigned);
        }
        job.assigned_to = Some(worker.to_string());
        Ok(())
    }

    /// Mark a job as completed and pay out the reward to the worker.
    pub fn complete_job(&mut self, job_id: u64) -> Result<(), JobManagerError> {
        let job =
            self.jobs.iter_mut().find(|j| j.id == job_id).ok_or(JobManagerError::JobNotFound)?;
        if job.completed {
            return Err(JobManagerError::AlreadyCompleted);
        }
        let worker = job.assigned_to.clone().ok_or(JobManagerError::AlreadyAssigned)?;
        job.completed = true;
        // release reward from escrow to worker
        self.ledger.transfer("escrow", &worker, job.reward).unwrap();
        Ok(())
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new(TokenLedger::new())
    }
}
