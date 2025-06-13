use futures::StreamExt;
use libp2p::{
    core::{transport::MemoryTransport, upgrade},
    identity, noise,
    ping::{Behaviour as Ping, Event as PingEvent},
    swarm::{Swarm, SwarmEvent},
    yamux, Multiaddr, PeerId, Transport,
};
use rand::Rng;
use std::time::Duration;

pub struct Node {
    pub peer_id: PeerId,
    swarm: Swarm<Ping>,
}

impl Node {
    pub fn new() -> Self {
        let id = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id.public());

        let transport = MemoryTransport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&id).expect("noise config"))
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let behaviour = Ping::default();
        let swarm =
            Swarm::new(transport, behaviour, peer_id, libp2p_swarm::Config::with_tokio_executor());
        Self { peer_id, swarm }
    }

    pub fn listen(&mut self) -> Multiaddr {
        let port: u64 = rand::thread_rng().gen_range(1..u64::MAX);
        let addr: Multiaddr = format!("/memory/{port}").parse().unwrap();
        self.swarm.listen_on(addr.clone()).unwrap();
        addr
    }

    pub fn dial(&mut self, addr: Multiaddr) {
        self.swarm.dial(addr).unwrap();
    }

    pub async fn next_event(&mut self) -> PingEvent {
        loop {
            if let SwarmEvent::Behaviour(e) = self.swarm.select_next_some().await {
                return e;
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn nodes_can_ping() {
        let mut a = Node::new();
        let mut b = Node::new();
        let addr = a.listen();
        b.dial(addr);
        let res = timeout(Duration::from_secs(5), async {
            loop {
                tokio::select! {
                    e = a.next_event() => if e.result.is_ok() { break },
                    e = b.next_event() => if e.result.is_ok() { break },
                }
            }
        })
        .await;
        assert!(res.is_ok(), "ping timeout");
    }
}
