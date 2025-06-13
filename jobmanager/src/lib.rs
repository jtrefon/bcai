use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: u64,
    pub description: String,
    pub reward: u64,
    pub assigned_to: Option<String>,
    pub completed: bool,
}

#[derive(Debug, Error)]
pub enum JobError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("job not found")]
    JobNotFound,
}

pub const DATA_FILE: &str = "jobs.json";

pub fn load_jobs() -> Result<Vec<Job>, JobError> {
    if !Path::new(DATA_FILE).exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(DATA_FILE)?;
    let jobs = serde_json::from_str(&data)?;
    Ok(jobs)
}

pub fn save_jobs(jobs: &[Job]) -> Result<(), JobError> {
    let data = serde_json::to_string_pretty(jobs)?;
    fs::write(DATA_FILE, data)?;
    Ok(())
}

pub fn post_job(jobs: &mut Vec<Job>, description: String, reward: u64) -> Job {
    let id = jobs.last().map(|j| j.id + 1).unwrap_or(1);
    let job = Job { id, description, reward, assigned_to: None, completed: false };
    jobs.push(job.clone());
    job
}

pub fn assign_job(jobs: &mut [Job], job_id: u64, worker: String) -> Result<(), JobError> {
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.assigned_to = Some(worker);
        return Ok(());
    }
    Err(JobError::JobNotFound)
}

pub fn complete_job(jobs: &mut [Job], job_id: u64) -> Result<(), JobError> {
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.completed = true;
        return Ok(());
    }
    Err(JobError::JobNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        let _ = fs::remove_file(DATA_FILE);
    }

    #[test]
    fn post_assign_complete_cycle() -> Result<(), JobError> {
        setup();
        let mut jobs = load_jobs()?;
        let job = post_job(&mut jobs, "a task".into(), 50);
        assert_eq!(job.id, 1);
        assign_job(jobs.as_mut_slice(), job.id, "worker1".into())?;
        complete_job(jobs.as_mut_slice(), job.id)?;
        save_jobs(&jobs)?;
        let loaded = load_jobs()?;
        assert!(loaded[0].completed);
        assert_eq!(loaded[0].assigned_to.as_deref(), Some("worker1"));
        Ok(())
    }
}
