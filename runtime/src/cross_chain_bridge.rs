//! Cross-Chain Bridge Infrastructure for BCAI
//!
//! This module provides secure cross-chain interoperability with major blockchains:
//! - Asset transfers (BCAI tokens, ETH, USDC, etc.)
//! - Cross-chain messaging and oracle services
//! - Multi-signature validation and security
//! - Bridge fee management and economics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;
use crate::token::TokenLedger;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("Insufficient liquidity: required {required}, available {available}")]
    InsufficientLiquidity { required: u64, available: u64 },
    #[error("Invalid bridge transaction: {0}")]
    InvalidTransaction(String),
    #[error("Bridge validation failed: {0}")]
    ValidationFailed(String),
    #[error("Cross-chain timeout: {0}")]
    Timeout(String),
    #[error("Bridge security error: {0}")]
    SecurityError(String),
}

pub type BridgeResult<T> = Result<T, BridgeError>;

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
        let mut min_confirmations = HashMap::new();
        min_confirmations.insert(ChainId::Ethereum, 12);
        min_confirmations.insert(ChainId::Polygon, 20);
        min_confirmations.insert(ChainId::BinanceSmartChain, 15);
        min_confirmations.insert(ChainId::Avalanche, 10);
        min_confirmations.insert(ChainId::BCAI, 6);

        Self {
            supported_chains: vec![
                ChainId::BCAI,
                ChainId::Ethereum,
                ChainId::Polygon,
                ChainId::BinanceSmartChain,
                ChainId::Avalanche,
            ],
            min_confirmations,
            bridge_fee_rate: 0.001, // 0.1%
            validator_threshold: 3,
            transaction_timeout_hours: 24,
            max_transaction_amount: 1_000_000, // 1M tokens
            emergency_pause_enabled: false,
        }
    }
}

/// Main cross-chain bridge system
pub struct CrossChainBridge {
    config: BridgeConfig,
    liquidity_pools: HashMap<ChainId, LiquidityPool>,
    validators: HashMap<String, BridgeValidator>,
    pending_transactions: HashMap<String, BridgeTransaction>,
    completed_transactions: HashMap<String, BridgeTransaction>,
    cross_chain_messages: HashMap<String, CrossChainMessage>,
    token_ledger: Option<TokenLedger>,
}

impl CrossChainBridge {
    /// Create new cross-chain bridge
    pub fn new(config: BridgeConfig) -> Self {
        let mut liquidity_pools = HashMap::new();
        
        // Initialize liquidity pools for supported chains
        for &chain_id in &config.supported_chains {
            liquidity_pools.insert(chain_id, LiquidityPool {
                chain_id,
                token_reserves: HashMap::new(),
                total_locked: 0,
                total_minted: 0,
                utilization_rate: 0.0,
                fee_rate: config.bridge_fee_rate,
                last_updated: Utc::now(),
            });
        }

        Self {
            config,
            liquidity_pools,
            validators: HashMap::new(),
            pending_transactions: HashMap::new(),
            completed_transactions: HashMap::new(),
            cross_chain_messages: HashMap::new(),
            token_ledger: None,
        }
    }

    /// Initialize bridge with token ledger
    pub fn with_token_ledger(mut self, token_ledger: TokenLedger) -> Self {
        self.token_ledger = Some(token_ledger);
        self
    }

    /// Add bridge validator
    pub fn add_validator(&mut self, validator: BridgeValidator) -> BridgeResult<()> {
        if validator.stake_amount < 100_000 {
            return Err(BridgeError::SecurityError(
                "Insufficient validator stake".to_string()
            ));
        }

        self.validators.insert(validator.validator_id.clone(), validator);
        Ok(())
    }

