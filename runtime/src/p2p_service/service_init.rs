use super::{
    behaviour::{BCAIBehaviourEvent, BCAINetworkBehaviour},
    codec::WireMessage,
    command::{Command, P2PHandle},
    config::P2PConfig,
    error::P2PError,
    types::{PeerInfo, P2PStats},
};
use futures::StreamExt;
use libp2p::{
    gossipsub, identity, kad,
    request_response::{self, ProtocolSupport},
    swarm::{Swarm, SwarmEvent},
    Multiaddr, PeerId, Transport,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use super::service::P2PService;

impl P2PService {
use super::{
    behaviour::{BCAIBehaviourEvent, BCAINetworkBehaviour},
    codec::WireMessage,
    command::{Command, P2PHandle},
    config::P2PConfig,
    error::P2PError,
    types::{PeerInfo, P2PStats},
};
use futures::StreamExt;
use libp2p::{
    gossipsub, identity, kad,
    request_response::{self, ProtocolSupport},
    swarm::{Swarm, SwarmEvent},
    Multiaddr, PeerId, Transport,
};
use std::collections::HashMap;
use std::str::FromStr; // Required for Multiaddr::from_str
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use super::service::P2PService;
use tracing::{info, warn}; // Added warn for parsing errors

impl P2PService {
    /// Create a new P2P service, which includes the service itself and a handle for interaction.
    pub async fn new(config: P2PConfig) -> Result<(Self, P2PHandle), P2PError> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("🤖 Local Peer ID: {}", local_peer_id); // Changed to info!

        let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(
                libp2p::noise::Config::new(&local_key)
                    .map_err(|e| P2PError::TransportError(e.to_string()))?,
            )
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .build()
            .map_err(|s| P2PError::ConnectionFailed(s.to_string()))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|s| P2PError::SerializationFailed(s.to_string()))?;

        let mut kademlia = // Made mutable to add addresses
            kad::Behaviour::new(local_peer_id, kad::store::MemoryStore::new(local_peer_id));

        // Add bootstrap nodes to Kademlia
        for multiaddr_str in &config.bootstrap_nodes {
            match Multiaddr::from_str(multiaddr_str) {
                Ok(multiaddr) => {
                    // Extract PeerId if present in the multiaddress
                    if let Some(libp2p::multiaddr::Protocol::P2p(peer_id_bytes)) = multiaddr.iter().last() {
                         match PeerId::from_bytes(&peer_id_bytes.to_bytes()) {
                            Ok(peer_id) => {
                                kademlia.add_address(&peer_id, multiaddr.clone());
                                info!("Added bootstrap node: {} with peer_id {}", multiaddr_str, peer_id);
                            }
                            Err(e) => {
                                warn!("Failed to parse PeerId from bootstrap address {}: {}", multiaddr_str, e);
                                // Optionally, still add the address if you want Kademlia to try resolving it
                                // kademlia.add_address(&PeerId::random(), multiaddr.clone()); // Or handle differently
                            }
                        }
                    } else {
                        // If PeerId is not part of the multiaddr, Kademlia will attempt to discover it.
                        // This requires the bootstrap node to be discoverable via other means or already known.
                        // For robust bootstrapping, multiaddrs that include /p2p/<PeerID> are preferred.
                        // Adding a random peer ID here is a placeholder; typically, you'd only add addresses
                        // if you can associate them with a PeerId or if Kademlia supports adding just addresses
                        // for bootstrap (which it does, but it's less direct for routing table).
                        // For now, we'll just add it and let Kademlia figure it out.
                        // A better approach might be to require /p2p/ in bootstrap addrs.
                        warn!("Bootstrap address {} does not contain a PeerId. Adding address to Kademlia for discovery.", multiaddr_str);
                        kademlia.add_address(&PeerId::random(), multiaddr.clone()); // Kademlia can handle this, but it's not ideal.
                                                                     // It's better if the address includes the PeerId.
                        // Cannot reliably dial without a PeerId here, Kademlia will handle discovery.
                    }
                }
                Err(e) => {
                    warn!("Failed to parse bootstrap multiaddress {}: {}", multiaddr_str, e);
                }
            }
        }

        let request_response = request_response::Behaviour::new(
            [(super::codec::WireProtocol(), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        let behaviour = BCAINetworkBehaviour {
            gossipsub,
            kademlia,
            request_response,
        };

        let mut swarm = Swarm::with_tokio_executor(transport, behaviour, local_peer_id);

        // Dial bootstrap nodes after adding them to Kademlia's table
        // This can help kickstart discovery and connection.
        for multiaddr_str in &config.bootstrap_nodes {
            if let Ok(multiaddr) = Multiaddr::from_str(multiaddr_str) {
                if let Some(libp2p::multiaddr::Protocol::P2p(peer_id_bytes)) = multiaddr.iter().last() {
                    if let Ok(peer_id) = PeerId::from_bytes(&peer_id_bytes.to_bytes()) {
                        match swarm.dial(peer_id) {
                            Ok(_) => info!("Attempting to dial bootstrap node: {}", multiaddr_str),
                            Err(e) => warn!("Error dialing bootstrap node {}: {:?}", multiaddr_str, e),
                        }
                    }
                }
                // If PeerId is not in multiaddr, direct dialing is less certain.
                // Kademlia's add_address is the primary mechanism then.
                // else {
                //    match swarm.dial(multiaddr.clone()) {
                //        Ok(_) => info!("Attempting to dial bootstrap address (no PeerId): {}", multiaddr_str),
                //        Err(e) => warn!("Error dialing bootstrap address {}: {:?}", multiaddr_str, e),
                //    }
                // }
            }
        }

        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", config.listen_port);
        swarm.listen_on(listen_addr.parse().unwrap()).unwrap();

        let (command_sender, command_receiver) = mpsc::channel(32);
        let handle = P2PHandle::new(command_sender);

        // Subscribe to the global topic
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&gossipsub::IdentTopic::new(super::service::GLOBAL_TOPIC))
            .unwrap();

        let service = Self {
            swarm,
            command_receiver,
            peers: HashMap::new(),
            stats: P2PStats::default(), // Use the Default impl
            start_time: Some(Instant::now()),
            config,
            request_map: HashMap::new(),
        };

        Ok((service, handle))
    }
} 