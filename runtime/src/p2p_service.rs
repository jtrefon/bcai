//! P2P Service bridging unified nodes with libp2p networking
//!
//! This module provides the real networking layer that connects our unified node
//! architecture with actual P2P communication using libp2p.

use crate::network::{NetworkCoordinator, NetworkMessage, NetworkStats};
use crate::node::{NodeCapability, UnifiedNode};
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::{
    gossipsub, identity, kad,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

/// Combined network behaviour for BCAI nodes
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "BCAIBehaviourEvent")]
pub struct BCAINetworkBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub request_response: request_response::Behaviour<WireCodec>,
}

/// Events emitted by the BCAINetworkBehaviour
#[derive(Debug)]
pub enum BCAIBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Kademlia(kad::Event),
    RequestResponse(request_response::Event<WireMessage, WireMessage>),
}

impl From<gossipsub::Event> for BCAIBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        BCAIBehaviourEvent::Gossipsub(event)
    }
}

impl From<kad::Event> for BCAIBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        BCAIBehaviourEvent::Kademlia(event)
    }
}

impl From<request_response::Event<WireMessage, WireMessage>> for BCAIBehaviourEvent {
    fn from(event: request_response::Event<WireMessage, WireMessage>) -> Self {
        BCAIBehaviourEvent::RequestResponse(event)
    }
}

/// Commands sent to the P2P Service
#[derive(Debug)]
pub enum Command {
    SendMessage {
        topic: gossipsub::IdentTopic,
        message: Vec<u8>,
        response: oneshot::Sender<Result<(), P2PError>>,
    },
    GetPeers {
        response: oneshot::Sender<Vec<PeerId>>,
    },
    Bootstrap {
        response: oneshot::Sender<Result<(), P2PError>>,
    },
    Request {
        peer_id: PeerId,
        message: WireMessage,
        response: oneshot::Sender<Result<WireMessage, P2PError>>,
    },
}

/// P2P service errors
#[derive(Debug, Error)]
pub enum P2PError {
    #[error("Network connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Message serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Service not started")]
    ServiceNotStarted,
    #[error("Channel communication error: {0}")]
    ChannelError(String),
    #[error("Libp2p transport error: {0}")]
    TransportError(String),
    #[error("Input/Output error: {0}")]
    IoError(String),
    #[error("Network error: {0}")]
    Network(String),
}

impl From<std::io::Error> for P2PError {
    fn from(e: std::io::Error) -> Self {
        P2PError::IoError(e.to_string())
    }
}

/// The wire protocol for request-response messages.
#[derive(Debug, Clone)]
pub struct WireCodec;

#[derive(Clone)]
pub struct WireProtocol();

impl std::fmt::Debug for WireProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WireProtocol").finish()
    }
}

impl AsRef<[u8]> for WireProtocol {
    fn as_ref(&self) -> &[u8] {
        b"/bcai/wire/1.0.0"
    }
}

impl libp2p::request_response::Codec for WireCodec {
    type Protocol = WireProtocol;
    type Request = WireMessage;
    type Response = WireMessage;

    async fn read_request<T>(&mut self, _: &WireProtocol, io: &mut T) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.read_to_end(&mut vec).await?;
        Ok(serde_json::from_slice(&vec).unwrap())
    }

    async fn read_response<T>(&mut self, _: &WireProtocol, io: &mut T) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.read_to_end(&mut vec).await?;
        Ok(serde_json::from_slice(&vec).unwrap())
    }

    async fn write_request<T>(&mut self, _: &WireProtocol, io: &mut T, req: Self::Request) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&req).unwrap();
        io.write_all(&buf).await
    }

    async fn write_response<T>(&mut self, _: &WireProtocol, io: &mut T, res: Self::Response) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&res).unwrap();
        io.write_all(&buf).await
    }
}

/// P2P message wrapper for network transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMessage {
    Block(crate::blockchain::block::Block),
    Transaction(crate::blockchain::transaction::Transaction),
    Ping,
    Pong,
}

/// P2P peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub capabilities: Option<NodeCapability>,
    pub last_seen: Instant,
    pub reputation: i32,
    pub connection_count: usize,
}

/// P2P service configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
    pub heartbeat_interval: Duration,
    pub message_timeout: Duration,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_port: 4001,
            bootstrap_peers: vec![],
            max_peers: 50,
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(10),
        }
    }
}

/// P2P service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PStats {
    pub peer_count: usize,
    pub connected_peers: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub uptime: Duration,
    pub network_stats: NetworkStats,
}

/// Real P2P service for distributed BCAI networking
pub struct P2PService {
    pub swarm: Swarm<BCAINetworkBehaviour>,
    pub command_receiver: mpsc::Receiver<Command>,
    peers: HashMap<String, PeerInfo>,
    stats: P2PStats,
    start_time: Option<Instant>,
    config: P2PConfig,
    request_map: HashMap<request_response::RequestId, oneshot::Sender<Result<WireMessage, P2PError>>>,
}

