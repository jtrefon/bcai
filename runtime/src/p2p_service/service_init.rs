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
    /// Create a new P2P service, which includes the service itself and a handle for interaction.
    pub async fn new(config: P2PConfig) -> Result<(Self, P2PHandle), P2PError> {
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

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .build()
            .map_err(|s| P2PError::ConnectionFailed(s.to_string()))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|s| P2PError::SerializationFailed(s.to_string()))?;

        let kademlia =
            kad::Behaviour::new(local_peer_id, kad::store::MemoryStore::new(local_peer_id));

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
            stats: P2PStats { // This should be moved to a `Default` impl
                peer_count: 0,
                connected_peers: 0,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                uptime: Duration::from_secs(0),
                network_stats: Default::default(),
            },
            start_time: Some(Instant::now()),
            config,
            request_map: HashMap::new(),
        };

        Ok((service, handle))
    }
} 