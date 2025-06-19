use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BandwidthUsage {
    pub bytes_transferred: u64,
    pub start_time: Instant,
    pub current_mbps: f32,
}

/// Tracks aggregate bandwidth consumption across peers.
#[derive(Debug)]
pub struct BandwidthTracker {
    pub(crate) upload_usage: HashMap<String, BandwidthUsage>,
    pub(crate) download_usage: HashMap<String, BandwidthUsage>,
    pub(crate) total_upload_mbps: f32,
    pub(crate) total_download_mbps: f32,
    pub(crate) max_upload_mbps: u32,
    pub(crate) max_download_mbps: u32,
} 