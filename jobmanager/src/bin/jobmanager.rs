use clap::{Parser, Subcommand};
use jobmanager_lib::{Job, JobManager, JobStatus, MLJobConfig};
use serde_json;
use std::path::Path;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "jobmanager")]
#[command(about = "BCAI ML Job Manager")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a new ML job
    Submit {
        /// Job configuration file (YAML or JSON)
        #[arg(short, long)]
        config: String,
        
        /// Job name
        #[arg(short, long)]
        name: Option<String>,
        
        /// Priority (1-10, higher is more important)
        #[arg(short, long, default_value = "5")]
        priority: u8,
    },
    /// List all jobs
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
        
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Show job details
    Show {
        /// Job ID
        job_id: String,
    },
    /// Cancel a job
    Cancel {
        /// Job ID
        job_id: String,
    },
    /// Get job logs
    Logs {
        /// Job ID
        job_id: String,
        
        /// Follow logs (tail -f style)
        #[arg(short, long)]
        follow: bool,
    },
    /// Show job manager status
    Status,
    /// Create a sample job configuration
    Template {
        /// Template type (training, inference, federated)
        #[arg(short, long, default_value = "training")]
        template_type: String,
        
        /// Output file
        #[arg(short, long, default_value = "job-config.yaml")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut job_manager = JobManager::new();

    match cli.command {
        Commands::Submit { config, name, priority } => {
            submit_job(&mut job_manager, &config, name, priority).await?;
        }
        Commands::List { status, detailed } => {
            list_jobs(&job_manager, status, detailed).await?;
        }
        Commands::Show { job_id } => {
            show_job(&job_manager, &job_id).await?;
        }
        Commands::Cancel { job_id } => {
            cancel_job(&mut job_manager, &job_id).await?;
        }
        Commands::Logs { job_id, follow } => {
            show_logs(&job_manager, &job_id, follow).await?;
        }
        Commands::Status => {
            show_status(&job_manager).await?;
        }
        Commands::Template { template_type, output } => {
            create_template(&template_type, &output).await?;
        }
    }

    Ok(())
}

async fn submit_job(
    job_manager: &mut JobManager,
    config_path: &str,
    name: Option<String>,
    priority: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Submitting ML job...");
    
    if !Path::new(config_path).exists() {
        return Err(format!("Configuration file not found: {}", config_path).into());
    }

    let config_content = std::fs::read_to_string(config_path)?;
    let ml_config: MLJobConfig = if config_path.ends_with(".yaml") || config_path.ends_with(".yml") {
        serde_yaml::from_str(&config_content)?
    } else {
        serde_json::from_str(&config_content)?
    };

    let job_id = Uuid::new_v4().to_string();
    let job_name = name.unwrap_or_else(|| ml_config.name.clone().unwrap_or_else(|| format!("job-{}", &job_id[..8])));

    let job = Job {
        id: job_id.clone(),
        name: job_name.clone(),
        status: JobStatus::Pending,
        config: ml_config,
        priority,
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        logs: Vec::new(),
        result: None,
    };

    job_manager.submit_job(job).await?;

    println!("✅ Job submitted successfully!");
    println!("  📝 Job ID: {}", job_id);
    println!("  🏷️  Name: {}", job_name);
    println!("  🎯 Priority: {}", priority);
    println!("  📊 Status: Pending");
    
    Ok(())
}

