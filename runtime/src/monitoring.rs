use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::time::Duration;
use chrono::{DateTime, Utc};

// --- Core Data Structures for Runtime Monitoring ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_in_kbps: f64,
    pub network_out_kbps: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainMetrics {
    pub block_height: u64,
    pub last_block_time: DateTime<Utc>,
    pub tx_pool_size: usize,
    pub average_tx_per_block: f64,
    pub time_to_produce_block_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMetrics {
    pub peer_count: usize,
    pub inbound_connections: usize,
    pub outbound_connections: usize,
    pub gossip_messages_sent: u64,
    pub gossip_messages_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighCpuUsage,
    HighMemoryUsage,
    LowDiskSpace,
    NodeUnresponsive,
    BlockchainHalted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Warning,
    Critical,
    Info,
}

// NOTE: Removed placeholder implementation structs:
// - MonitoringSystem
// - Alerter
// - MetricsCollector
// This file now only defines the data models for runtime monitoring.
