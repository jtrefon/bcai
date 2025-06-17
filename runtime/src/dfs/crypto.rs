//! Cryptographic operations for the Decentralized Filesystem (DFS).
//! Handles encryption, decryption, and key generation.

use crate::dfs::types::{DfsError, EncryptionMetadata, FilePermissions};
use aes_gcm::{
    aead::{Aead, AeadCore, OsRng},
    Aes256Gcm, Key, KeyInit, Nonce,
};
use base64::{engine::general_purpose, Engine as _};

/// Encrypts file data based on the specified permission model.
pub async fn encrypt_file_data(
    data: &[u8],
    permissions: &FilePermissions,
    owner: &str,
) -> Result<(Vec<u8>, EncryptionMetadata), DfsError> {
    let key = generate_symmetric_key()?;
    let (encrypted_data, nonce_vec) = encrypt_with_symmetric_key(data, &key)?;

    let encrypted_key_for_owner = key.clone(); // In a real system, encrypt this with owner's public key

    let final_permissions = match permissions {
        FilePermissions::Public => {
            return Ok((
                data.to_vec(),
                EncryptionMetadata {
                    is_encrypted: false,
                    encryption_algorithm: "none".to_string(),
                    nonce: None,
                    permissions: FilePermissions::Public,
                },
            ));
        }
        _ => FilePermissions::OwnerOnly {
            owner: owner.to_string(),
            encrypted_key: encrypted_key_for_owner,
        },
        // TODO: Implement other permission types like Group, Custom, TimeBound
    };

    let metadata = EncryptionMetadata {
        is_encrypted: true,
        encryption_algorithm: "AES-256-GCM".to_string(),
        nonce: Some(general_purpose::STANDARD.encode(nonce_vec)),
        permissions: final_permissions,
    };

    Ok((encrypted_data, metadata))
}

/// Decrypts file data using a provided symmetric key.
pub async fn decrypt_file_data_with_key(
    encrypted_data: &[u8],
    metadata: &EncryptionMetadata,
    key_b64: &str,
) -> Result<Vec<u8>, DfsError> {
    if !metadata.is_encrypted {
        return Ok(encrypted_data.to_vec());
    }

    let nonce_b64 = metadata
        .nonce
        .as_ref()
        .ok_or(DfsError::EncryptionError("Missing nonce for decryption".to_string()))?;
    let nonce = general_purpose::STANDARD.decode(nonce_b64)
        .map_err(|e| DfsError::EncryptionError(format!("Invalid nonce encoding: {}", e)))?;

    decrypt_with_symmetric_key(encrypted_data, key_b64, &nonce)
}

/// Generates a new random AES-256 symmetric key.
fn generate_symmetric_key() -> Result<String, DfsError> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    Ok(general_purpose::STANDARD.encode(key))
}

/// Encrypts data with a given Base64-encoded symmetric key.
fn encrypt_with_symmetric_key(
    data: &[u8],
    key_b64: &str,
) -> Result<(Vec<u8>, Vec<u8>), DfsError> {
    let key_bytes = general_purpose::STANDARD.decode(key_b64)
        .map_err(|e| DfsError::KeyError(format!("Invalid key encoding: {}", e)))?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let encrypted_data = cipher.encrypt(&nonce, data)
        .map_err(|e| DfsError::EncryptionError(format!("AES encryption failed: {}", e)))?;

    Ok((encrypted_data, nonce.to_vec()))
}

/// Decrypts data with a given Base64-encoded symmetric key and nonce.
fn decrypt_with_symmetric_key(
    data: &[u8],
    key_b64: &str,
    nonce: &[u8],
) -> Result<Vec<u8>, DfsError> {
    let key_bytes = general_purpose::STANDARD.decode(key_b64)
        .map_err(|e| DfsError::KeyError(format!("Invalid key encoding: {}", e)))?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce_slice = Nonce::from_slice(nonce);

    cipher.decrypt(nonce_slice, data)
        .map_err(|e| DfsError::EncryptionError(format!("AES decryption failed: {}", e)))
} 