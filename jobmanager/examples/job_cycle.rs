use jobmanager_lib::{load_jobs, save_jobs, post_job, assign_job, complete_job};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut jobs = load_jobs()?;
    let job = post_job(&mut jobs, "demo job".into(), 10);
    assign_job(jobs.as_mut_slice(), job.id, "worker1".into())?;
    complete_job(jobs.as_mut_slice(), job.id)?;
    save_jobs(&jobs)?;
    println!("Job {} completed", job.id);
    Ok(())
}
