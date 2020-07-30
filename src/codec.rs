#![allow(dead_code)]
use std::io;

use actix::Message;
use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use serde_json as json;
use tokio_util::codec::{Decoder, Encoder};

/// Client request
#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
#[serde(tag = "cmd", content = "data")]
pub enum MemsonRequest {
    /// Send command
    Command(String),
    /// Ping
    Ping,
}

/// Server response
#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
#[serde(tag = "cmd", content = "data")]
pub enum MemsonResponse {
    // Heartbeat
    Ping,
    /// Message
    Data(String),
}

/// Codec for Client -> Server transport
pub struct MemsonCodec;

impl Decoder for MemsonCodec {
    type Item = MemsonRequest;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.advance(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<MemsonRequest>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<MemsonResponse> for MemsonCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        msg: MemsonResponse,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}

/// Codec for Server -> Client transport
pub struct ClientMemsonCodec;

impl Decoder for ClientMemsonCodec {
    type Item = MemsonResponse;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.advance(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<MemsonResponse>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<MemsonRequest> for ClientMemsonCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        msg: MemsonRequest,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}