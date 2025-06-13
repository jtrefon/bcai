use runtime::job_manager::{Job, JobManagerError};
use runtime::token::{LedgerError, TokenLedger};
use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DevnetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
#[derive(Debug, Serialize, Deserialize)]
struct LedgerWrapper {
    ledger: TokenLedger,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobsWrapper {
    jobs: Vec<Job>,
}

pub const LEDGER_FILE: &str = "ledger.json";
pub const JOBS_FILE: &str = "jobs.json";
pub const TREASURY: &str = "treasury";

pub fn load_ledger() -> Result<TokenLedger, DevnetError> {
    if !std::path::Path::new(LEDGER_FILE).exists() {
        return Ok(TokenLedger::new());
    }
    let data = fs::read_to_string(LEDGER_FILE)?;
    let wrapper: LedgerWrapper = serde_json::from_str(&data)?;
    Ok(wrapper.ledger)
}

pub fn save_ledger(ledger: &TokenLedger) -> Result<(), DevnetError> {
    let data = serde_json::to_string_pretty(&LedgerWrapper { ledger: ledger.clone() })?;
    fs::write(LEDGER_FILE, data)?;
    Ok(())
}

pub fn load_jobs() -> Result<Vec<Job>, DevnetError> {
    if !std::path::Path::new(JOBS_FILE).exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(JOBS_FILE)?;
    let wrapper: JobsWrapper = serde_json::from_str(&data)?;
    Ok(wrapper.jobs)
}

pub fn save_jobs(jobs: &[Job]) -> Result<(), DevnetError> {
    let data = serde_json::to_string_pretty(&JobsWrapper { jobs: jobs.to_vec() })?;
    fs::write(JOBS_FILE, data)?;
    Ok(())
}

pub fn mint(ledger: &mut TokenLedger, account: &str, amount: u64) {
    ledger.mint(account, amount);
}

pub fn transfer(
    ledger: &mut TokenLedger,
    from: &str,
    to: &str,
    amount: u64,
) -> Result<(), LedgerError> {
    ledger.transfer(from, to, amount)
}

pub fn stake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    ledger.stake(account, amount)
}

pub fn unstake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    ledger.unstake(account, amount)
}

pub fn slash(ledger: &mut TokenLedger, offender: &str, amount: u64) -> Result<(), LedgerError> {
    ledger.slash(offender, TREASURY, amount)
}

pub fn reputation(ledger: &TokenLedger, account: &str) -> i32 {
    ledger.reputation(account)
}

pub fn adjust_reputation(ledger: &mut TokenLedger, account: &str, delta: i32) {
    ledger.adjust_reputation(account, delta);
}

pub fn train_and_verify(size: usize, seed: u64, difficulty: u32) -> bool {
    use runtime::evaluator::Evaluator;
    use runtime::pouw::generate_task;
    use runtime::trainer::Trainer;

    let task = generate_task(size, seed);
    let trainer = Trainer::new("trainer");
    let solution = trainer.train(&task, difficulty);
    let evaluator = Evaluator::new("evaluator");
    evaluator.evaluate(&task, &solution, difficulty)
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
    ledger.transfer(poster, "escrow", reward).unwrap();
    let id = jobs.last().map(|j| j.id + 1).unwrap_or(1);
    jobs.push(Job { id, description, reward, assigned_to: None, completed: false });
    Ok(())
}

pub fn assign_job(jobs: &mut [Job], job_id: u64, worker: &str) -> Result<(), JobManagerError> {
    let job = jobs.iter_mut().find(|j| j.id == job_id).ok_or(JobManagerError::JobNotFound)?;
    if job.completed {
        return Err(JobManagerError::AlreadyCompleted);
    }
    if job.assigned_to.is_some() {
        return Err(JobManagerError::AlreadyAssigned);
    }
    job.assigned_to = Some(worker.to_string());
    Ok(())
}

pub fn complete_job(
    jobs: &mut [Job],
    ledger: &mut TokenLedger,
    job_id: u64,
) -> Result<(), JobManagerError> {
    let job = jobs.iter_mut().find(|j| j.id == job_id).ok_or(JobManagerError::JobNotFound)?;
    if job.completed {
        return Err(JobManagerError::AlreadyCompleted);
    }
    let worker = job.assigned_to.clone().ok_or(JobManagerError::AlreadyAssigned)?;
    job.completed = true;
    ledger.transfer("escrow", &worker, job.reward).unwrap();
    Ok(())
}
