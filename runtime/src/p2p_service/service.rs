//! Defines the core `P2PService` and its main event loop.

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
    Multiaddr, PeerId,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

pub(super) const GLOBAL_TOPIC: &str = "bcai_global";

/// The main P2P service struct. It owns the libp2p Swarm and handles all
/// network events and application-level commands.
pub struct P2PService {
    pub swarm: Swarm<BCAINetworkBehaviour>,
    pub command_receiver: mpsc::Receiver<Command>,
    peers: HashMap<String, PeerInfo>,
    stats: P2PStats,
    start_time: Option<Instant>,
    config: P2PConfig,
    pub(super) request_map: HashMap<
        request_response::RequestId,
        oneshot::Sender<Result<WireMessage, P2PError>>,
    >,
}

impl P2PService {
    /// The main event loop of the P2P service.
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                },
                Some(command) = self.command_receiver.recv() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    // Implementations moved to `service_event.rs` and `service_command.rs`.
} 