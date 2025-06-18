use crate::error::DevnetError;
use crate::job::Job;
use crate::ledger::TokenLedger;
use serde::{Deserialize, Serialize};
use std::fs;

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

pub fn load_ledger() -> Result<TokenLedger, DevnetError> {
    if !std::path::Path::new(LEDGER_FILE).exists() {
        return Ok(TokenLedger::new());
    }
    let data = fs::read_to_string(LEDGER_FILE)?;
    let wrapper: LedgerWrapper = serde_json::from_str(&data)?;
    Ok(wrapper.ledger)
}

pub fn save_ledger(ledger: &TokenLedger) -> Result<(), DevnetError> {
    let data = serde_json::to_string_pretty(&LedgerWrapper {
        ledger: ledger.clone(),
    })?;
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
    let data = serde_json::to_string_pretty(&JobsWrapper {
        jobs: jobs.to_vec(),
    })?;
    fs::write(JOBS_FILE, data)?;
    Ok(())
} 