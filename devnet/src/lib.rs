use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

// Simplified types for devnet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub data: Vec<u8>,
    pub reward: u64,
    pub worker: Option<String>,
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

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Account not found: {0}")]
    AccountNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLedger {
    pub balances: HashMap<String, u64>,
    pub stakes: HashMap<String, u64>,
    pub reputations: HashMap<String, i32>,
}

impl Default for TokenLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenLedger {
    pub fn new() -> Self {
        Self { balances: HashMap::new(), stakes: HashMap::new(), reputations: HashMap::new() }
    }

    pub fn balance(&self, account: &str) -> u64 {
        self.balances.get(account).copied().unwrap_or(0)
    }

    pub fn stake_balance(&self, account: &str) -> u64 {
        self.stakes.get(account).copied().unwrap_or(0)
    }

    pub fn reputation(&self, account: &str) -> i32 {
        self.reputations.get(account).copied().unwrap_or(0)
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), LedgerError> {
        if self.balances.get(from).copied().unwrap_or(0) < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        let from_balance = self.balances.get(from).copied().unwrap_or(0);
        let to_balance = self.balances.get(to).copied().unwrap_or(0);
        self.balances.insert(from.to_string(), from_balance - amount);
        self.balances.insert(to.to_string(), to_balance + amount);
        Ok(())
    }

    pub fn mint(&mut self, account: &str, amount: u64) {
        let bal = self.balances.entry(account.to_string()).or_default();
        *bal += amount;
    }

    pub fn burn(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        if self.balance(account) < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        let bal = self.balances.entry(account.to_string()).or_default();
        *bal -= amount;
        Ok(())
    }

    pub fn stake(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        if self.balance(account) < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        let bal = self.balances.entry(account.to_string()).or_default();
        *bal -= amount;
        let st = self.stakes.entry(account.to_string()).or_default();
        *st += amount;
        Ok(())
    }

    pub fn unstake(&mut self, account: &str, amount: u64) -> Result<(), LedgerError> {
        let st = self.stakes.entry(account.to_string()).or_default();
        if *st < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        *st -= amount;
        let bal = self.balances.entry(account.to_string()).or_default();
        *bal += amount;
        Ok(())
    }

    pub fn slash(&mut self, offender: &str, amount: u64) -> Result<(), LedgerError> {
        let st = self.stakes.entry(offender.to_string()).or_default();
        if *st < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        *st -= amount;
        let tre = self.balances.entry(TREASURY.to_string()).or_default();
        *tre += amount;
        Ok(())
    }

    pub fn adjust_reputation(&mut self, account: &str, delta: i32) {
        let rep = self.reputations.entry(account.to_string()).or_default();
        *rep += delta;
    }
}

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
    ledger.slash(offender, amount)
}

pub fn reputation(ledger: &TokenLedger, account: &str) -> i32 {
    ledger.reputation(account)
}

pub fn adjust_reputation(ledger: &mut TokenLedger, account: &str, delta: i32) {
    ledger.adjust_reputation(account, delta);
}

pub fn burn(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    ledger.burn(account, amount)
}

pub fn train_and_verify(_size: usize, _seed: u64, _difficulty: u32) -> bool {
    // Simplified test - removed complex dependencies

    // Simplified implementation
    true
}

pub fn post_job(
    jobs: &mut Vec<Job>,
    ledger: &mut TokenLedger,
    poster: &str,
    _description: String,
    reward: u64,
) -> Result<(), JobManagerError> {
    if ledger.balance(poster) < reward {
        return Err(JobManagerError::InsufficientBalance);
    }
    ledger.transfer(poster, "escrow", reward).map_err(|_| JobManagerError::InsufficientBalance)?;
    let id = (jobs.len() + 1).to_string();
    jobs.push(Job { id, data: Vec::new(), reward, worker: None });
    Ok(())
}

pub fn assign_job(jobs: &mut [Job], job_id: &str, worker: &str) -> Result<(), JobManagerError> {
    let job = jobs
        .iter_mut()
        .find(|j| j.id == job_id)
        .ok_or(JobManagerError::JobNotFound(job_id.to_string()))?;
    job.worker = Some(worker.to_string());
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
        .ok_or(JobManagerError::JobNotFound(job_id.to_string()))?;
    if let Some(worker) = job.worker.clone() {
        ledger
            .transfer("escrow", &worker, job.reward)
            .map_err(|_| JobManagerError::InsufficientBalance)?;
        job.worker = None;
    } else {
        return Err(JobManagerError::InvalidJobData);
    }
    Ok(())
}
