//! Defines the command API for interacting with the P2P service.

use super::{codec::WireMessage, error::P2PError};
use libp2p::{gossipsub, PeerId};
use tokio::sync::{mpsc, oneshot};

/// Commands sent from other parts of the application to the `P2PService`.
#[derive(Debug)]
pub enum Command {
    /// Broadcast a message to all peers on a given topic.
    SendMessage {
        topic: gossipsub::IdentTopic,
        message: Vec<u8>,
        response: oneshot::Sender<Result<(), P2PError>>,
    },
    /// Get a list of all connected peers.
    GetPeers {
        response: oneshot::Sender<Vec<PeerId>>,
    },
    /// Trigger the Kademlia bootstrap process.
    Bootstrap {
        response: oneshot::Sender<Result<(), P2PError>>,
    },
    /// Send a direct request to a specific peer and await a response.
    Request {
        peer_id: PeerId,
        message: WireMessage,
        response: oneshot::Sender<Result<WireMessage, P2PError>>,
    },
}

/// A handle to the `P2PService` that can be safely cloned and shared.
/// It provides a public API for interacting with the networking layer.
#[derive(Clone)]
pub struct P2PHandle {
    pub command_sender: mpsc::Sender<Command>,
}

impl P2PHandle {
    pub fn new(command_sender: mpsc::Sender<Command>) -> Self {
        Self { command_sender }
    }

    /// Broadcast a message to a gossipsub topic.
    pub async fn send_message(&self, topic: String, message: Vec<u8>) -> Result<(), P2PError> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::SendMessage {
                topic: gossipsub::IdentTopic::new(topic),
                message,
                response: response_sender,
            })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        response_receiver.await.map_err(|e| P2PError::ChannelError(e.to_string()))?
    }

    /// Get a list of connected peer IDs.
    pub async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::GetPeers { response: response_sender })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        response_receiver.await.map_err(|e| P2PError::ChannelError(e.to_string()))?
    }

    /// Start the Kademlia bootstrap process.
    pub async fn bootstrap(&self) -> Result<(), P2PError> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::Bootstrap { response: response_sender })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        response_receiver.await.map_err(|e| P2PError::ChannelError(e.to_string()))?
    }

    /// Send a direct request to a peer.
    pub async fn request(&self, peer_id: PeerId, message: WireMessage) -> Result<WireMessage, P2PError> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::Request {
                peer_id,
                message,
                response: response_sender,
            })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        response_receiver.await.map_err(|e| P2PError::ChannelError(e.to_string()))?
    }
} 