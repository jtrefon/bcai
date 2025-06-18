use super::{
    command::Command,
    error::P2PError,
    service::P2PService,
};

impl P2PService {
    pub(super) async fn handle_command(&mut self, command: Command) {
        match command {
            Command::SendMessage { topic, message, response } => {
                let result = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic, message)
                    .map(|_| ())
                    .map_err(|e| P2PError::SerializationFailed(e.to_string()));
                let _ = response.send(result);
            }
            Command::GetPeers { response } => {
                let peers = self
                    .swarm
                    .behaviour()
                    .kademlia
                    .kbuckets()
                    .flat_map(|b| b.iter().map(|e| *e.node.key.preimage()))
                    .collect();
                let _ = response.send(peers);
            }
            Command::Bootstrap { response } => {
                let result = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .bootstrap()
                    .map(|_| ())
                    .map_err(|e| P2PError::Network(format!("Bootstrap failed: {:?}", e)));
                let _ = response.send(result);
            }
            Command::Request { peer_id, message, response } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer_id, message);
                self.request_map.insert(request_id, response);
            }
        }
    }
} 