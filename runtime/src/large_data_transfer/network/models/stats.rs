/// High-level snapshot of the transfer network state.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub active_transfers: usize,
    pub total_upload_mbps: f32,
    pub total_download_mbps: f32,
    pub available_chunks: usize,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self { connected_peers: 0, active_transfers: 0, total_upload_mbps: 0.0, total_download_mbps: 0.0, available_chunks: 0 }
    }
} 