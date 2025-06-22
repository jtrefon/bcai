mod cli;

use cli::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Production CLI v3.0.0");
    println!("📊 Enterprise-Grade AI Network Management");
    println!("═══════════════════════════════════════");

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        show_help();
        return Ok(());
    }

    match args[1].as_str() {
        "dashboard" => show_production_dashboard().await?,
        "deploy" => handle_deployment(&args[2..]).await?,
        "contract" => handle_smart_contracts(&args[2..]).await?,
        "monitor" => show_monitoring_system().await?,
        "network" => show_network_status().await?,
        "security" => show_security_status().await?,
        "store" => handle_store(&args[2..]).await?,
        "dfs" => handle_dfs(&args[2..]).await?,
        "test" => run_integration_tests().await?,
        _ => show_help(),
    }

    Ok(())
} 