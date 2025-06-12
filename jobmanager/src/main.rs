use clap::{Parser, Subcommand};
use jobmanager_lib::{assign_job, complete_job, load_jobs, post_job, save_jobs, JobError};

#[derive(Parser)]
#[command(name = "jobmanager")]
#[command(about = "Prototype CLI for managing AI training jobs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Post a new job
    Post {
        /// Description of the job
        description: String,
        /// Reward offered for the job
        reward: u64,
    },
    /// Assign a worker to a job
    Assign {
        /// Job ID
        job_id: u64,
        /// Worker identifier
        worker: String,
    },
    /// Mark a job as completed
    Complete {
        /// Job ID
        job_id: u64,
    },
    /// List all jobs
    List,
}

fn main() -> Result<(), JobError> {
    let cli = Cli::parse();
    let mut jobs = load_jobs()?;

    match cli.command {
        Commands::Post { description, reward } => {
            let job = post_job(&mut jobs, description, reward);
            save_jobs(&jobs)?;
            println!("Posted job #{}", job.id);
        }
        Commands::Assign { job_id, worker } => {
            if assign_job(&mut jobs, job_id, worker).is_some() {
                save_jobs(&jobs)?;
                println!("Assigned job #{job_id}");
            } else {
                println!("Job #{job_id} not found");
            }
        }
        Commands::Complete { job_id } => {
            if complete_job(&mut jobs, job_id).is_some() {
                save_jobs(&jobs)?;
                println!("Completed job #{job_id}");
            } else {
                println!("Job #{job_id} not found");
            }
        }
        Commands::List => {
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

    Ok(())
}
