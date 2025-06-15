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

    match cli.command {
        Commands::Post { description, reward } => {
            let mut jobs = load_jobs()?;
            let job = post_job(&mut jobs, description, reward);
            save_jobs(&jobs)?;
            println!("✅ Job {} posted with reward {}", job.id, reward);
        }
        Commands::Assign { job_id, worker } => {
            let mut jobs = load_jobs()?;
            assign_job(&mut jobs, job_id, worker.clone())?;
            save_jobs(&jobs)?;
            println!("✅ Job {} assigned to {}", job_id, worker);
        }
        Commands::Complete { job_id } => {
            let mut jobs = load_jobs()?;
            complete_job(&mut jobs, job_id)?;
            save_jobs(&jobs)?;
            println!("✅ Job {} completed", job_id);
        }
        Commands::List => {
            let jobs = load_jobs()?;
            if jobs.is_empty() {
                println!("No jobs found.");
            } else {
                println!("{:<5} {:<20} {:<10} {:<15} {:<10}", "ID", "Description", "Reward", "Assigned To", "Status");
                println!("{}", "-".repeat(65));
                for job in jobs {
                    let assigned = job.assigned_to.as_deref().unwrap_or("None");
                    let status = if job.completed { "Completed" } else if job.assigned_to.is_some() { "Assigned" } else { "Open" };
                    println!("{:<5} {:<20} {:<10} {:<15} {:<10}", 
                        job.id, 
                        if job.description.len() > 18 { format!("{}...", &job.description[..15]) } else { job.description }, 
                        job.reward, 
                        assigned, 
                        status
                    );
                }
            }
        }
    }

    Ok(())
} 