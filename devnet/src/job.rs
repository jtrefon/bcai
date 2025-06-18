use crate::ledger::{TokenLedger, TREASURY};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub data: Vec<u8>,
    pub reward: u64,
}

#[derive(Debug, Error)]
pub enum JobManagerError {
    #[error("Job not found: {0}")]
    JobNotFound(String),
    #[error("Invalid job data")]
    InvalidJobData,
    #[error("Insufficient balance")]
    InsufficientBalance,
}

pub fn post_job(
    jobs: &mut Vec<Job>,
    ledger: &mut TokenLedger,
    poster: &str,
    description: String,
    reward: u64,
) -> Result<(), JobManagerError> {
    if ledger.balance(poster) < reward {
        return Err(JobManagerError::InsufficientBalance);
    }

    ledger.transfer(poster, TREASURY, reward).unwrap(); // Escrow

    let job = Job {
        id: format!("job_{}", jobs.len()),
        data: description.into_bytes(),
        reward,
    };
    jobs.push(job);
    Ok(())
}

pub fn assign_job(jobs: &mut [Job], job_id: &str, _worker: &str) -> Result<(), JobManagerError> {
    if !jobs.iter().any(|j| j.id == job_id) {
        return Err(JobManagerError::JobNotFound(job_id.to_string()));
    }
    // In a real system, we'd assign this to a specific worker
    Ok(())
}

pub fn complete_job(
    jobs: &mut [Job],
    ledger: &mut TokenLedger,
    job_id: &str,
) -> Result<(), JobManagerError> {
    let job = jobs
        .iter_mut()
        .find(|j| j.id == job_id)
        .ok_or_else(|| JobManagerError::JobNotFound(job_id.to_string()))?;

    // In a real system, we'd transfer reward to the worker
    ledger.transfer(TREASURY, "worker", job.reward).unwrap();
    job.reward = 0;

    Ok(())
} 