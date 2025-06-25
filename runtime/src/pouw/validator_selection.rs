use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use super::types::ValidatorSelectionConfig;

/// Selects a subset of validators based on stake weights and a VRF seed.
///
/// * `validators` - Vector of tuples `(validator_id, stake)`.
/// * `count` - Number of validators to select.
/// * `seed` - 32-byte seed derived from a VRF output.
pub fn select_validators(
    validators: Vec<(String, u64)>,
    seed: [u8; 32],
    config: &ValidatorSelectionConfig,
) -> Vec<String> {
    if validators.is_empty() || config.subset_size == 0 {
        return vec![];
    }
    let eligible: Vec<(String, u64)> = validators
        .into_iter()
        .filter(|(_, stake)| *stake >= config.min_stake)
        .collect();
    if eligible.is_empty() {
        return vec![];
    }
    let total_stake: u64 = eligible.iter().map(|v| v.1).sum();
    let mut rng = StdRng::from_seed(seed);
    let mut selected = Vec::new();

    for _ in 0..config.subset_size.min(eligible.len()) {
        let mut threshold = rng.gen_range(0..total_stake);
        for (id, stake) in &eligible {
            if threshold < *stake {
                if !selected.contains(id) {
                    selected.push(id.clone());
                }
                break;
            }
            threshold -= *stake;
        }
    }
    selected
}