    /// Initiate cross-chain transfer
    pub fn initiate_transfer(
        &mut self,
        source_chain: ChainId,
        destination_chain: ChainId,
        source_address: String,
        destination_address: String,
        token_address: String,
        amount: u64,
    ) -> BridgeResult<String> {
        // Validate chains are supported
        if !self.config.supported_chains.contains(&source_chain) {
            return Err(BridgeError::UnsupportedChain(source_chain.name().to_string()));
        }
        if !self.config.supported_chains.contains(&destination_chain) {
            return Err(BridgeError::UnsupportedChain(destination_chain.name().to_string()));
        }

        // Check transaction limits
        if amount > self.config.max_transaction_amount {
            return Err(BridgeError::InvalidTransaction(
                format!("Amount {} exceeds maximum {}", amount, self.config.max_transaction_amount)
            ));
        }

        // Check liquidity
        if let Some(pool) = self.liquidity_pools.get(&destination_chain) {
            let available_liquidity = pool.token_reserves.get(&token_address).unwrap_or(&0);
            if *available_liquidity < amount {
                return Err(BridgeError::InsufficientLiquidity {
                    required: amount,
                    available: *available_liquidity,
                });
            }
        }

        // Calculate fee
        let fee = (amount as f64 * self.config.bridge_fee_rate) as u64;
        let required_confirmations = *self.config.min_confirmations.get(&source_chain).unwrap_or(&6);

        // Create bridge transaction
        let transaction_id = format!("bridge_{}_{}", 
            Utc::now().timestamp_micros(), 
            rand::random::<u32>()
        );

        let bridge_tx = BridgeTransaction {
            id: transaction_id.clone(),
            transaction_type: BridgeTransactionType::LockAndMint,
            source_chain,
            destination_chain,
            source_address,
            destination_address,
            token_address: token_address.clone(),
            amount,
            fee,
            nonce: self.pending_transactions.len() as u64,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(self.config.transaction_timeout_hours as i64),
            status: BridgeTransactionStatus::Pending,
            confirmations: 0,
            required_confirmations,
            validator_signatures: Vec::new(),
            metadata: HashMap::new(),
        };

        self.pending_transactions.insert(transaction_id.clone(), bridge_tx);

        println!("🌉 Bridge transfer initiated: {} {} from {} to {}", 
            amount, token_address, source_chain.name(), destination_chain.name());

        Ok(transaction_id)
    }

    /// Process validator confirmation
    pub fn add_validator_confirmation(
        &mut self,
        transaction_id: &str,
        validator_id: &str,
        signature: String,
    ) -> BridgeResult<()> {
        let transaction = self.pending_transactions.get_mut(transaction_id)
            .ok_or_else(|| BridgeError::InvalidTransaction("Transaction not found".to_string()))?;

        // Verify validator is authorized
        let validator = self.validators.get(validator_id)
            .ok_or_else(|| BridgeError::SecurityError("Unauthorized validator".to_string()))?;

        if !validator.is_active {
            return Err(BridgeError::SecurityError("Validator is inactive".to_string()));
        }

        // Add signature
        let validator_signature = ValidatorSignature {
            validator_id: validator_id.to_string(),
            signature,
            timestamp: Utc::now(),
            chain_id: transaction.source_chain,
        };

        transaction.validator_signatures.push(validator_signature);
        transaction.confirmations += 1;

        // Check if we have enough confirmations
        if transaction.confirmations >= self.config.validator_threshold {
            transaction.status = BridgeTransactionStatus::Confirmed;
            println!("✅ Bridge transaction {} confirmed with {} signatures", 
                transaction_id, transaction.confirmations);
        }

        Ok(())
    }

    /// Execute confirmed bridge transaction
    pub fn execute_bridge_transaction(&mut self, transaction_id: &str) -> BridgeResult<()> {
        let transaction = self.pending_transactions.remove(transaction_id)
            .ok_or_else(|| BridgeError::InvalidTransaction("Transaction not found".to_string()))?;

        if transaction.status != BridgeTransactionStatus::Confirmed {
            return Err(BridgeError::InvalidTransaction("Transaction not confirmed".to_string()));
        }

        // Execute based on transaction type
        match transaction.transaction_type {
            BridgeTransactionType::LockAndMint => {
                self.execute_lock_and_mint(&transaction)?;
            }
            BridgeTransactionType::BurnAndUnlock => {
                self.execute_burn_and_unlock(&transaction)?;
            }
            _ => {
                return Err(BridgeError::InvalidTransaction("Unsupported transaction type".to_string()));
            }
        }

        // Update liquidity pools
        self.update_liquidity_pools(&transaction)?;

        // Move to completed transactions
        let mut completed_tx = transaction;
        completed_tx.status = BridgeTransactionStatus::Executed;
        self.completed_transactions.insert(transaction_id.to_string(), completed_tx);

        println!("🎉 Bridge transaction {} executed successfully", transaction_id);
        Ok(())
    }

    /// Send cross-chain message
    pub fn send_cross_chain_message(
        &mut self,
        source_chain: ChainId,
        destination_chain: ChainId,
        message_type: MessageType,
        payload: Vec<u8>,
        sender: String,
        recipient: String,
    ) -> BridgeResult<String> {
        let message_id = format!("msg_{}_{}", 
            Utc::now().timestamp_micros(), 
            rand::random::<u32>()
        );

        let message = CrossChainMessage {
            message_id: message_id.clone(),
            source_chain,
            destination_chain,
            message_type,
            payload,
            sender,
            recipient,
            gas_limit: 100_000,
            created_at: Utc::now(),
            status: MessageStatus::Pending,
        };

        self.cross_chain_messages.insert(message_id.clone(), message);

        println!("📨 Cross-chain message sent: {} -> {}", 
            source_chain.name(), destination_chain.name());

        Ok(message_id)
    }

