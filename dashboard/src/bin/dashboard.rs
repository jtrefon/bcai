use clap::{Parser, Subcommand};
use dashboard::serve;

#[derive(Parser)]
#[command(name = "dashboard")]
#[command(about = "Simple job dashboard HTTP server")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the dashboard web server
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port, host } => {
            let addr = format!("{}:{}", host, port);
            println!("🌐 Starting dashboard server at http://{}", addr);
            println!("📊 Visit http://{}/jobs to view jobs", addr);
            serve(&addr)?;
        }
    }

    Ok(())
} 