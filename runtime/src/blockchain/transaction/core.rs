use serde::{Deserialize, Serialize};

/// Storage-related transaction payloads
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StorageTx {
    /// User pays `price` for storing a descriptor representing `total_bytes`.
    StoreFile {
        descriptor_hash: String,
        total_bytes: u128,
        price: u128,
        replica_nodes: Vec<String>,
    },
    /// Node claims reward for holding a descriptor replica.
    RewardHolding {
        descriptor_hash: String,
        node_id: String,
        reward: u128,
    },
    /// Admin transaction carrying latest Node metrics snapshot.
    UpdateMetrics {
        metrics: Vec<crate::distributed_storage::allocation::NodeMetrics>,
    },
}

/// A signed value-transfer transaction on the chain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    /// Optional storage payload (None for ordinary transfers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageTx>,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64, fee: u64, nonce: u64) -> Self {
        Self { from, to, amount, fee, nonce, storage: None, signature: None }
    }

    /// Convenience constructor for file-storage payment
    pub fn new_store_file(
        from: String,
        descriptor_hash: String,
        total_bytes: u128,
        price: u128,
        replica_nodes: Vec<String>,
        fee: u64,
        nonce: u64,
    ) -> Self {
        Self {
            from,
            to: "".into(),
            amount: 0,
            fee,
            nonce,
            storage: Some(StorageTx::StoreFile { descriptor_hash, total_bytes, price, replica_nodes }),
            signature: None,
        }
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
        if let Some(ref payload) = self.storage {
            let payload_bytes = bincode::serialize(payload).expect("serialize payload");
            buf.extend_from_slice(&payload_bytes);
        }
        buf
    }
} 