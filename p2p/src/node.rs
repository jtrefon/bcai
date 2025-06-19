use libp2p::{
    identity,
    swarm::{Swarm, SwarmEvent},
    Multiaddr, PeerId,
};

use crate::{
    behaviour::{Behaviour, Capability, NodeEvent},
    transport::{create_memory_transport, create_tcp_transport, create_behaviour, create_swarm},
    network::NetworkOperations,
    training::MLTrainer,
};

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

        let transport = create_memory_transport(&id).expect("memory transport");
        let behaviour = create_behaviour();
        let swarm = create_swarm(transport, behaviour, peer_id);
        
        Self { peer_id, swarm, capability: Capability { cpus, gpus } }
    }

    /// Create a node that communicates over TCP.
    pub fn new_tcp(cpus: u8, gpus: u8) -> Self {
        let id = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id.public());

        let transport = create_tcp_transport(&id).expect("tcp transport");
        let behaviour = create_behaviour();
        let swarm = create_swarm(transport, behaviour, peer_id);

        Self { peer_id, swarm, capability: Capability { cpus, gpus } }
    }

    pub fn capability(&self) -> Capability {
        self.capability.clone()
    }

    pub fn listen(&mut self) -> Multiaddr {
        let addr = NetworkOperations::generate_memory_address();
        self.swarm.listen_on(addr.clone()).expect("listen_on");
        addr
    }

    /// Listen on a TCP port, returning the bound multiaddress.
    pub fn listen_tcp(&mut self, port: u16) -> Multiaddr {
        let addr = NetworkOperations::generate_tcp_address(port);
        self.swarm.listen_on(addr.clone()).expect("listen_on");
        addr
    }

    pub fn dial(&mut self, addr: Multiaddr) {
        self.swarm.dial(addr).expect("dial");
    }

    pub fn send_handshake(&mut self, peer: PeerId) {
        let req = NetworkOperations::create_handshake_request(self.capability.clone());
        self.swarm.behaviour_mut().req.send_request(&peer, req);
    }

    pub fn send_train(&mut self, peer: PeerId, data: Vec<u8>) {
        let req = NetworkOperations::create_train_request(data);
        self.swarm.behaviour_mut().req.send_request(&peer, req);
    }

    pub fn train_lr(&self, data: &[u8]) -> Vec<f32> {
        MLTrainer::train_linear_regression(data)
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