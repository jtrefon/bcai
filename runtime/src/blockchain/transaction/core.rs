use serde::{Deserialize, Serialize};

/// A signed value-transfer transaction on the chain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64, fee: u64, nonce: u64) -> Self {
        Self { from, to, amount, fee, nonce, signature: None }
    }

    /// Lightweight accessor for signer hex string.
    pub fn signer(&self) -> &String { &self.from }
    /// Recipient hex string.
    pub fn recipient(&self) -> &String { &self.to }

    /// Serialize fields (excluding signature) into deterministic byte vector –
    /// used by hashing & signing routines.
    pub fn to_hash_bytes(&self) -> Vec<u8> {
        let mut buf = self.from.as_bytes().to_vec();
        buf.extend_from_slice(self.to.as_bytes());
        buf.extend_from_slice(&self.amount.to_le_bytes());
        buf.extend_from_slice(&self.fee.to_le_bytes());
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf
    }
} 