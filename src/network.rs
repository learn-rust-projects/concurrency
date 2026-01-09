use bytes::BytesMut;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{info, warn};

use crate::{
    backend::Backend,
    cmd::{Command, CommandExecutor},
    resp::{RespDecode, RespEncode, RespError, RespFrame},
};

#[derive(Debug)]
pub struct RespFrameCodec;

pub async fn stream_handler(stream: TcpStream, backend: Backend) -> anyhow::Result<()> {
    let mut framed = Framed::new(stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(frame) => {
                let frame = frame?;

                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                let response = request_handler(request).await?;
                info!("write response: {:?}", response);
                framed.send(response.frame).await?;
            }
            None => {
                warn!("client closed connection");
            }
        };
    }
}
#[derive(Debug)]
pub struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}

#[derive(Debug)]
pub struct RedisResponse {
    frame: RespFrame,
}

async fn request_handler(request: RedisRequest) -> anyhow::Result<RedisResponse> {
    let RedisRequest { frame, backend } = request;
    let command: Command = frame.try_into()?;
    info!("execute command: {:?}", command);
    let frame = command.execute(&backend);
    Ok(RedisResponse { frame })
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let frame = item.encode();
        dst.extend_from_slice(&frame);
        Ok(())
    }
}
impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
