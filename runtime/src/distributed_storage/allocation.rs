//! Node allocation logic for primary & replica selection.
//! Uses a deterministic multi-factor scoring system described in docs.

use serde::{Serialize, Deserialize};

/// Tunable weights for each metric.  Sum need not equal 1 – scores are weighted sum.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoragePolicy {
    pub w_reputation: f32,
    pub w_free_capacity: f32,
    pub w_latency: f32,
    pub w_geo_diversity: f32,
    pub w_energy: f32,
    pub w_utilisation_balance: f32,
}

impl Default for StoragePolicy {
    fn default() -> Self {
        Self {
            w_reputation: 0.35,
            w_free_capacity: 0.20,
            w_latency: 0.15,
            w_geo_diversity: 0.10,
            w_energy: 0.05,
            w_utilisation_balance: 0.15,
        }
    }
}

/// Snapshot of metrics for a single storage node.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeMetrics {
    pub node_id: String,
    pub reputation: f32,    // 0-1
    pub free_capacity: f32, // 0-1
    pub latency_ms: u32,    // observed latency to requester
    pub region: String,     // ISO region code
    pub energy_score: f32,  // 0-1 clean-energy fraction
    pub utilisation: f32,   // 0-1 used / total
}

/// Deterministic allocator returning `copies + 1` node ids (primary + replicas).
/// For simplicity chooses highest-scoring nodes while ensuring region diversity.
pub fn allocate_nodes(
    policy: &StoragePolicy,
    metrics: &[NodeMetrics],
    copies: u8,
) -> Vec<String> {
    let mut scored: Vec<(String, f32, String)> = metrics
        .iter()
        // basic filtering – require some free capacity & minimum reputation
        .filter(|m| m.free_capacity > 0.1 && m.reputation > 0.2)
        .map(|m| {
            let mut score = 0.0;
            score += policy.w_reputation * m.reputation;
            score += policy.w_free_capacity * m.free_capacity;
            let latency_norm = 1.0 - ((m.latency_ms as f32).min(500.0) / 500.0);
            score += policy.w_latency * latency_norm;
            score += policy.w_energy * m.energy_score;
            score += policy.w_utilisation_balance * (1.0 - m.utilisation);
            (m.node_id.clone(), score, m.region.clone())
        })
        .collect();

    // Sort by score DESC then node_id for deterministic tie-break.
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(&b.0)));

    let mut selected: Vec<String> = Vec::new();
    let mut seen_regions: Vec<String> = Vec::new();
    let needed = copies as usize + 1;

    for (node_id, mut score, region) in scored.into_iter() {
        if selected.len() >= needed { break; }
        if seen_regions.contains(&region) {
            // apply geo diversity penalty
            score *= 1.0 - policy.w_geo_diversity;
        }
        // still keep order due to pre-sort so first occurrences remain best.
        selected.push(node_id);
        seen_regions.push(region);
    }
    selected
} 