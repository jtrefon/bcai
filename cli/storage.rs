use runtime::large_data_transfer::{pricing, redundancy::RedundancyPolicy};
use schnorrkel::SecretKey;
use runtime::blockchain::transaction::Transaction;
use runtime::blockchain::validation;
use uuid::Uuid;
use std::fs::{File, create_dir_all};
use std::io::{Read, Write, BufReader};
use std::path::PathBuf;
use runtime::large_data_transfer::chunk::ChunkId;
use runtime::large_data_transfer::descriptor::LargeDataDescriptor;

const PRICE_PER_GIB_BCAI: u128 = 10; // flat rate per GiB per copy (placeholder)
const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

pub async fn handle_store(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: store <FILE> [--copies N] [--quote-only] --key <SECRET_KEY> --nonce <N> [--fee FEE]");
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

    // Required params for transaction creation
    let key_path = parse_value(args, "--key")
        .ok_or("--key <SECRET_KEY_FILE> is required unless --quote-only is used")?;
    let nonce: u64 = parse_value(args, "--nonce")
        .ok_or("--nonce <N> is required unless --quote-only is used")?
        .parse()?;
    let fee: u64 = parse_value(args, "--fee").unwrap_or("1".into()).parse()?;

    // Load secret key
    let secret_key_bytes = std::fs::read(&key_path)?;
    let secret_key = SecretKey::from_bytes(&secret_key_bytes)
        .map_err(|e| format!("Invalid secret key: {}", e))?;

    // Generate placeholder descriptor hash – in real implementation use content hash/Merkle root.
    let descriptor_hash = format!("file-{}", Uuid::new_v4());

    // === Chunking & local persistence ===
    let home = std::env::var("HOME")?;
    let base_dir = PathBuf::from(home).join(".bcai/dfs");
    let chunk_dir = base_dir.join("chunks");
    let desc_dir = base_dir.join("descriptors");
    create_dir_all(&chunk_dir)?;
    create_dir_all(&desc_dir)?;

    let mut chunk_hashes: Vec<String> = Vec::new();
    let file = File::open(&file_path)?;
    let mut reader = BufReader::new(file);
    loop {
        let mut buf = vec![0u8; CHUNK_SIZE];
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        buf.truncate(n);
        let chunk_id = ChunkId::from_data(&buf);
        let chunk_path = chunk_dir.join(format!("{}.bin", chunk_id.0));
        if !chunk_path.exists() {
            let mut f = File::create(&chunk_path)?;
            f.write_all(&buf)?;
        }
        chunk_hashes.push(chunk_id.0);
    }

    let descriptor = LargeDataDescriptor {
        id: descriptor_hash.clone(),
        content_hash: descriptor_hash.clone(),
        size_bytes: bytes as u64,
        chunk_hashes: chunk_hashes.clone(),
    };
    let desc_path = desc_dir.join(format!("{}.json", descriptor_hash));
    if !desc_path.exists() {
        let json = serde_json::to_string_pretty(&descriptor)?;
        std::fs::write(&desc_path, json)?;
    }

    let tx = Transaction::new_store_file_signed(
        &secret_key,
        descriptor_hash.clone(),
        bytes,
        quote.price_bcai,
        Vec::new(), // replica_nodes to be allocated later
        fee,
        nonce,
    );

    // Basic stateless validation (signature etc.)
    validation::validate_transaction_stateless(&tx)?;

    println!("📝 Created storage transaction (hash: {}):\n{}", tx.hash(), serde_json::to_string_pretty(&tx)?);
    println!("⚠️  Broadcasting to network not implemented yet – TODO");
    Ok(())
}

fn parse_copies(args: &[String]) -> Option<u8> {
    args.windows(2)
        .find(|w| w[0] == "--copies")
        .and_then(|w| w[1].parse::<u8>().ok())
}

fn parse_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
} 