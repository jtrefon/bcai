//! P2P Service bridging unified nodes with libp2p networking
//!
//! This module provides the real networking layer that connects our unified node
//! architecture with actual P2P communication using libp2p.

use crate::network::NetworkCoordinator;
use crate::node::{NodeCapability, UnifiedNode};
use crate::wire::WireMessage;
use futures::StreamExt;
use libp2p::{
    gossipsub, identity, kad,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

/// Real P2P service for distributed BCAI networking
pub struct P2PService {
    pub swarm: Swarm<BCAINetworkBehaviour>,
    pub command_receiver: mpsc::Receiver<Command>,
    network_coordinator: Arc<Mutex<NetworkCoordinator>>,
    start_time: Option<Instant>,
    config: P2PConfig,
}

const GLOBAL_TOPIC: &str = "bcai_global";

impl P2PService {
    /// Create a new P2P service
    pub async fn new(
        config: P2PConfig,
        network_coordinator: Arc<Mutex<NetworkCoordinator>>,
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

        let mut swarm = libp2p::Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

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
            network_coordinator,
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

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<BCAIBehaviourEvent, kad::ToSwarm<kad::QueryId, <kad::store::MemoryStore as kad::store::RecordStore>::OutRecord>>,
    ) {
        match event {
            SwarmEvent::Behaviour(BCAIBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: _,
                message,
            })) => {
                match bincode::deserialize::<WireMessage>(&message.data) {
                    Ok(wire_message) => {
                        let mut coordinator = self.network_coordinator.lock().unwrap();
                        if let Err(e) =
                            coordinator.handle_wire_message(wire_message, &peer_id.to_string())
                        {
                            eprintln!("Error processing message from {}: {}", peer_id, e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize message from {}: {}", peer_id, e);
                    }
                }
            }
            SwarmEvent::Behaviour(BCAIBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::Bootstrap(Ok(result)),
                ..
            })) => {
                println!("Kademlia bootstrap progress: {} peers found", result.num_remaining);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("👂 Listening on {}", address.with_p2p(*self.swarm.local_peer_id()).unwrap());
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

    #[test]
    fn p2p_message_serialization() {
        let message = WireMessage::Ping;
        let serialized = bincode::serialize(&message).unwrap();
        let deserialized: WireMessage = bincode::deserialize(&serialized).unwrap();
        assert!(matches!(deserialized, WireMessage::Ping));
    }

    #[test]
    fn p2p_service_creation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let config = P2PConfig::default();
            let node = create_test_node();
            let coordinator = Arc::new(Mutex::new(NetworkCoordinator::new(node)));
            let (service, handle) = P2PService::new(config, coordinator).await.unwrap();
            assert_eq!(service.swarm.behaviour().kademlia.kbuckets().count(), 0);
        });
    }

    #[test]
    // ... existing code ...
} 