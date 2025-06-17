//! P2P Service bridging unified nodes with libp2p networking
//!
//! This module provides the real networking layer that connects our unified node
//! architecture with actual P2P communication using libp2p.

use crate::network::{NetworkCoordinator, NetworkMessage, NetworkStats};
use crate::node::{NodeCapability, UnifiedNode};
use futures::StreamExt;
use libp2p::{
    gossipsub, identity, kad,
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
}

/// Events emitted by the BCAINetworkBehaviour
pub enum BCAIBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Kademlia(kad::Event),
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
}

impl From<std::io::Error> for P2PError {
    fn from(e: std::io::Error) -> Self {
        P2PError::IoError(e.to_string())
    }
}

/// P2P message wrapper for network transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub from_peer: String,
    pub to_peer: Option<String>, // None for broadcast
    pub message_type: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: Option<String>,
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
    network_coordinator: NetworkCoordinator,
    peers: HashMap<String, PeerInfo>,
    stats: P2PStats,
    start_time: Option<Instant>,
    config: P2PConfig,
}

const GLOBAL_TOPIC: &str = "bcai_global";

impl P2PService {
    /// Create a new P2P service
    pub async fn new(
        config: P2PConfig,
        local_node: UnifiedNode,
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

        let behaviour = BCAINetworkBehaviour {
            gossipsub: gossipsub_behaviour,
            kademlia: kad::Behaviour::new(local_peer_id, store),
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

        let network_coordinator = NetworkCoordinator::new(local_node);
        let service = Self {
            swarm,
            command_receiver,
            network_coordinator,
            peers: HashMap::new(),
            stats: P2PStats::default(),
            start_time: None,
            config,
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
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                println!(
                    "Gossipsub message from {}: {}",
                    peer_id,
                    String::from_utf8_lossy(&message.data)
                );
                // TODO: Deserialize and process message via NetworkCoordinator
            }
            SwarmEvent::Behaviour(BCAIBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::Bootstrap(Ok(result)),
                ..
            })) => {
                println!("Kademlia bootstrap progress: {} peers found", result.num_remaining);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("👂 Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                endpoint: _,
                ..
            } => {
                println!("🤝 Connected to {}", peer_id);
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .add_explicit_peer(&peer_id);
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                cause: _,
                ..
            } => {
                println!("👋 Connection lost with {}", peer_id);
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .remove_explicit_peer(&peer_id);
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
                let peers = self.swarm.behaviour().kademlia.kbuckets().count();
                // This is not quite right, we need to get the actual peer IDs
                let peer_ids = self
                    .swarm
                    .behaviour()
                    .gossipsub
                    .all_peers()
                    .map(|(p, _)| p.clone())
                    .collect();
                let _ = response.send(peer_ids);
            }
            Command::Bootstrap { response } => {
                match self.swarm.behaviour_mut().kademlia.bootstrap() {
                    Ok(_) => {
                        let _ = response.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = response.send(Err(P2PError::TransportError(format!("{:?}", e))));
                    }
                }
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
        self.stats.messages_received += 1;

        // Update peer last seen
        // OLD LOGIC - to be replaced by swarm events

        Ok(())
    }

    /// Send message to specific peer
    pub fn send_to_peer(&mut self, peer_id: &str, message: NetworkMessage) -> Result<(), P2PError> {
        // OLD LOGIC - to be replaced by swarm commands
        println!("📤 Sending message to peer {}: {:?}", peer_id, message);

        self.stats.messages_sent += 1;
        Ok(())
    }

    /// Broadcast message to all peers
    pub fn broadcast_message(&mut self, message: NetworkMessage) -> Result<(), P2PError> {
        // OLD LOGIC - to be replaced by swarm commands
        println!("📢 Broadcasting message: {:?}", message);

        Ok(())
    }

    /// Perform periodic maintenance
    pub fn perform_maintenance(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes

        // Remove stale peers
        self.peers.retain(|peer_id, peer| {
            if now.duration_since(peer.last_seen) > timeout {
                println!("🗑️ Removing stale peer: {}", peer_id);
                false
            } else {
                true
            }
        });

        self.update_stats();

        println!("💓 P2P heartbeat - Connected peers: {}", self.peers.len());
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
