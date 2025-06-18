// This module will handle the logic for creating and processing cross-chain transactions. 

use super::external_chain::ChainId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bridge transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeTransactionType {
    /// Lock tokens on source chain, mint on destination
    LockAndMint,
    /// Burn tokens on source chain, unlock on destination
    BurnAndUnlock,
    /// Cross-chain message passing
    MessageRelay,
    /// Oracle data feed
    OracleUpdate,
}

/// Cross-chain bridge transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: String,
    pub transaction_type: BridgeTransactionType,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub source_address: String,
    pub destination_address: String,
    pub token_address: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: BridgeTransactionStatus,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub validator_signatures: Vec<ValidatorSignature>,
    pub metadata: HashMap<String, String>,
}

/// Bridge transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeTransactionStatus {
    Pending,
    Confirmed,
    Executed,
    Failed,
    Expired,
    Cancelled,
}

/// Validator signature for bridge security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator_id: String,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub chain_id: ChainId,
} 