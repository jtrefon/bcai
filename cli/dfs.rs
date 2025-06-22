use runtime::large_data_transfer::{pricing, redundancy::RedundancyPolicy};
use runtime::distributed_storage::{run_auto_heal, ReplicationManager, StorageNode};
use std::sync::Arc;

const PRICE_PER_GIB_BCAI: u128 = 10;

pub async fn handle_dfs(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        print_help();
        return Ok(());
    }
    match args[0].as_str() {
        "quote" => quote_price(&args[1..])?,
        "rebalance" => {
            println!("🔄 Triggering network rebalance (auto-heal)…");
            // Stub – create empty managers for now
            let cm = Arc::new(runtime::large_data_transfer::manager::ChunkManager::default());
            let repl = ReplicationManager { nodes: Vec::new() };
            tokio::spawn(run_auto_heal(cm, repl, 3));
        },
        "stats" => {
            println!("📊 DFS Stats: (placeholder – real metrics TBD)");
            println!("  • Total nodes      : 0");
            println!("  • Total replicas   : 0");
            println!("  • Avg reliability  : 0%\n");
        },
        _ => print_help(),
    }
    Ok(())
}

fn quote_price(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: dfs quote <FILE> [--copies N]");
        return Ok(());
    }
    let file = &args[0];
    let copies = parse_copies(args).unwrap_or(1);
    let bytes = std::fs::metadata(file)?.len() as u128;
    let policy = RedundancyPolicy { copies, geo_spread: true };
    let quote = pricing::quote(bytes, policy, PRICE_PER_GIB_BCAI);
    println!("💰 Quote: {:.2} GiB, {} copies → {} BCAI", quote.total_bytes as f64/1.073741824e9, copies, quote.price_bcai);
    Ok(())
}

fn parse_copies(args: &[String]) -> Option<u8> {
    args.windows(2)
        .find(|w| w[0] == "--copies")
        .and_then(|w| w[1].parse::<u8>().ok())
}

fn print_help() {
    println!("DFS subcommands:");
    println!("  dfs quote <FILE> [--copies N]   – price estimation");
    println!("  dfs rebalance                   – trigger auto-heal");
    println!("  dfs stats                       – show storage stats");
} 