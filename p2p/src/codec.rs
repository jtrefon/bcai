use async_trait::async_trait;
use libp2p::request_response::Codec;
use serde::{Deserialize, Serialize};
use std::io;

use crate::{JobRequest, JobResponse};

#[derive(Clone, Default)]
pub struct JobCodec;

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
        let bytes = bincode::serialize(&req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
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
        let bytes = bincode::serialize(&res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        futures::io::AsyncWriteExt::write_all(io, &bytes).await?;
        futures::io::AsyncWriteExt::close(io).await
    }
} 