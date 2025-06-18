use libp2p::{
    core::{transport::MemoryTransport, upgrade},
    identity, noise,
    request_response::{Config as RequestResponseConfig, ProtocolSupport},
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport,
};
use rand::Rng;
use std::time::Duration;

use crate::{behaviour::{Behaviour, Capability, JobRequest, JobResponse, NodeEvent}, codec::JobCodec};

pub struct Node {
    pub peer_id: PeerId,
    swarm: Swarm<Behaviour>,
    capability: Capability,
}

impl Node {
    pub fn new(cpus: u8, gpus: u8) -> Self {
        Self::new_memory(cpus, gpus)
    }

    /// Create a node using an in-memory transport. Primarily used for tests.
    pub fn new_memory(cpus: u8, gpus: u8) -> Self {
        let id = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id.public());

        let transport = MemoryTransport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&id).expect("noise config"))
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let ping = libp2p::ping::Behaviour::default();
        let cfg = RequestResponseConfig::default();
        let protocols = std::iter::once(("/job/1.0.0".to_string(), ProtocolSupport::Full));
        let req = libp2p::request_response::Behaviour::new(protocols, cfg);
        let behaviour = Behaviour { ping, req };
        let swarm = Swarm::new(transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor());
        Self { peer_id, swarm, capability: Capability { cpus, gpus } }
    }

    /// Create a node that communicates over TCP.
    pub fn new_tcp(cpus: u8, gpus: u8) -> Self {
        let id = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id.public());

        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&id).expect("noise config"))
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let ping = libp2p::ping::Behaviour::default();
        let cfg = RequestResponseConfig::default();
        let protocols = std::iter::once(("/job/1.0.0".to_string(), ProtocolSupport::Full));
        let req = libp2p::request_response::Behaviour::new(protocols, cfg);
        let behaviour = Behaviour { ping, req };
        let swarm = Swarm::new(transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor());
        Self { peer_id, swarm, capability: Capability { cpus, gpus } }
    }

    pub fn capability(&self) -> Capability {
        self.capability.clone()
    }

    pub fn listen(&mut self) -> Multiaddr {
        let port: u64 = rand::thread_rng().gen_range(1..u64::MAX);
        let addr: Multiaddr = format!("/memory/{port}").parse().expect("memory addr");
        self.swarm.listen_on(addr.clone()).expect("listen_on");
        addr
    }

    /// Listen on a TCP port, returning the bound multiaddress.
    pub fn listen_tcp(&mut self, port: u16) -> Multiaddr {
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{port}").parse().expect("tcp addr");
        self.swarm.listen_on(addr.clone()).expect("listen_on");
        addr
    }

    pub fn dial(&mut self, addr: Multiaddr) {
        self.swarm.dial(addr).expect("dial");
    }

    pub fn send_handshake(&mut self, peer: PeerId) {
        let req = JobRequest::Handshake(self.capability.clone());
        self.swarm.behaviour_mut().req.send_request(&peer, req);
    }

    pub fn send_train(&mut self, peer: PeerId, data: Vec<u8>) {
        let req = JobRequest::Train(data);
        self.swarm.behaviour_mut().req.send_request(&peer, req);
    }

    fn train_lr(data: &[u8]) -> Vec<f32> {
        // TODO: implement ML logic properly; using placeholder for now.
        data.iter().map(|b| *b as f32).collect()
    }

    pub async fn next_event(&mut self) -> NodeEvent {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(evt) => return evt,
                _ => {}
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new_memory(1, 0)
    }
} 