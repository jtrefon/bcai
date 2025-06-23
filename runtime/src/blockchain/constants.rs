/// The reward a miner receives for successfully mining a new block.
pub const BLOCK_REWARD: u64 = 100;

/// The public key of the developer, pre-funded in the genesis block for testing.
pub const DEV_PUBLIC_KEY: &str = "d75a980182b10ab7d54bfed3c964073a0ee17e152516d0047913076135327269";

/// The initial amount the developer's account is funded with.
pub const DEV_FUNDING: u64 = 1_000_000_000;

/// The context for signing transactions to prevent replay attacks on other systems.
pub const SIGNING_CONTEXT: &[u8] = b"bcai-transaction";

/// Public key authorised to submit UpdateMetrics admin transactions.
pub const METRICS_ORACLE_PUB: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";