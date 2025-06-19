use crate::large_data_transfer::{chunk::ChunkId, protocol::message::TransferMessage};
use super::peer::PeerCapabilities;

/// Control-plane messages exchanged over the P2P layer to coordinate chunk transfers.
#[derive(Debug, Clone)]
pub enum NetworkTransferMessage {
    /// Peer announces which chunks it can provide.
    ChunkAnnouncement { peer_id: String, available_chunks: Vec<ChunkId> },
    /// Request a specific chunk from a peer.
    ChunkRequest { chunk_id: ChunkId, requester_id: String },
    /// Deliver chunk data or error.
    ChunkResponse { chunk_id: ChunkId, data: Option<Vec<u8>>, error: Option<String> },
    /// Forward lower-level transfer protocol messages.
    TransferControl(TransferMessage),
    /// Simple bandwidth negotiation handshake.
    BandwidthNegotiation { requested_mbps: u32, granted_mbps: u32 },
    /// Update peer capabilities (e.g., after storage change).
    CapabilityUpdate { peer_id: String, capabilities: PeerCapabilities },
} 