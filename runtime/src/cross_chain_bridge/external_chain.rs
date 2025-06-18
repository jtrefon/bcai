use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChainId {
    BCAI = 1,
    Ethereum = 2,
    Polygon = 3,
    BinanceSmartChain = 4,
    Avalanche = 5,
    Solana = 6,
    Arbitrum = 7,
    Optimism = 8,
}

impl ChainId {
    pub fn name(&self) -> &'static str {
        match self {
            ChainId::BCAI => "BCAI",
            ChainId::Ethereum => "Ethereum",
            ChainId::Polygon => "Polygon",
            ChainId::BinanceSmartChain => "Binance Smart Chain",
            ChainId::Avalanche => "Avalanche",
            ChainId::Solana => "Solana",
            ChainId::Arbitrum => "Arbitrum",
            ChainId::Optimism => "Optimism",
        }
    }

    pub fn native_token(&self) -> &'static str {
        match self {
            ChainId::BCAI => "BCAI",
            ChainId::Ethereum => "ETH",
            ChainId::Polygon => "MATIC",
            ChainId::BinanceSmartChain => "BNB",
            ChainId::Avalanche => "AVAX",
            ChainId::Solana => "SOL",
            ChainId::Arbitrum => "ETH",
            ChainId::Optimism => "ETH",
        }
    }
}

/// Bridge liquidity pool for each chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub chain_id: ChainId,
    pub token_reserves: HashMap<String, u64>,
    pub total_locked: u64,
    pub total_minted: u64,
    pub utilization_rate: f64,
    pub fee_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// Bridge validator node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeValidator {
    pub validator_id: String,
    pub public_key: String,
    pub supported_chains: Vec<ChainId>,
    pub stake_amount: u64,
    pub reputation_score: f64,
    pub is_active: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub total_validations: u64,
    pub successful_validations: u64,
}

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

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub supported_chains: Vec<ChainId>,
    pub min_confirmations: HashMap<ChainId, u32>,
    pub bridge_fee_rate: f64,
    pub validator_threshold: u32,
    pub transaction_timeout_hours: u64,
    pub max_transaction_amount: u64,
    pub emergency_pause_enabled: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            supported_chains: vec![
                ChainId::Ethereum,
                ChainId::Polygon,
                ChainId::BinanceSmartChain,
            ],
            min_confirmations: HashMap::from([
                (ChainId::Ethereum, 12),
                (ChainId::Polygon, 64),
                (ChainId::BinanceSmartChain, 15),
            ]),
            bridge_fee_rate: 0.001, // 0.1%
            validator_threshold: 3,
            transaction_timeout_hours: 24,
            max_transaction_amount: 1_000_000, // $1M
            emergency_pause_enabled: false,
        }
    }
} 