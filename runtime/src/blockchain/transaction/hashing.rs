use super::core::Transaction;
use sha2::{Digest, Sha256};

impl Transaction {
    /// Compute deterministic SHA-256 hash of the transaction (including signature).
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.to_hash_bytes());
        hasher.update(self.signature.clone().unwrap_or_default().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Calculate a simple Merkle root over an ordered list of transactions.
    pub fn merkle_root(txs: &[Transaction]) -> String {
        if txs.is_empty() {
            return "0".repeat(64);
        }
        let mut hashes: Vec<String> = txs.iter().map(|tx| tx.hash()).collect();
        if hashes.len() % 2 != 0 { hashes.push(hashes.last().cloned().unwrap()); }
        while hashes.len() > 1 {
            let mut next = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0].as_bytes());
                if let Some(h2) = chunk.get(1) { hasher.update(h2.as_bytes()); }
                next.push(hex::encode(hasher.finalize()));
            }
            hashes = next;
        }
        hashes.pop().unwrap()
    }
} 