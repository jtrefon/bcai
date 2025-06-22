use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Storage node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode {
    pub node_id: String,
    pub address: String,
    pub capacity: u64,
    pub used_space: u64,
    pub last_seen: u64,
    pub reliability_score: f32,
} 
/// Naive replication planner ensuring desired replica count.
#[derive(Debug, Clone)]
pub struct ReplicationManager {
    pub nodes: Vec<StorageNode>,
}

impl ReplicationManager {
    /// Select healthy target not already in replica set.
    fn select_target<'a>(&'a self, existing: &[String]) -> Option<&'a StorageNode> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.nodes
            .iter()
            .filter(|n| now.saturating_sub(n.last_seen) < 300 && !existing.contains(&n.node_id))
            .max_by(|a, b| a.reliability_score.partial_cmp(&b.reliability_score).unwrap())
    }

    /// Produce plan of (node_id,key) pairs to reach `required_copies` (excluding original).
    pub fn plan_replication(&self, key: &str, replicas: &[String], required_copies: u32) -> Vec<(String,String)> {
        let mut plan = Vec::new();
        let mut current = replicas.to_vec();
        while current.len() < (required_copies as usize).saturating_add(1) {
            if let Some(target) = self.select_target(&current) {
                plan.push((target.node_id.clone(), key.to_string()));
                current.push(target.node_id.clone());
            } else { break; }
        }
        plan
    }
}