const GLOBAL_TOPIC: &str = "bcai_global";

impl P2PService {
    /// Create a new P2P service
    pub async fn new(
        config: P2PConfig,
    ) -> Result<(Self, P2PHandle), P2PError> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("🤖 Local Peer ID: {}", local_peer_id);

        let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(
                libp2p::noise::Config::new(&local_key)
                    .map_err(|e| P2PError::TransportError(e.to_string()))?,
            )
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        let store = kad::store::MemoryStore::new(local_peer_id);
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| P2PError::SerializationFailed(e.to_string()))?;

        let gossipsub_behaviour = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| P2PError::SerializationFailed(e.to_string()))?;

        let request_response_behaviour = request_response::Behaviour::new(
            [(WireProtocol(), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        let behaviour = BCAINetworkBehaviour {
            gossipsub: gossipsub_behaviour,
            kademlia: kad::Behaviour::new(local_peer_id, store),
            request_response: request_response_behaviour,
        };

        let mut swarm = libp2p::Swarm::new(transport, behaviour, local_peer_id, libp2p::swarm::Config::with_tokio_executor());

        swarm.listen_on(
            format!("/ip4/0.0.0.0/tcp/{}", config.listen_port)
                .parse()
                .unwrap(),
        )?;

        // Subscribe to the global topic
        let global_topic = gossipsub::IdentTopic::new(GLOBAL_TOPIC);
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&global_topic)
            .map_err(|e| P2PError::SerializationFailed(e.to_string()))?;

        // Add bootstrap nodes to Kademlia
        for peer_addr_str in &config.bootstrap_peers {
            if let Ok(addr) = peer_addr_str.parse::<Multiaddr>() {
                if let Some(peer_id) = addr.iter().find_map(|proto| {
                    if let libp2p::core::multiaddr::Protocol::P2p(peer_id) = proto {
                        Some(peer_id)
                    } else {
                        None
                    }
                }) {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                    println!("Added bootstrap node: {} with address {}", peer_id, addr);
                }
            }
        }

        let (command_sender, command_receiver) = mpsc::channel(100);

        let service = Self {
            swarm,
            command_receiver,
            peers: HashMap::new(),
            stats: P2PStats::default(),
            start_time: None,
            config,
            request_map: HashMap::new(),
        };

        let handle = P2PHandle::new(command_sender);

        Ok((service, handle))
    }

    pub async fn run(mut self) {
        self.start_time = Some(Instant::now());
        let mut bootstrap_timer = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = bootstrap_timer.tick() => {
                    if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
                        eprintln!("Failed to start Kademlia bootstrap: {:?}", e);
                    }
                },
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                },
                Some(command) = self.command_receiver.recv() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<BCAIBehaviourEvent, kad::ToSwarm<kad::QueryId, <kad::store::MemoryStore as kad::store::RecordStore>::OutRecord>>) {
        match event {
            SwarmEvent::Behaviour(BCAIBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: _peer_id,
                message_id: _id,
                message,
            })) => {
                if let Ok(wire_message) = serde_json::from_slice::<WireMessage>(&message.data) {
                    println!("Received gossipsub message: {:?}", wire_message);
                    // TODO: Pass this up to the application layer
                }
            }
            SwarmEvent::Behaviour(BCAIBehaviourEvent::Kademlia(event)) => {
                println!("Kademlia event: {:?}", event);
            }
            SwarmEvent::Behaviour(BCAIBehaviourEvent::RequestResponse(request_response::Event::Message {
                peer,
                message: request_response::Message::Request { request, channel, .. },
            })) => {
                // For now, just pong back
                let response = WireMessage::Pong;
                self.swarm.behaviour_mut().request_response.send_response(channel, response).unwrap();

            }
            SwarmEvent::Behaviour(BCAIBehaviourEvent::RequestResponse(request_response::Event::Message {
                peer,
                message: request_response::Message::Response { request_id, response },
            })) => {
                if let Some(sender) = self.request_map.remove(&request_id) {
                    let _ = sender.send(Ok(response));
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Disconnected from {}", peer_id);
            }
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::SendMessage {
                topic,
                message,
                response,
            } => {
                if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, message) {
                    let _ = response.send(Err(P2PError::SerializationFailed(e.to_string())));
                } else {
                    let _ = response.send(Ok(()));
                }
            }
            Command::GetPeers { response } => {
                let peers = self.swarm.behaviour().kademlia.kbuckets().map(|kbucket| kbucket.iter().map(|entry| *entry.node.key.preimage()).collect()).concat();
                let _ = response.send(peers);
            }
            Command::Bootstrap { response } => {
                match self.swarm.behaviour_mut().kademlia.bootstrap() {
                    Ok(_) => { let _ = response.send(Ok(())); }
                    Err(e) => { let _ = response.send(Err(P2PError::Network(format!("Bootstrap failed: {:?}", e)))); }
                }
            }
            Command::Request { peer_id, message, response } => {
                let request_id = self.swarm.behaviour_mut().request_response.send_request(&peer_id, message);
                self.request_map.insert(request_id, response);
            }
        }
    }

    /// Start the P2P service
    pub fn start(&mut self) -> Result<(), P2PError> {
        println!("🌐 Starting BCAI P2P Service on port {}", self.config.listen_port);

        self.start_time = Some(Instant::now());

        // OLD LOGIC - to be replaced
        // let announcement = self.network_coordinator.announce_capabilities();
        // self.broadcast_message(announcement)?;

        println!("✅ P2P service started and listening");
        Ok(())
    }

    /// Handle incoming network messages
    pub fn handle_incoming_message(
        &mut self,
        sender_id: String,
        message: NetworkMessage,
    ) -> Result<(), P2PError> {
        // This logic should be moved to the application layer (e.g., a new NetworkCoordinator)
        println!("Received message from {}: {:?}", sender_id, message);
        Ok(())
    }

    /// Send message to specific peer
    pub fn send_to_peer(&mut self, peer_id: &str, message: NetworkMessage) -> Result<(), P2PError> {
        // This logic should be moved to the application layer
        println!("Sending message to {}: {:?}", peer_id, message);
        Ok(())
    }

    /// Broadcast message to all peers
    pub fn broadcast_message(&mut self, message: NetworkMessage) -> Result<(), P2PError> {
        // This logic should be moved to the application layer
        println!("Broadcasting message: {:?}", message);
        Ok(())
    }

    /// Perform periodic maintenance
    pub fn perform_maintenance(&mut self) {
        // Placeholder
    }

    /// Update service statistics
    fn update_stats(&mut self) {
        self.stats.peer_count = self.peers.len();
        self.stats.connected_peers = self.peers.len();
        self.stats.network_stats = self.network_coordinator.get_network_stats();

        if let Some(start_time) = self.start_time {
            self.stats.uptime = Instant::now().duration_since(start_time);
        }
    }

    /// Get connected peers
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.values().cloned().collect()
    }

    /// Get P2P service statistics
    pub fn get_stats(&mut self) -> P2PStats {
        self.update_stats();
        self.stats.clone()
    }
}

