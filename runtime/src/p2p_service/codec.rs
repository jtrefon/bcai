//! Defines the request-response codec and wire format for P2P messages.

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::request_response;
use serde::{Deserialize, Serialize};

/// The message format that goes over the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMessage {
    Block(crate::blockchain::block::Block),
    Transaction(crate::blockchain::transaction::Transaction),
    Ping,
    Pong,
}

/// The codec used for the request-response protocol.
/// It uses JSON for serialization.
#[derive(Debug, Clone, Default)]
pub struct WireCodec;

#[derive(Clone)]
pub struct WireProtocol();

impl std::fmt::Debug for WireProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WireProtocol").finish()
    }
}

impl AsRef<[u8]> for WireProtocol {
    fn as_ref(&self) -> &[u8] {
        b"/bcai/wire/1.0.0"
    }
}

impl AsRef<str> for WireProtocol {
    fn as_ref(&self) -> &str {
        "/bcai/wire/1.0.0"
    }
}

impl request_response::Codec for WireCodec {
    type Protocol = WireProtocol;
    type Request = WireMessage;
    type Response = WireMessage;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_message(io).await
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_message(io).await
    }

    async fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_message(io, &req).await
    }

    async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, res: Self::Response) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_message(io, &res).await
    }
}

// Helper functions to handle reading and writing, with basic error handling.
async fn read_message<T, M>(io: &mut T) -> std::io::Result<M>
where
    T: AsyncRead + Unpin + Send,
    M: for<'de> Deserialize<'de>,
{
    let mut vec = Vec::new();
    io.read_to_end(&mut vec).await?;
    serde_json::from_slice(&vec).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

async fn write_message<T, M>(io: &mut T, msg: &M) -> std::io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
    M: Serialize,
{
    let buf = serde_json::to_vec(msg).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    io.write_all(&buf).await
} 