use super::{
    session::TransferSession,
    state::ChunkStatus,
};

impl TransferSession {
    /// Calculate the current progress of the transfer as a percentage.
    pub fn progress(&self) -> f32 {
        let total_chunks = self.descriptor.as_ref().map_or(0, |d| d.chunk_hashes.len());
        if total_chunks == 0 {
            return 0.0;
        }
        let completed_chunks = self
            .chunk_status
            .values()
            .filter(|s| matches!(s, ChunkStatus::Complete(_)))
            .count();
        (completed_chunks as f32 / total_chunks as f32) * 100.0
    }
} 