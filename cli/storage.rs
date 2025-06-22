use runtime::large_data_transfer::{pricing, redundancy::RedundancyPolicy};
use std::path::Path;

const PRICE_PER_GIB_BCAI: u128 = 10; // flat rate per GiB per copy (placeholder)

pub async fn handle_store(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: store <FILE> [--copies N] [--quote-only]");
        return Ok(());
    }

    let file_path = &args[0];
    let copies = parse_copies(args).unwrap_or(1);
    let quote_only = args.iter().any(|a| a == "--quote-only");

    let bytes = std::fs::metadata(file_path)?.len() as u128;
    let policy = RedundancyPolicy { copies, geo_spread: true };
    let quote = pricing::quote(bytes, policy, PRICE_PER_GIB_BCAI);

    println!("💰 Price Quote: storing {:.2} GiB with {} extra copies = {} BCAI", quote.total_bytes as f64/1.073741824e9, copies, quote.price_bcai);
    if quote_only { return Ok(()); }

    // TODO: integrate actual upload logic (split, push chunks, etc.)
    println!("🚀 Uploading {file_path} ... (not yet implemented)");
    Ok(())
}

fn parse_copies(args: &[String]) -> Option<u8> {
    args.windows(2)
        .find(|w| w[0] == "--copies")
        .and_then(|w| w[1].parse::<u8>().ok())
} 