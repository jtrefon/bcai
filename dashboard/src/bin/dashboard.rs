use clap::{Parser, Subcommand};
use dashboard::{create_dashboard_server, DashboardConfig};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "dashboard")]
#[command(about = "BCAI Network Dashboard")]
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
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        
        /// Devnet node address to connect to
        #[arg(long, default_value = "127.0.0.1:8080")]
        node: String,
        
        /// Enable development mode (hot reload, debug logs)
        #[arg(long)]
        dev: bool,
    },
    /// Show dashboard information
    Info,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port, host, node, dev } => {
            start_dashboard(port, &host, &node, dev).await?;
        }
        Commands::Info => {
            show_info().await?;
        }
    }

    Ok(())
}

async fn start_dashboard(
    port: u16,
    host: &str,
    node_addr: &str,
    dev_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let config = DashboardConfig {
        listen_addr: addr,
        node_address: node_addr.to_string(),
        dev_mode,
        static_files_path: "dashboard/static".to_string(),
        websocket_enabled: true,
        refresh_interval_seconds: 5,
    };

    println!("🌐 Starting BCAI Network Dashboard");
    println!("=================================");
    println!("📊 Dashboard URL: http://{}", addr);
    println!("🔗 Node Address: {}", node_addr);
    println!("🔧 Development Mode: {}", if dev_mode { "✅" } else { "❌" });
    println!();
    
    println!("✨ Features Available:");
    println!("  • 📈 Real-time network metrics");
    println!("  • 🧠 Enhanced VM monitoring");
    println!("  • 📋 ML job queue status");
    println!("  • 🔍 Transaction explorer");
    println!("  • 🎯 Performance analytics");
    println!("  • 🔧 Node management");
    println!();
    
    println!("🚀 Starting server...");
    
    create_dashboard_server(config).await?;
    
    Ok(())
}

async fn show_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 BCAI Network Dashboard");
    println!("=========================");
    println!();
    
    println!("🌟 Features:");
    println!("  📈 Real-time Monitoring:");
    println!("    • Network hashrate and difficulty");
    println!("    • Block production metrics");
    println!("    • Transaction throughput");
    println!("    • Node connectivity status");
    println!();
    
    println!("  🧠 Enhanced VM Analytics:");
    println!("    • ML job execution statistics");
    println!("    • Python bridge usage metrics");
    println!("    • GPU utilization graphs");
    println!("    • Memory consumption tracking");
    println!();
    
    println!("  📋 Job Management:");
    println!("    • Active job queue visualization");
    println!("    • Job completion rates");
    println!("    • Error analysis and logs");
    println!("    • Performance benchmarks");
    println!();
    
    println!("  🔍 Network Explorer:");
    println!("    • Block explorer with ML transaction details");
    println!("    • Smart contract execution traces");
    println!("    • Network topology visualization");
    println!("    • Peer discovery and connectivity");
    println!();
    
    println!("🔧 Technical Specifications:");
    println!("  • Built with modern web technologies");
    println!("  • Real-time WebSocket updates");
    println!("  • Responsive design for all devices");
    println!("  • RESTful API endpoints");
    println!("  • Secure authentication");
    println!();
    
    println!("🚀 Quick Start:");
    println!("  1. Start a devnet node: devnet start --rpc");
    println!("  2. Launch dashboard: dashboard start");
    println!("  3. Open http://localhost:3000 in your browser");
    println!("  4. Explore the network in real-time!");
    
    Ok(())
} 