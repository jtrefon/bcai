use libp2p::{
    core::{transport::MemoryTransport, upgrade},
    identity, noise,
    request_response::{Config as RequestResponseConfig, ProtocolSupport},
    swarm::Swarm,
    tcp, yamux, PeerId, Transport,
};
use std::time::Duration;

use crate::behaviour::Behaviour;

pub fn create_memory_transport(id: &identity::Keypair) -> Result<impl Transport<Output = (PeerId, libp2p::core::muxing::StreamMuxerBox)> + Clone, Box<dyn std::error::Error>> {
    let transport = MemoryTransport::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(id)?)
        .multiplex(yamux::Config::default())
        .timeout(Duration::from_secs(20))
        .boxed();
    Ok(transport)
}

pub fn create_tcp_transport(id: &identity::Keypair) -> Result<impl Transport<Output = (PeerId, libp2p::core::muxing::StreamMuxerBox)> + Clone, Box<dyn std::error::Error>> {
    let transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(id)?)
        .multiplex(yamux::Config::default())
        .timeout(Duration::from_secs(20))
        .boxed();
    Ok(transport)
}

pub fn create_behaviour() -> Behaviour {
    let ping = libp2p::ping::Behaviour::default();
    let cfg = RequestResponseConfig::default();
    let protocols = std::iter::once(("/job/1.0.0".to_string(), ProtocolSupport::Full));
    let req = libp2p::request_response::Behaviour::new(protocols, cfg);
    Behaviour { ping, req }
}

pub fn create_swarm<T>(transport: T, behaviour: Behaviour, peer_id: PeerId) -> Swarm<Behaviour>
where
    T: Transport + Clone + Send + Unpin + 'static,
    T::Output: Send + Unpin,
    T::Error: Send + Sync + std::error::Error,
    T::Dial: Send,
    T::ListenerUpgrade: Send,
{
    Swarm::new(transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor())
} 