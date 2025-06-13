use async_trait::async_trait;
use futures::StreamExt;
use libp2p::{
    core::{transport::MemoryTransport, upgrade},
    identity, noise,
    ping::{Behaviour as Ping, Event as PingEvent},
    request_response::{
        Behaviour as RequestResponse, Codec, Config as RequestResponseConfig,
        Event as RequestResponseEvent, Message as RequestResponseMessage, ProtocolSupport,
    },
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    yamux, Multiaddr, PeerId, Transport,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

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

#[derive(Clone, Default)]
struct JobCodec;

#[async_trait]
impl Codec for JobCodec {
    type Protocol = String;
    type Request = JobRequest;
    type Response = JobResponse;

    async fn read_request<T>(&mut self, _: &String, io: &mut T) -> io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
        Ok(bincode::deserialize(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }

    async fn read_response<T>(&mut self, _: &String, io: &mut T) -> io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
        Ok(bincode::deserialize(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }

    async fn write_request<T>(
        &mut self,
        _: &String,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let bytes =
            bincode::serialize(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        futures::io::AsyncWriteExt::write_all(io, &bytes).await?;
        futures::io::AsyncWriteExt::close(io).await
    }

    async fn write_response<T>(
        &mut self,
        _: &String,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let bytes =
            bincode::serialize(&res).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        futures::io::AsyncWriteExt::write_all(io, &bytes).await?;
        futures::io::AsyncWriteExt::close(io).await
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NodeEvent", prelude = "libp2p_swarm::derive_prelude")]
struct Behaviour {
    ping: Ping,
    req: RequestResponse<JobCodec>,
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

pub struct Node {
    pub peer_id: PeerId,
    swarm: Swarm<Behaviour>,
    capability: Capability,
}

impl Node {
    pub fn new(cpus: u8, gpus: u8) -> Self {
        let id = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id.public());

        let transport = MemoryTransport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&id).expect("noise config"))
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let ping = Ping::default();
        let cfg = RequestResponseConfig::default();
        let protocols = std::iter::once(("/job/1.0.0".to_string(), ProtocolSupport::Full));
        let req = RequestResponse::new(protocols, cfg);
        let behaviour = Behaviour { ping, req };
        let swarm =
            Swarm::new(transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor());
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
        let text = std::str::from_utf8(data).expect("utf8 dataset");
        let mut floats = Vec::new();
        for line in text.lines() {
            for part in line.split(',') {
                floats.push(part.parse::<f32>().expect("parse float"));
            }
        }
        let rows = floats.len() / 6;
        let mut weights = [0.0f32; 5];
        for _ in 0..10 {
            let mut grads = [0.0f32; 5];
            for i in 0..rows {
                let start = i * 6;
                let x = &floats[start..start + 5];
                let y = floats[start + 5];
                let pred: f32 = weights.iter().zip(x).map(|(w, xi)| w * xi).sum();
                let err = pred - y;
                for j in 0..5 {
                    grads[j] += err * x[j];
                }
            }
            for j in 0..5 {
                weights[j] -= 0.01 * grads[j] / rows as f32;
            }
        }
        weights.to_vec()
    }

    pub async fn next_event(&mut self) -> NodeEvent {
        loop {
            if let SwarmEvent::Behaviour(event) = self.swarm.select_next_some().await {
                match event {
                    NodeEvent::RequestResponse(ev) => {
                        if let RequestResponseEvent::Message { peer, connection_id, message } = ev {
                            match message {
                                RequestResponseMessage::Request { request, channel, .. } => {
                                    match request {
                                        JobRequest::Handshake(_) => {
                                            let resp =
                                                JobResponse::HandshakeAck(self.capability.clone());
                                            self.swarm
                                                .behaviour_mut()
                                                .req
                                                .send_response(channel, resp)
                                                .expect("send handshake response");
                                            continue;
                                        }
                                        JobRequest::Train(data) => {
                                            let weights = Self::train_lr(&data);
                                            let resp = JobResponse::TrainResult(weights);
                                            self.swarm
                                                .behaviour_mut()
                                                .req
                                                .send_response(channel, resp)
                                                .expect("send train response");
                                            continue;
                                        }
                                    }
                                }
                                RequestResponseMessage::Response { request_id, response } => {
                                    return NodeEvent::RequestResponse(
                                        RequestResponseEvent::Message {
                                            peer,
                                            connection_id,
                                            message: RequestResponseMessage::Response {
                                                request_id,
                                                response,
                                            },
                                        },
                                    );
                                }
                            }
                        }
                        return NodeEvent::RequestResponse(ev);
                    }
                    other => return other,
                }
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new(4, 1)
    }
}
