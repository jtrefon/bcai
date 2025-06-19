use super::{
    session::TransferSession,
    state::ChunkStatus,
};
use std::time::Instant;

impl TransferSession {
    /// Update the status of a specific chunk.
    pub fn set_chunk_status(&mut self, chunk_index: u32, status: ChunkStatus) {
        self.chunk_status.insert(chunk_index, status);
        self.last_activity = Instant::now();
    }

    /// Get a list of chunk indices that still need to be downloaded.
    pub fn pending_chunks(&self) -> Vec<u32> {
        if let Some(descriptor) = &self.descriptor {
            (0..descriptor.chunk_hashes.len() as u32)
                .filter(|i| !matches!(self.chunk_status.get(i), Some(ChunkStatus::Complete(_))))
                .collect()
        } else {
            vec![]
        }
    }
} 