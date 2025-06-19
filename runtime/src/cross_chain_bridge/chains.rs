use serde::{Deserialize, Serialize};

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