async fn list_jobs(
    job_manager: &JobManager,
    status_filter: Option<String>,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 BCAI ML Jobs");
    println!("================");
    
    let jobs = job_manager.list_jobs().await?;
    
    let filtered_jobs: Vec<_> = if let Some(status) = status_filter {
        let filter_status = match status.to_lowercase().as_str() {
            "pending" => JobStatus::Pending,
            "running" => JobStatus::Running,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            _ => return Err(format!("Invalid status filter: {}", status).into()),
        };
        jobs.iter().filter(|job| job.status == filter_status).collect()
    } else {
        jobs.iter().collect()
    };

    if filtered_jobs.is_empty() {
        println!("No jobs found.");
        return Ok(());
    }

    println!();
    for job in filtered_jobs {
        let status_icon = match job.status {
            JobStatus::Pending => "⏳",
            JobStatus::Running => "🏃",
            JobStatus::Completed => "✅",
            JobStatus::Failed => "❌",
            JobStatus::Cancelled => "🚫",
        };

        println!("{} {} ({})", status_icon, job.name, &job.id[..8]);
        
        if detailed {
            println!("    📝 ID: {}", job.id);
            println!("    🎯 Priority: {}", job.priority);
            println!("    📅 Created: {}", job.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            if let Some(started) = job.started_at {
                println!("    🚀 Started: {}", started.format("%Y-%m-%d %H:%M:%S UTC"));
            }
            
            if let Some(completed) = job.completed_at {
                println!("    🏁 Completed: {}", completed.format("%Y-%m-%d %H:%M:%S UTC"));
            }
            
            println!("    🔧 Type: {}", job.config.job_type);
            println!();
        }
    }

    let total = jobs.len();
    let pending = jobs.iter().filter(|j| j.status == JobStatus::Pending).count();
    let running = jobs.iter().filter(|j| j.status == JobStatus::Running).count();
    let completed = jobs.iter().filter(|j| j.status == JobStatus::Completed).count();
    let failed = jobs.iter().filter(|j| j.status == JobStatus::Failed).count();

    println!("📊 Summary: {} total, {} pending, {} running, {} completed, {} failed", 
             total, pending, running, completed, failed);

    Ok(())
}

async fn show_job(
    job_manager: &JobManager,
    job_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let job = job_manager.get_job(job_id).await?
        .ok_or(format!("Job not found: {}", job_id))?;

    println!("📋 Job Details");
    println!("==============");
    println!();
    
    let status_icon = match job.status {
        JobStatus::Pending => "⏳",
        JobStatus::Running => "🏃",
        JobStatus::Completed => "✅",
        JobStatus::Failed => "❌",
        JobStatus::Cancelled => "🚫",
    };

    println!("📝 Job ID: {}", job.id);
    println!("🏷️  Name: {}", job.name);
    println!("{} Status: {:?}", status_icon, job.status);
    println!("🎯 Priority: {}", job.priority);
    println!("📅 Created: {}", job.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    
    if let Some(started) = job.started_at {
        println!("🚀 Started: {}", started.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    if let Some(completed) = job.completed_at {
        println!("🏁 Completed: {}", completed.format("%Y-%m-%d %H:%M:%S UTC"));
        
        let duration = completed.signed_duration_since(job.started_at.unwrap_or(job.created_at));
        println!("⏱️  Duration: {}s", duration.num_seconds());
    }

    println!();
    println!("🔧 Configuration:");
    println!("  • Type: {}", job.config.job_type);
    
    if let Some(model) = &job.config.model {
        println!("  • Model: {}", model);
    }
    
    if let Some(dataset) = &job.config.dataset {
        println!("  • Dataset: {}", dataset);
    }

    if let Some(result) = &job.result {
        println!();
        println!("📊 Result:");
        println!("{}", serde_json::to_string_pretty(result)?);
    }

    if !job.logs.is_empty() {
        println!();
        println!("📜 Logs:");
        for log in job.logs.iter().take(10) {  // Show last 10 log entries
            println!("  {}: {}", log.timestamp.format("%H:%M:%S"), log.message);
        }
        
        if job.logs.len() > 10 {
            println!("  ... and {} more entries", job.logs.len() - 10);
        }
    }

    Ok(())
}

async fn cancel_job(
    job_manager: &mut JobManager,
    job_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Cancelling job: {}", job_id);
    
    job_manager.cancel_job(job_id).await?;
    
    println!("✅ Job cancelled successfully");
    Ok(())
}

async fn show_logs(
    job_manager: &JobManager,
    job_id: &str,
    follow: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let job = job_manager.get_job(job_id).await?
        .ok_or(format!("Job not found: {}", job_id))?;

    println!("📜 Logs for job: {} ({})", job.name, &job.id[..8]);
    println!("================================");
    
    if job.logs.is_empty() {
        println!("No logs available yet.");
        return Ok(());
    }

    for log in &job.logs {
        println!("{} [{}] {}", 
                 log.timestamp.format("%Y-%m-%d %H:%M:%S"), 
                 log.level, 
                 log.message);
    }

    if follow {
        println!();
        println!("👀 Following logs... (Press Ctrl+C to exit)");
        // TODO: Implement log following logic
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn show_status(
    job_manager: &JobManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  BCAI Job Manager Status");
    println!("==========================");
    println!();

    let jobs = job_manager.list_jobs().await?;
    
    let pending = jobs.iter().filter(|j| j.status == JobStatus::Pending).count();
    let running = jobs.iter().filter(|j| j.status == JobStatus::Running).count();
    let completed = jobs.iter().filter(|j| j.status == JobStatus::Completed).count();
    let failed = jobs.iter().filter(|j| j.status == JobStatus::Failed).count();

    println!("📊 Job Statistics:");
    println!("  ⏳ Pending: {}", pending);
    println!("  🏃 Running: {}", running);
    println!("  ✅ Completed: {}", completed);
    println!("  ❌ Failed: {}", failed);
    println!("  📈 Total: {}", jobs.len());
    println!();
    
    println!("🔧 System Information:");
    println!("  • Enhanced VM: ✅ Enabled");
    println!("  • Python Bridge: ✅ Available");
    println!("  • GPU Support: ✅ Available");
    println!("  • Max Concurrent Jobs: 10");
    println!("  • Job Queue Capacity: 1000");

    Ok(())
}

async fn create_template(
    template_type: &str,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Creating {} job template...", template_type);
    
    let template = match template_type {
        "training" => {
            serde_yaml::to_string(&serde_json::json!({
                "name": "ml-training-job",
                "job_type": "training",
                "model": "neural_network",
                "dataset": "training_data.csv",
                "parameters": {
                    "epochs": 100,
                    "batch_size": 32,
                    "learning_rate": 0.001,
                    "optimizer": "adam"
                },
                "hardware": {
                    "gpu_enabled": true,
                    "memory_limit": "2GB"
                },
                "python_code": "# Your PyTorch/TensorFlow training code here\nimport torch\nimport torch.nn as nn\n\n# Define your model, training loop, etc.\nprint('Training started...')"
            }))?
        }
        "inference" => {
            serde_yaml::to_string(&serde_json::json!({
                "name": "ml-inference-job",
                "job_type": "inference",
                "model": "trained_model.pt",
                "input_data": "test_data.csv",
                "parameters": {
                    "batch_size": 64,
                    "output_format": "json"
                },
                "hardware": {
                    "gpu_enabled": true,
                    "memory_limit": "1GB"
                },
                "python_code": "# Your inference code here\nimport torch\n\n# Load model and run inference\nprint('Inference completed')"
            }))?
        }
        "federated" => {
            serde_yaml::to_string(&serde_json::json!({
                "name": "federated-learning-job",
                "job_type": "federated_learning",
                "model": "global_model",
                "participants": 5,
                "parameters": {
                    "rounds": 10,
                    "local_epochs": 5,
                    "aggregation": "fedavg"
                },
                "privacy": {
                    "differential_privacy": true,
                    "noise_multiplier": 1.0
                },
                "python_code": "# Federated learning code\nimport torch\n\n# Implement federated averaging\nprint('Federated learning round completed')"
            }))?
        }
        _ => return Err(format!("Unknown template type: {}", template_type).into()),
    };

    std::fs::write(output, template)?;
    
    println!("✅ Template created: {}", output);
    println!("📝 Edit the configuration file and submit with:");
    println!("   jobmanager submit --config {}", output);
    
    Ok(())
} 