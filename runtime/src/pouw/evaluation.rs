use super::types::SignedEvaluation;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
#[cfg(feature = "p2p")]
use crate::p2p_service::{P2PHandle, P2PError};
#[cfg(feature = "p2p")]
use crate::network::NetworkMessage;
use sha2::{Sha256, Digest};

/// Creates a signed evaluation using the validator's signing key.
pub fn sign_evaluation(task_id: &str, accuracy: u32, key: &SigningKey) -> SignedEvaluation {
    let mut msg = task_id.as_bytes().to_vec();
    msg.extend_from_slice(&accuracy.to_be_bytes());
    let signature: Signature = key.sign(&msg);
    SignedEvaluation {
        task_id: task_id.to_string(),
        accuracy,
        validator: hex::encode(key.verifying_key().to_bytes()),
        signature: signature.to_bytes().to_vec(),
    }
}

/// Verifies a signed evaluation result.
pub fn verify_evaluation(eval: &SignedEvaluation) -> bool {
    let Ok(pk_bytes) = hex::decode(&eval.validator) else { return false };
    let Ok(vk) = VerifyingKey::from_bytes(&pk_bytes.try_into().unwrap_or([0u8;32])) else { return false };
    let mut msg = eval.task_id.as_bytes().to_vec();
    msg.extend_from_slice(&eval.accuracy.to_be_bytes());
    let Ok(sig) = Signature::from_slice(&eval.signature) else { return false };
    vk.verify(&msg, &sig).is_ok()
}

/// Compute a SHA-256 hash over the serialized evaluation.
pub fn evaluation_hash(eval: &SignedEvaluation) -> String {
    let bytes = bincode::serialize(eval).expect("serialize evaluation");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Broadcast a signed evaluation over gossipsub.
#[cfg(feature = "p2p")]
pub async fn broadcast_evaluation(handle: &P2PHandle, eval: &SignedEvaluation) -> Result<(), P2PError> {
    let msg = NetworkMessage::PoUWEvaluation { evaluation: eval.clone() };
    let bytes = bincode::serialize(&msg).map_err(|e| P2PError::SerializationFailed(e.to_string()))?;
    handle.send_message("pouw_evaluations".into(), bytes).await
}
