use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Join { username: String },
    Message { username: String, content: String },
    Leave { username: String },
}

pub fn write_packet(stream: &mut std::net::TcpStream, packet: &Packet) -> io::Result<()> {
    let bytes = serde_json::to_vec(packet)?;
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&bytes)?;
    stream.flush()?;
    Ok(())
}

pub fn read_packet(stream: &mut std::net::TcpStream) -> io::Result<Option<Packet>> {
    let mut len_bytes = [0u8; 4];
    match stream.read_exact(&mut len_bytes) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_be_bytes(len_bytes) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    let packet: Packet = serde_json::from_slice(&buf)?;
    Ok(Some(packet))
}