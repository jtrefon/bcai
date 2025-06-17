/// A well-known development public key, pre-funded in the genesis block.
/// Corresponds to the secret key in `wallet.key`.
pub const DEV_GENESIS_PUBKEY: &str =
    "d8712b8424a1e2a043446654e39678125a6924a689b6f39572421397a618e7e8";
    
/// The fixed reward credited to a miner for successfully mining a new block.
pub const BLOCK_REWARD: u64 = 100;

/// The context for signing transactions to prevent replay attacks on other systems.
pub const SIGNING_CONTEXT: &[u8] = b"bcai-transaction"; 