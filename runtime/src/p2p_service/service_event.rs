use libp2p::{gossipsub, kad, request_response, swarm::SwarmEvent, PeerId};
use super::{
    behaviour::{BCAIBehaviourEvent, BCAINetworkBehaviour},
    error::P2PError,
    service::P2PService,
};

/// Extended implementation for `P2PService` that handles libp2p swarm events.
impl P2PService {
    pub(super) async fn handle_swarm_event(&mut self, event: SwarmEvent<BCAIBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BCAIBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: _,
                message_id: _,
                message,
            })) => {
                println!(
                    "Received gossipsub message: {:?}",
                    String::from_utf8_lossy(&message.data)
                );
            }
            SwarmEvent::Behaviour(BCAIBehaviourEvent::Kademlia(event)) => {
                if let kad::Event::OutboundQueryCompleted { result, .. } = event {
                    if let kad::QueryResult::GetClosestPeers(Ok(res)) = result {
                        for peer in res.peers {
                            let id = peer.to_string();
                            self.peers
                                .entry(id.clone())
                                .or_insert(super::types::PeerInfo {
                                    peer_id: id.clone(),
                                    capabilities: None,
                                    last_seen: std::time::Instant::now(),
                                    reputation: 0,
                                    connection_count: 1,
                                });
                        }
                        self.stats.peer_count = self.peers.len();
                    }
                } else {
                    tracing::debug!(?event, "Kademlia event");
                }
            }
            SwarmEvent::Behaviour(BCAIBehaviourEvent::RequestResponse(
                request_response::Event::Message { peer, message },
            )) => {
                match message {
                    request_response::Message::Request { request, channel, .. } => {
                        let response = match request {
                            super::codec::WireMessage::Ping => super::codec::WireMessage::Pong,
                            _ => super::codec::WireMessage::Pong,
                        };
                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(channel, response);
                    }
                    request_response::Message::Response { request_id, response } => {
                        if let Some(tx) = self.request_map.remove(&request_id) {
                            let _ = tx.send(Ok(response));
                        }
                    }
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {}", peer_id);
            }
            _ => {}
        }
    }
} 