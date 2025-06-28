//! Cryptography Implementation for Large Data Transfer
//!
//! Provides lightweight AES-GCM encryption utilities used for securing chunks
//! during transfer.  The goal is not to offer a full cryptographic framework but
//! simply to remove plain-text transmissions from the prototype.

use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use crate::large_data_transfer::error::{LargeDataError, LargeDataResult};

/// Encryption helper utilities.
pub struct CryptoUtils;

impl CryptoUtils {
    /// Encrypt `data` using AES-256-GCM.  The caller must provide a 32-byte key
    /// and a 12-byte nonce.
    pub fn encrypt(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> LargeDataResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(key.into());
        cipher
            .encrypt(Nonce::from_slice(nonce), data)
            .map_err(|e| LargeDataError::Encryption(e.to_string()))
    }

    /// Decrypt previously encrypted data with AES-256-GCM.
    pub fn decrypt(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> LargeDataResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(key.into());
        cipher
            .decrypt(Nonce::from_slice(nonce), data)
            .map_err(|e| LargeDataError::Encryption(e.to_string()))
    }
}