/// Handle to the P2P service for external control
#[derive(Clone)]
pub struct P2PHandle {
    pub command_sender: mpsc::Sender<Command>,
}

impl P2PHandle {
    pub fn new(command_sender: mpsc::Sender<Command>) -> Self {
        Self { command_sender }
    }

    pub async fn send_message(&self, topic: String, message: Vec<u8>) -> Result<(), P2PError> {
        let (tx, rx) = oneshot::channel();
        self.command_sender
            .send(Command::SendMessage {
                topic: gossipsub::IdentTopic::new(topic),
                message,
                response: tx,
            })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        rx.await.map_err(|e| P2PError::ChannelError(e.to_string()))?
    }

    pub async fn get_peers(&self) -> Result<Vec<PeerId>, P2PError> {
        let (tx, rx) = oneshot::channel();
        self.command_sender
            .send(Command::GetPeers { response: tx })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        rx.await.map_err(|e| P2PError::ChannelError(e.to_string()))
    }

    pub async fn bootstrap(&self) -> Result<(), P2PError> {
        let (tx, rx) = oneshot::channel();
        self.command_sender
            .send(Command::Bootstrap { response: tx })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        rx.await.map_err(|e| P2PError::ChannelError(e.to_string()))?
    }

    pub async fn request(&self, peer_id: PeerId, message: WireMessage) -> Result<WireMessage, P2PError> {
        let (tx, rx) = oneshot::channel();
        self.command_sender
            .send(Command::Request { peer_id, message, response: tx })
            .await
            .map_err(|e| P2PError::ChannelError(e.to_string()))?;
        rx.await.map_err(|e| P2PError::ChannelError(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeRole;
    use tokio::runtime::Runtime;

    fn create_test_node() -> UnifiedNode {
        UnifiedNode::new(NodeRole::Validator, NodeCapability::GpuCompute { model_ids: vec![] })
    }

    #[test]
    fn p2p_service_creation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let config = P2PConfig::default();
            let node = create_test_node();
            let (service, handle) = P2PService::new(config, node).await.unwrap();
            assert_eq!(service.swarm.behaviour().kademlia.kbuckets().count(), 0);
        });
    }

    #[test]
    fn p2p_message_serialization() {
        let message = P2PMessage {
            from_peer: "peer1".to_string(),
            to_peer: Some("peer2".to_string()),
            message_type: "capability_announcement".to_string(),
            payload: vec![1, 2, 3, 4],
            timestamp: 1234567890,
            signature: None,
        };

        // Test that the message can be cloned
        let _cloned = message.clone();

        assert_eq!(message.from_peer, "peer1");
        assert_eq!(message.to_peer, Some("peer2".to_string()));
    }
}
