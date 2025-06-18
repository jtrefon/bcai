use crate::config::DevnetConfig;
use crate::error::DevnetError;
use crate::job::post_job;
use crate::ledger::{mint, TREASURY};
use crate::persistence::{load_jobs, load_ledger, save_jobs, save_ledger};

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
        println!(
            "🤖 Created AI worker: {} with {} tokens",
            worker_id,
            config.initial_tokens / 2
        );
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
        100,
    )?;

    save_jobs(&jobs)?;
    save_ledger(&ledger)?;

    println!("🎉 Devnet initialized successfully!");
    println!(
        "📊 Total nodes: {}, AI workers: {}",
        config.node_count, config.ai_workers
    );
    println!("💰 Treasury balance: {}", ledger.balance(TREASURY));
    println!("📝 Available jobs: {}", jobs.len());

    Ok(())
} 