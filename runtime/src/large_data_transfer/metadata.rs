use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::TransferPriority;

/// Metadata associated with a large data transfer – single purpose: hold descriptive
/// information; it **does not** perform networking or disk I/O.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetadata {
    pub name: String,
    pub content_type: String,
    pub filename: Option<String>,
    pub created_at: u64,
    pub modified_at: u64,
    pub priority: TransferPriority,
    pub tags: HashMap<String, String>,
    pub source_node: Option<String>,
    pub target_nodes: Vec<String>,
    pub timeout_seconds: Option<u64>,
}

impl Default for TransferMetadata {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            name: "unnamed".to_string(),
            content_type: "application/octet-stream".to_string(),
            filename: None,
            created_at: now,
            modified_at: now,
            priority: TransferPriority::Normal,
            tags: HashMap::new(),
            source_node: None,
            target_nodes: Vec::new(),
            timeout_seconds: Some(3600),
        }
    }
}

impl TransferMetadata {
    pub fn new(name: String) -> Self { Self { name, ..Default::default() } }

    pub fn with_content_type(mut self, ctype: String) -> Self { self.content_type = ctype; self }

    pub fn with_priority(mut self, priority: TransferPriority) -> Self { self.priority = priority; self }

    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout_seconds {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now.saturating_sub(self.created_at) > timeout
        } else { false }
    }
} 