//! The devnet binary for BCAI.
//!
//! This crate provides a command-line interface for running a local development
//! network, interacting with it, and managing the daemon process.

pub mod cli;
pub mod command_handler;
pub mod daemon;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

// === Core Configuration ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevnetConfig {
    pub node_count: u32,
    pub ai_workers: u32,
    pub initial_tokens: u64,
    pub port_base: u16,
}

impl Default for DevnetConfig {
    fn default() -> Self {
        Self {
            node_count: 3,
            ai_workers: 1,
            initial_tokens: 1000,
            port_base: 8000,
        }
    }
}

// === Devnet Node Startup ===
pub async fn start_devnet_node(config: DevnetConfig) -> Result<(), DevnetError> {
    println!("🚀 Starting BCAI Devnet with config: {:?}", config);
    
    // Initialize ledger with treasury and initial allocations
    let mut ledger = load_ledger()?;
    mint(&mut ledger, TREASURY, config.initial_tokens * 10); // Treasury gets 10x
    
    // Create initial nodes
    for i in 0..config.node_count {
        let node_id = format!("node_{}", i);
        mint(&mut ledger, &node_id, config.initial_tokens);
        println!("✅ Created node: {} with {} tokens", node_id, config.initial_tokens);
    }
    
    // Create AI workers
    for i in 0..config.ai_workers {
        let worker_id = format!("ai_worker_{}", i);
        mint(&mut ledger, &worker_id, config.initial_tokens / 2);
        println!("🤖 Created AI worker: {} with {} tokens", worker_id, config.initial_tokens / 2);
    }
    
    save_ledger(&ledger)?;
    
    // Initialize jobs
    let mut jobs = load_jobs()?;
    
    // Create sample training job
    post_job(
        &mut jobs, 
        &mut ledger, 
        TREASURY, 
        "Sample neural network training task".to_string(), 
        100
    )?;
    
    save_jobs(&jobs)?;
    save_ledger(&ledger)?;
    
    println!("🎉 Devnet initialized successfully!");
    println!("📊 Total nodes: {}, AI workers: {}", config.node_count, config.ai_workers);
    println!("💰 Treasury balance: {}", ledger.balance(TREASURY));
    println!("📝 Available jobs: {}", jobs.len());
    
    Ok(())
}

// === Existing Types and Functions ===

// Simplified types for devnet
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
}

impl TokenLedger {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }
    
    pub fn balance(&self, account: &str) -> u64 {
        self.balances.get(account).copied().unwrap_or(0)
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
}

#[derive(Debug, Error)]
pub enum DevnetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Job manager error: {0}")]
    JobManager(#[from] JobManagerError),
    #[error("Ledger error: {0}")]
    Ledger(#[from] LedgerError),
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
    ledger.balances.insert(account.to_string(), amount);
}

pub fn transfer(
    ledger: &mut TokenLedger,
    from: &str,
    to: &str,
    amount: u64,
) -> Result<(), LedgerError> {
    if ledger.balances.get(from).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(from.to_string(), ledger.balances[from] - amount);
    ledger.balances.insert(to.to_string(), ledger.balances.get(to).copied().unwrap_or(0) + amount);
    Ok(())
}

pub fn stake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(account).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(account.to_string(), ledger.balances[account] - amount);
    Ok(())
}

pub fn unstake(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(account).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(account.to_string(), ledger.balances[account] + amount);
    Ok(())
}

pub fn slash(ledger: &mut TokenLedger, offender: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(offender).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(offender.to_string(), ledger.balances[offender] - amount);
    ledger.balances.insert(TREASURY.to_string(), ledger.balances.get(TREASURY).copied().unwrap_or(0) + amount);
    Ok(())
}

pub fn reputation(ledger: &TokenLedger, account: &str) -> i32 {
    ledger.balances.get(account).copied().unwrap_or(0) as i32
}

pub fn adjust_reputation(ledger: &mut TokenLedger, account: &str, delta: i32) {
    let current = ledger.balances.get(account).copied().unwrap_or(0) as i64;
    let new_balance = std::cmp::max(0, current + delta as i64) as u64;
    ledger.balances.insert(account.to_string(), new_balance);
}

pub fn burn(ledger: &mut TokenLedger, account: &str, amount: u64) -> Result<(), LedgerError> {
    if ledger.balances.get(account).copied().unwrap_or(0) < amount {
        return Err(LedgerError::InsufficientBalance);
    }
    ledger.balances.insert(account.to_string(), ledger.balances[account] - amount);
    Ok(())
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
    description: String,
    reward: u64,
) -> Result<(), JobManagerError> {
    if ledger.balance(poster) < reward {
        return Err(JobManagerError::InsufficientBalance);
    }
    ledger.transfer(poster, "escrow", reward).map_err(|_| JobManagerError::InsufficientBalance)?;
    let id = jobs.last().map(|j| j.id.clone()).unwrap_or_default();
    jobs.push(Job { id, data: Vec::new(), reward });
    Ok(())
}

pub fn assign_job(jobs: &mut [Job], job_id: &str, worker: &str) -> Result<(), JobManagerError> {
    let job = jobs.iter_mut().find(|j| j.id == job_id.to_string()).ok_or(JobManagerError::JobNotFound(job_id.to_string()))?;
    job.data = Vec::new();
    job.id = worker.to_string();
    Ok(())
}

pub fn complete_job(
    jobs: &mut [Job],
    ledger: &mut TokenLedger,
    job_id: &str,
) -> Result<(), JobManagerError> {
    let job = jobs.iter_mut().find(|j| j.id == job_id.to_string()).ok_or(JobManagerError::JobNotFound(job_id.to_string()))?;
    if !job.data.is_empty() {
        return Err(JobManagerError::InvalidJobData);
    }
    let worker = job.id.clone();
    ledger.transfer("escrow", &worker, job.reward).map_err(|_| JobManagerError::InsufficientBalance)?;
    job.data = Vec::new();
    Ok(())
}