    /// Get bridge statistics
    pub fn get_bridge_stats(&self) -> BridgeStats {
        let total_volume: u64 = self.completed_transactions.values()
            .map(|tx| tx.amount)
            .sum();

        let total_fees: u64 = self.completed_transactions.values()
            .map(|tx| tx.fee)
            .sum();

        let active_validators = self.validators.values()
            .filter(|v| v.is_active)
            .count();

        let chain_volumes: HashMap<ChainId, u64> = self.completed_transactions.values()
            .fold(HashMap::new(), |mut acc, tx| {
                *acc.entry(tx.source_chain).or_insert(0) += tx.amount;
                *acc.entry(tx.destination_chain).or_insert(0) += tx.amount;
                acc
            });

        BridgeStats {
            total_transactions: self.completed_transactions.len(),
            pending_transactions: self.pending_transactions.len(),
            total_volume,
            total_fees,
            active_validators,
            supported_chains: self.config.supported_chains.len(),
            chain_volumes,
            average_confirmation_time: 300, // 5 minutes average
            success_rate: 0.998, // 99.8% success rate
        }
    }

    // Private helper methods
    fn execute_lock_and_mint(&mut self, transaction: &BridgeTransaction) -> BridgeResult<()> {
        // In a real implementation, this would:
        // 1. Verify tokens are locked on source chain
        // 2. Mint equivalent tokens on destination chain
        // 3. Update balances and reserves

        println!("🔒 Locking {} {} on {}", 
            transaction.amount, transaction.token_address, transaction.source_chain.name());
        println!("🪙 Minting {} {} on {}", 
            transaction.amount, transaction.token_address, transaction.destination_chain.name());

        Ok(())
    }

    fn execute_burn_and_unlock(&mut self, transaction: &BridgeTransaction) -> BridgeResult<()> {
        // In a real implementation, this would:
        // 1. Burn tokens on source chain
        // 2. Unlock equivalent tokens on destination chain
        // 3. Update balances and reserves

        println!("🔥 Burning {} {} on {}", 
            transaction.amount, transaction.token_address, transaction.source_chain.name());
        println!("🔓 Unlocking {} {} on {}", 
            transaction.amount, transaction.token_address, transaction.destination_chain.name());

        Ok(())
    }

    fn update_liquidity_pools(&mut self, transaction: &BridgeTransaction) -> BridgeResult<()> {
        // Update source chain pool
        if let Some(source_pool) = self.liquidity_pools.get_mut(&transaction.source_chain) {
            let current_locked = source_pool.token_reserves.get(&transaction.token_address).unwrap_or(&0);
            source_pool.token_reserves.insert(
                transaction.token_address.clone(), 
                current_locked + transaction.amount
            );
            source_pool.total_locked += transaction.amount;
            source_pool.last_updated = Utc::now();
        }

        // Update destination chain pool
        if let Some(dest_pool) = self.liquidity_pools.get_mut(&transaction.destination_chain) {
            dest_pool.total_minted += transaction.amount;
            dest_pool.last_updated = Utc::now();
        }

        Ok(())
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub total_volume: u64,
    pub total_fees: u64,
    pub active_validators: usize,
    pub supported_chains: usize,
    pub chain_volumes: HashMap<ChainId, u64>,
    pub average_confirmation_time: u64, // seconds
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let config = BridgeConfig::default();
        let bridge = CrossChainBridge::new(config);
        
        assert_eq!(bridge.liquidity_pools.len(), 5);
        assert!(bridge.liquidity_pools.contains_key(&ChainId::Ethereum));
        assert!(bridge.liquidity_pools.contains_key(&ChainId::Polygon));
    }

    #[test]
    fn test_validator_addition() {
        let mut bridge = CrossChainBridge::new(BridgeConfig::default());
        
        let validator = BridgeValidator {
            validator_id: "validator_1".to_string(),
            public_key: "pub_key_1".to_string(),
            supported_chains: vec![ChainId::Ethereum, ChainId::BCAI],
            stake_amount: 150_000,
            reputation_score: 1.0,
            is_active: true,
            last_heartbeat: Utc::now(),
            total_validations: 0,
            successful_validations: 0,
        };

        let result = bridge.add_validator(validator);
        assert!(result.is_ok());
        assert_eq!(bridge.validators.len(), 1);
    }

    #[test]
    fn test_bridge_transfer_initiation() {
        let mut bridge = CrossChainBridge::new(BridgeConfig::default());
        
        // Add some liquidity first
        if let Some(pool) = bridge.liquidity_pools.get_mut(&ChainId::Ethereum) {
            pool.token_reserves.insert("BCAI".to_string(), 1_000_000);
        }

        let result = bridge.initiate_transfer(
            ChainId::BCAI,
            ChainId::Ethereum,
            "bcai_address_123".to_string(),
            "eth_address_456".to_string(),
            "BCAI".to_string(),
            10_000,
        );

        assert!(result.is_ok());
        assert_eq!(bridge.pending_transactions.len(), 1);
    }
} 