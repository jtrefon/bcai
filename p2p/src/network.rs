use libp2p::{Multiaddr, PeerId};
use rand::Rng;
use crate::behaviour::{Capability, JobRequest};

pub struct NetworkOperations;

impl NetworkOperations {
    pub fn generate_memory_address() -> Multiaddr {
        let port: u64 = rand::thread_rng().gen_range(1..u64::MAX);
        format!("/memory/{port}").parse().expect("memory addr")
    }

    pub fn generate_tcp_address(port: u16) -> Multiaddr {
        format!("/ip4/0.0.0.0/tcp/{port}").parse().expect("tcp addr")
    }

    pub fn create_handshake_request(capability: Capability) -> JobRequest {
        JobRequest::Handshake(capability)
    }

    pub fn create_train_request(data: Vec<u8>) -> JobRequest {
        JobRequest::Train(data)
    }
} 