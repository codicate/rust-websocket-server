use crate::http::Request;
use crate::http::Response;
use base64::prelude::*;
use sha1::{Digest, Sha1};
use std::io::Read;
use std::net::TcpStream;

#[derive(PartialEq)]
pub enum FrameKind {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

pub struct Frame {
    pub is_fin: bool,
    pub kind: FrameKind,
    pub payload: Vec<u8>,
}

impl Frame {
    // Data frame reference: https://datatracker.ietf.org/doc/html/rfc6455#section-5.2
    pub fn new(mut stream: &TcpStream) -> std::io::Result<Frame> {
        std::thread::sleep(std::time::Duration::from_micros(1)); // Wait for data to be available
        let mut buffer = [0u8; 2];
        stream.read_exact(&mut buffer[..1])?; // Read the first byte

        let is_fin = buffer[0] & 128 != 0; // First bit
        let opcode = buffer[0] & 15; // Last 4 bits

        let kind = match opcode {
            1 => FrameKind::Text,
            8 => FrameKind::Close,
            _ => {
                println!("Unsupported opcode: {}", opcode);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unsupported opcode",
                ));
            }
        };

        let mut frame = Frame {
            is_fin,
            kind,
            payload: vec![],
        };

        if frame.kind == FrameKind::Close {
            return Ok(frame);
        }

        stream.read_exact(&mut buffer[..1])?; // Read the second byte
        let mut payload_len = (buffer[0] & 127) as u16; // Last 7 bits

        if payload_len == 127 {
            println!("Payload is too large");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Payload is too large",
            ));
        } else if payload_len == 126 {
            stream.read_exact(&mut buffer)?; // Read 2 bytes for extended payload length
            payload_len = u16::from_be_bytes(buffer);
        }

        let mut mask_buffer = [0u8; 4];
        stream.read_exact(&mut mask_buffer)?; // Read 4-byte mask
        let mask = u32::from_be_bytes(mask_buffer);

        let mut payload = vec![0u8; payload_len as usize];
        stream.read_exact(&mut payload)?; // Read the payload
        unmask_payload(mask, &mut payload);
        frame.payload = payload;

        Ok(frame)
    }
}

fn unmask_payload(mask: u32, payload: &mut [u8]) {
    let mask_bytes = mask.to_be_bytes(); // Convert mask to a byte array
    for (i, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask_bytes[i % 4]; // XOR each payload byte with the corresponding mask byte
    }
}

pub fn ws_handshake(request: Request) -> Response {
    let Some(key) = request.headers.get("Sec-WebSocket-Key") else {
        return Response::new(400, "Bad Request");
    };

    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"); // Magic string
    let accept = BASE64_STANDARD.encode(hasher.finalize());

    let mut response = Response::new(101, "Switching Protocols");
    response.add_header("Upgrade", "websocket");
    response.add_header("Connection", "Upgrade");
    response.add_header("Sec-WebSocket-Accept", &accept);
    response
}

pub fn encode_message(message: String) -> Vec<u8> {
    let mut frame = vec![];
    let length = message.len();
    frame.push(0x81); // Text frame opcode
    frame.push(length as u8); // Message length
    frame.extend_from_slice(message.as_bytes());
    frame
}
