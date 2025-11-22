// TLV (Type-Length-Value) framing for Chi protocol
// Wire format: [type: u8][length: u32][value: bytes]

use std::io::{self, Read, Write};

/// Message type identifier
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Text = 1,
    Image = 2,
    // Future: Geometry = 3, Notification = 4, etc.
}

impl MessageType {
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(MessageType::Text),
            2 => Some(MessageType::Image),
            _ => None,
        }
    }
}

/// TLV frame encoder
pub fn encode_frame(msg_type: MessageType, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(1 + 4 + payload.len());

    // Type (1 byte)
    frame.push(msg_type as u8);

    // Length (4 bytes, big-endian for network byte order)
    let len = payload.len() as u32;
    frame.extend_from_slice(&len.to_be_bytes());

    // Value (payload)
    frame.extend_from_slice(payload);

    frame
}

/// TLV frame decoder
pub fn decode_frame<R: Read>(reader: &mut R) -> io::Result<(MessageType, Vec<u8>)> {
    // Read type (1 byte)
    let mut type_buf = [0u8; 1];
    reader.read_exact(&mut type_buf)?;

    let msg_type = MessageType::from_u8(type_buf[0])
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Unknown message type"))?;

    // Read length (4 bytes)
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Read payload
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;

    Ok((msg_type, payload))
}

/// Write TLV frame to a writer (e.g., TCP stream)
pub fn write_frame<W: Write>(writer: &mut W, msg_type: MessageType, payload: &[u8]) -> io::Result<()> {
    let frame = encode_frame(msg_type, payload);
    writer.write_all(&frame)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_encode_decode_roundtrip() {
        let payload = b"Hello, Chi!";
        let frame = encode_frame(MessageType::Text, payload);

        let mut cursor = Cursor::new(frame);
        let (msg_type, decoded_payload) = decode_frame(&mut cursor).unwrap();

        assert_eq!(msg_type, MessageType::Text);
        assert_eq!(decoded_payload, payload);
    }

    #[test]
    fn test_frame_structure() {
        let payload = b"test";
        let frame = encode_frame(MessageType::Text, payload);

        // Type: 1 byte, Length: 4 bytes, Payload: 4 bytes = 9 total
        assert_eq!(frame.len(), 9);
        assert_eq!(frame[0], MessageType::Text as u8);

        // Length should be 4 (big-endian)
        let len = u32::from_be_bytes([frame[1], frame[2], frame[3], frame[4]]);
        assert_eq!(len, 4);
    }
}
