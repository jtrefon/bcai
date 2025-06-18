//! Defines the error types for the devnet environment.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JobError {
    #[error("Job not found: {0}")]
    JobNotFound(String),
    #[error("Invalid job data")]
    InvalidJobData,
    #[error("Insufficient balance to post job")]
    InsufficientBalance,
}

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Insufficient balance for transfer or stake")]
    InsufficientBalance,
    #[error("Account not found: {0}")]
    AccountNotFound(String),
}

#[derive(Debug, Error)]
pub enum DevnetError {
    #[error("An I/O error occurred: {0}")]
    Io(#[from] std::io::Error),
    #[error("A serialization/deserialization error occurred: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("A job-related error occurred: {0}")]
    Job(#[from] JobError),
    #[error("A ledger-related error occurred: {0}")]
    Ledger(#[from] LedgerError),
} 