use super::core::Transaction;
use crate::blockchain::constants::SIGNING_CONTEXT;
use schnorrkel::{signing_context, Signature, PublicKey, SecretKey};

impl Transaction {
    /// Create and sign a value transfer.
    pub fn new_transfer(
        from_secret_key: &SecretKey,
        to_public_key: PublicKey,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Self {
        let signer_pk = from_secret_key.to_public();
        let mut tx = Transaction {
            from: hex::encode(signer_pk.to_bytes()),
            to: hex::encode(to_public_key.to_bytes()),
            amount,
            fee,
            nonce,
            storage: None,
            signature: None,
        };
        // Sign hash bytes.
        let msg = tx.to_hash_bytes();
        let sig = from_secret_key.sign(signing_context(SIGNING_CONTEXT).bytes(&msg), &signer_pk);
        tx.signature = Some(hex::encode(sig.to_bytes()));
        tx
    }

    /// Verify Schnorrkel signature matches `from` field.
    pub fn verify_signature(&self) -> bool {
        let signer_bytes = match hex::decode(&self.from) { Ok(b) => b, Err(_) => return false };
        let signer_pk = match PublicKey::from_bytes(&signer_bytes) { Ok(pk) => pk, Err(_) => return false };
        let sig_bytes = match hex::decode(self.signature.clone().unwrap_or_default()) { Ok(b) => b, Err(_) => return false };
        let signature = match Signature::from_bytes(&sig_bytes) { Ok(s) => s, Err(_) => return false };
        signer_pk.verify(signing_context(SIGNING_CONTEXT).bytes(&self.to_hash_bytes()), &signature).is_ok()
    }

    /// Create, populate and sign a StorageTx::StoreFile transaction.
    pub fn new_store_file_signed(
        from_secret_key: &SecretKey,
        descriptor_hash: String,
        total_bytes: u128,
        price: u128,
        replica_nodes: Vec<String>,
        fee: u64,
        nonce: u64,
    ) -> Self {
        let signer_pk = from_secret_key.to_public();
        let mut tx = Transaction::new_store_file(
            hex::encode(signer_pk.to_bytes()),
            descriptor_hash,
            total_bytes,
            price,
            replica_nodes,
            fee,
            nonce,
        );

        let msg = tx.to_hash_bytes();
        let sig = from_secret_key.sign(signing_context(SIGNING_CONTEXT).bytes(&msg), &signer_pk);
        tx.signature = Some(hex::encode(sig.to_bytes()));
        tx
    }

    /// Create and sign an admin UpdateMetrics transaction (fee 0). Only oracle key allowed.
    pub fn new_update_metrics_signed(
        oracle_secret_key: &SecretKey,
        metrics: Vec<crate::distributed_storage::allocation::NodeMetrics>,
        nonce: u64,
    ) -> Self {
        let signer_pk = oracle_secret_key.to_public();
        let mut tx = Transaction {
            from: hex::encode(signer_pk.to_bytes()),
            to: String::new(),
            amount: 0,
            fee: 0,
            nonce,
            storage: Some(super::core::StorageTx::UpdateMetrics { metrics }),
            signature: None,
        };

        let msg = tx.to_hash_bytes();
        let sig = oracle_secret_key.sign(signing_context(SIGNING_CONTEXT).bytes(&msg), &signer_pk);
        tx.signature = Some(hex::encode(sig.to_bytes()));
        tx
    }
} 