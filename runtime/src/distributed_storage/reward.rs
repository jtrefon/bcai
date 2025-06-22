use serde::{Deserialize, Serialize};

/// Reward policy parameters – static, network-wide.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RewardPolicy {
    /// Base reward in BCAI tokens per GiB per hour (original copy).
    pub base_rate_per_gib_hour: u128,
    /// Multiplier for each redundant copy beyond the original (0–1).
    pub redundancy_multiplier: f32,
}

impl Default for RewardPolicy {
    fn default() -> Self { Self { base_rate_per_gib_hour: 1, redundancy_multiplier: 0.4 } }
}

/// Calculate reward for storing `bytes` for `hours` with `copies` redundant replicas.
/// NOTE: `copies` excludes the original data owner's copy.
pub fn calculate_reward(bytes: u64, hours: u64, copies: u8, policy: RewardPolicy) -> u128 {
    let gib = 1_073_741_824u128;
    let base_units = (bytes as u128 + gib - 1) / gib; // round-up to GiB
    let original = base_units * (hours as u128) * policy.base_rate_per_gib_hour;
    let extra = if copies == 0 {
        0
    } else {
        let replica_units = base_units * (hours as u128) * (copies as u128);
        (replica_units as f32 * policy.redundancy_multiplier) as u128
    };
    original + extra
} 