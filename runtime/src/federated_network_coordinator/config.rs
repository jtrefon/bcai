use crate::federated::FederatedConfig;

/// Configuration for federated network training
#[derive(Debug, Clone)]
pub struct FederatedNetworkConfig {
    pub federated_config: FederatedConfig,
    pub chunk_size_mb: u32,
    pub max_data_size_gb: u64,
    pub reward_per_participant: u64,
    pub coordinator_fee_percent: f32,
    pub min_reputation: i32,
}

impl Default for FederatedNetworkConfig {
    fn default() -> Self {
        Self {
            federated_config: FederatedConfig::default(),
            chunk_size_mb: 4, // 4MB chunks for LLM data
            max_data_size_gb: 5000, // Support up to 5TB
            reward_per_participant: 1000, // BCAI tokens
            coordinator_fee_percent: 5.0, // 5% coordinator fee
            min_reputation: 10, // Minimum reputation to participate
        }
    }
} 