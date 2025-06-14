use clap::{Parser, Subcommand};
use devnet::*;
use runtime::token::LedgerError;
use runtime::token::TokenLedger;

#[derive(Parser)]
#[command(name = "devnet")]
#[command(about = "Dev network CLI with token and staking", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ledger file
    Init,
    /// Mint tokens
    Mint { account: String, amount: u64 },
    /// Transfer tokens
    Transfer { from: String, to: String, amount: u64 },
    /// Stake tokens
    Stake { account: String, amount: u64 },
    /// Unstake tokens
    Unstake { account: String, amount: u64 },
    /// Slash staked tokens to the treasury
    Slash { account: String, amount: u64 },
    /// Burn tokens from an account
    Burn { account: String, amount: u64 },
    /// Show balances
    Balance { account: String },
    /// Show reputation score
    Reputation { account: String },
    /// Adjust reputation by delta
    AdjustRep { account: String, delta: i32 },
    /// Mine a block executing a dummy GPU task
    Mine,
    /// Run a PoUW training task
    Train { size: usize, seed: u64, difficulty: u32 },
    /// Train a logistic regression model on the digits dataset
    Mnist,
    /// Train a neural network
    Neural {
        #[arg(short, long, value_delimiter = ',')]
        layers: Vec<usize>,
        #[arg(short, long, default_value = "10")]
        epochs: usize,
        #[arg(short, long, default_value = "100")]
        samples: usize,
    },
    /// Manage jobs
    Job {
        #[command(subcommand)]
        job: JobCommands,
    },
}

#[derive(Subcommand)]
enum JobCommands {
    /// Post a new job
    Post { poster: String, description: String, reward: u64 },
    /// Assign a worker
    Assign { job_id: u64, worker: String },
    /// Complete a job
    Complete { job_id: u64 },
    /// List jobs
    List,
}

fn main() -> Result<(), DevnetError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            save_ledger(&TokenLedger::new())?;
            println!("Initialized ledger");
        }
        _ => {
            let mut ledger = load_ledger()?;
            match cli.command {
                Commands::Mint { account, amount } => {
                    mint(&mut ledger, &account, amount);
                }
                Commands::Transfer { from, to, amount } => {
                    match transfer(&mut ledger, &from, &to, amount) {
                        Ok(()) => {}
                        Err(LedgerError::InsufficientBalance) => {
                            println!("insufficient balance");
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(DevnetError::Io(std::io::Error::other(e.to_string())));
                        }
                    }
                }
                Commands::Stake { account, amount } => {
                    if let Err(e) = stake(&mut ledger, &account, amount) {
                        println!("{e}");
                    }
                }
                Commands::Unstake { account, amount } => {
                    if let Err(e) = unstake(&mut ledger, &account, amount) {
                        println!("{e}");
                    }
                }
                Commands::Slash { account, amount } => {
                    if let Err(e) = slash(&mut ledger, &account, amount) {
                        println!("{e}");
                    }
                }
                Commands::Burn { account, amount } => {
                    if let Err(e) = burn(&mut ledger, &account, amount) {
                        println!("{e}");
                    }
                }
                Commands::Balance { account } => {
                    println!(
                        "balance: {} staked: {}",
                        ledger.balance(&account),
                        ledger.staked(&account)
                    );
                }
                Commands::Reputation { account } => {
                    println!("reputation: {}", reputation(&ledger, &account));
                }
                Commands::AdjustRep { account, delta } => {
                    adjust_reputation(&mut ledger, &account, delta);
                }
                Commands::Mine => {
                    let input = vec![1.0f32, 2.0, 3.0, 4.0];
                    match runtime::gpu::double_numbers(&input) {
                        Ok(res) => println!("mined block with result: {:?}", res),
                        Err(e) => println!("gpu task failed: {e}"),
                    }
                }
                Commands::Train { size, seed, difficulty } => {
                    if train_and_verify(size, seed, difficulty) {
                        println!("training succeeded");
                    } else {
                        println!("training failed");
                    }
                }
                Commands::Mnist => match runtime::mnist::train_digits() {
                    Ok(acc) => println!("digits training accuracy: {:.2}", acc),
                    Err(e) => println!("training failed: {e}"),
                },
                Commands::Neural { layers, epochs, samples } => {
                    match train_neural_network(layers.clone(), epochs, samples) {
                        Ok(metrics) => {
                            println!("Neural Network Training Results:");
                            println!("Architecture: {:?}", layers);
                            for metric in metrics {
                                println!(
                                    "  Epoch {}: loss={:.4}, accuracy={:.3}, time={}ms",
                                    metric.epoch,
                                    metric.loss,
                                    metric.accuracy,
                                    metric.training_time_ms
                                );
                            }
                        }
                        Err(e) => println!("neural network training failed: {e}"),
                    }
                }
                Commands::Job { job } => {
                    let mut jobs = load_jobs()?;
                    match job {
                        JobCommands::Post { poster, description, reward } => {
                            match post_job(&mut jobs, &mut ledger, &poster, description, reward) {
                                Ok(()) => {
                                    if let Some(j) = jobs.last() {
                                        println!("posted job #{}", j.id);
                                    }
                                }
                                Err(e) => println!("{e}"),
                            }
                        }
                        JobCommands::Assign { job_id, worker } => {
                            if let Err(e) = assign_job(&mut jobs, job_id, &worker) {
                                println!("{e}");
                            }
                        }
                        JobCommands::Complete { job_id } => {
                            if let Err(e) = complete_job(&mut jobs, &mut ledger, job_id) {
                                println!("{e}");
                            }
                        }
                        JobCommands::List => {
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
                }
                Commands::Init => unreachable!(),
            }
            save_ledger(&ledger)?;
        }
    }
    Ok(())
}

fn train_and_verify(size: usize, seed: u64, difficulty: u32) -> bool {
    let task = runtime::pouw::generate_task(size, seed);
    let trainer = runtime::trainer::Trainer::new("alice");
    let solution = trainer.train(&task, difficulty);
    let evaluator = runtime::evaluator::Evaluator::new("bob");
    evaluator.evaluate(&task, &solution, difficulty)
}

fn train_neural_network(
    layers: Vec<usize>,
    epochs: usize,
    samples: usize,
) -> Result<Vec<runtime::neural_network::TrainingMetrics>, String> {
    use runtime::neural_network::{generate_synthetic_data, NeuralNetwork};

    if layers.len() < 2 {
        return Err("Neural network must have at least 2 layers (input and output)".to_string());
    }

    // Create neural network
    let mut network = NeuralNetwork::new(&layers, 0.01);

    // Generate training data
    let input_size = layers[0];
    let output_size = layers[layers.len() - 1];
    let data = generate_synthetic_data(samples, input_size, output_size);

    // Train the network
    let metrics = network.train(&data, epochs as u32);

    Ok(metrics)
}
