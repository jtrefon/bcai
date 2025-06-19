use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::chains::ChainId;

/// Cross-chain message for oracle services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    pub message_id: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub sender: String,
    pub recipient: String,
    pub gas_limit: u64,
    pub created_at: DateTime<Utc>,
    pub status: MessageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    AIModelResult,
    TrainingJobUpdate,
    GovernanceProposal,
    PriceOracle,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    Pending,
    Relayed,
    Executed,
    Failed,
} 