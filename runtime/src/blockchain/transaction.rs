use crate::blockchain::constants::SIGNING_CONTEXT;
use schnorrkel::{signing_context, PublicKey, SecretKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
        Self {
            from,
            to,
            amount,
            fee,
            nonce,
            signature: None,
        }
    }

    /// Creates a hash of the transaction for signing.
    pub fn to_hash_bytes(&self) -> Vec<u8> {
        let mut transaction_data = self.from.as_bytes().to_vec();
        transaction_data.extend_from_slice(self.to.as_bytes());
        transaction_data.extend_from_slice(&self.amount.to_le_bytes());
        transaction_data.extend_from_slice(&self.fee.to_le_bytes());
        transaction_data.extend_from_slice(&self.nonce.to_le_bytes());
        transaction_data
    }

    /// Creates and signs a new transfer transaction.
    pub fn new_transfer(
        from_secret_key: &SecretKey,
        to_public_key: PublicKey,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Self {
        let signer_pubkey = from_secret_key.public_key();
        let mut tx_for_signing = Transaction {
            from: hex::encode(signer_pubkey.to_bytes()),
            to: hex::encode(to_public_key.to_bytes()),
            amount,
            fee,
            nonce,
            // A placeholder signature is created and then immediately replaced.
            signature: Some(hex::encode(from_secret_key.sign(signing_context(SIGNING_CONTEXT).bytes(b"placeholder")).to_bytes())),
        };

        let message = tx_for_signing.to_hash_bytes();
        let signature = from_secret_key.sign(signing_context(SIGNING_CONTEXT).bytes(&message));
        tx_for_signing.signature = Some(hex::encode(signature.to_bytes()));

        tx_for_signing
    }

    /// Verifies the transaction's signature.
    pub fn verify_signature(&self) -> bool {
        let signer_bytes = match hex::decode(&self.from) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let signer_pubkey = match PublicKey::from_bytes(&signer_bytes) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        let message = self.to_hash_bytes();
        let signature_bytes = match hex::decode(&self.signature.clone().unwrap_or_default()) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let signature = match Signature::from_bytes(&signature_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        signer_pubkey
            .verify(
                signing_context(SIGNING_CONTEXT).bytes(&message),
                &signature,
            )
            .is_ok()
    }

    /// Computes a unique and deterministic hash for the transaction.
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.to_hash_bytes());
        hasher.update(self.signature.clone().unwrap_or_default().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Calculates the Merkle root of a set of transactions.
    pub fn merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64);
        }
        let mut hashes: Vec<String> = transactions.iter().map(|tx| tx.hash()).collect();
        // Ensure the number of hashes is even.
        if hashes.len() % 2 != 0 {
            if let Some(last) = hashes.last().cloned() {
                hashes.push(last);
            }
        }
        while hashes.len() > 1 {
            let mut next_level_hashes = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0].as_bytes());
                if let Some(h2) = chunk.get(1) {
                    hasher.update(h2.as_bytes());
                }
                next_level_hashes.push(hex::encode(hasher.finalize()));
            }
            hashes = next_level_hashes;
        }
        hashes.pop().unwrap_or_else(|| "0".repeat(64))
    }
} 