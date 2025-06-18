use libp2p::{
    ping::{Behaviour as Ping, Event as PingEvent},
    request_response::{Behaviour as RequestResponse, Event as RequestResponseEvent, ProtocolSupport},
    swarm::NetworkBehaviour,
};
use serde::{Deserialize, Serialize};

use crate::codec::JobCodec;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub cpus: u8,
    pub gpus: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobRequest {
    Handshake(Capability),
    Train(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobResponse {
    HandshakeAck(Capability),
    TrainResult(Vec<f32>),
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NodeEvent", prelude = "libp2p_swarm::derive_prelude")]
pub struct Behaviour {
    pub ping: Ping,
    pub req: RequestResponse<JobCodec>,
}

#[derive(Debug)]
pub enum NodeEvent {
    Ping(PingEvent),
    RequestResponse(RequestResponseEvent<JobRequest, JobResponse>),
}

impl From<PingEvent> for NodeEvent {
    fn from(e: PingEvent) -> Self {
        NodeEvent::Ping(e)
    }
}

impl From<RequestResponseEvent<JobRequest, JobResponse>> for NodeEvent {
    fn from(e: RequestResponseEvent<JobRequest, JobResponse>) -> Self {
        NodeEvent::RequestResponse(e)
    }
} 