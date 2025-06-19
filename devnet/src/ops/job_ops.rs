use crate::error::DevnetError;
use crate::job::{assign_job, complete_job, post_job};
use crate::persistence::{load_jobs, save_jobs, load_ledger, save_ledger};

pub fn handle_job_command(cmd: crate::commands::JobCommands) -> Result<(), DevnetError> {
    let mut ledger = load_ledger()?;
    let mut jobs = load_jobs()?;

    match cmd {
        crate::commands::JobCommands::Post { poster, description, reward } => {
            if let Err(e) = post_job(&mut jobs, &mut ledger, &poster, description, reward) {
                println!("{e}");
            } else if let Some(j) = jobs.last() {
                println!("posted job #{}", j.id);
            }
        }
        crate::commands::JobCommands::Assign { job_id, worker } => {
            if let Err(e) = assign_job(&mut jobs, job_id, &worker) {
                println!("{e}");
            }
        }
        crate::commands::JobCommands::Complete { job_id } => {
            if let Err(e) = complete_job(&mut jobs, &mut ledger, job_id) {
                println!("{e}");
            }
        }
        crate::commands::JobCommands::List => {
            for job in &jobs {
                println!(
                    "#{:<3} {:<20} reward:{:<5} assigned:{:<10} completed:{}",
                    job.id,
                    job.description,
                    job.reward,
                    job.assigned_to.as_deref().unwrap_or("-"),
                    job.completed
                );
            }
        }
    }

    save_jobs(&jobs)?;
    save_ledger(&ledger)?;
    Ok(())
} 