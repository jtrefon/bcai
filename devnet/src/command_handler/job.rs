use super::core::{CommandHandler, JobQueue};
use runtime::job::Job;
use std::error::Error;
use tracing::info;

impl CommandHandler {
    /// Handle job-related commands (e.g., submit training jobs).
    pub async fn handle_job_command(
        &mut self,
        job_command: crate::cli::JobCommands,
    ) -> Result<String, Box<dyn Error>> {
        match job_command {
            crate::cli::JobCommands::Submit {
                model_id,
                dataset_id,
                iterations,
            } => {
                let job_id = self.job_id_counter;
                self.job_id_counter += 1;

                let job = Job::new(job_id, model_id, dataset_id, iterations);
                self.job_queue.lock().await.push_back(job.clone());

                info!("Added new job to queue: {:?}", job);
                Ok(format!("Submitted job with ID: {}", job_id))
            }
        }
    }
} 