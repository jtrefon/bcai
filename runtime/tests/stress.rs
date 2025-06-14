use std::sync::{Arc, Mutex};
use std::thread;

use runtime::job_manager::{JobManager, JobManagerError};
use runtime::token::TokenLedger;

#[test]
fn concurrent_job_flow() -> Result<(), JobManagerError> {
    const THREADS: usize = 4;
    const JOBS_PER_THREAD: usize = 25;

    let mut ledger = TokenLedger::new();
    for i in 0..THREADS {
        ledger.mint(&format!("user{i}"), 1_000);
    }
    let manager = Arc::new(Mutex::new(JobManager::new(ledger)));

    let mut handles = Vec::new();
    for i in 0..THREADS {
        let jm = Arc::clone(&manager);
        handles.push(thread::spawn(move || {
            for j in 0..JOBS_PER_THREAD {
                let poster = format!("user{i}");
                let id = {
                    let mut jm = jm.lock().unwrap();
                    jm.post_job(&poster, format!("job {i}-{j}"), 1).unwrap().id
                };
                let mut jm = jm.lock().unwrap();
                jm.assign_job(id, &poster).unwrap();
                jm.complete_job(id).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let jm = manager.lock().unwrap();
    assert_eq!(jm.jobs().len(), THREADS * JOBS_PER_THREAD);
    assert!(jm.jobs().iter().all(|j| j.completed));
    Ok(())
}
