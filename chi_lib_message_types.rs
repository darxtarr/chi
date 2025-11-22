// Message type definitions for Chi protocol
// Apps emit these, Spirit interprets them

use std::io;

/// Text bubble - semantic text content with formatting
#[derive(Debug, Clone, PartialEq)]
pub struct TextMessage {
    pub content: String,
    pub color_rgba: u32,  // RGBA as packed u32
    // Future: font, size, style, etc.
}

impl TextMessage {
    pub fn new(content: String, color_rgba: u32) -> Self {
        Self { content, color_rgba }
    }

    /// Encode to binary payload
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Color (4 bytes)
        bytes.extend_from_slice(&self.color_rgba.to_be_bytes());

        // Text length (4 bytes)
        let text_len = self.content.len() as u32;
        bytes.extend_from_slice(&text_len.to_be_bytes());

        // Text (UTF-8 bytes)
        bytes.extend_from_slice(self.content.as_bytes());

        bytes
    }

    /// Decode from binary payload
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 8 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Payload too short"));
        }

        // Color (4 bytes)
        let color_rgba = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        // Text length (4 bytes)
        let text_len = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;

        if bytes.len() < 8 + text_len {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Incomplete text data"));
        }

        // Text (UTF-8)
        let content = String::from_utf8(bytes[8..8 + text_len].to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Self { content, color_rgba })
    }
}

/// Image bubble - bitmap content
#[derive(Debug, Clone)]
pub struct ImageMessage {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,  // Raw RGBA pixels
}

impl ImageMessage {
    pub fn new(width: u32, height: u32, rgba_data: Vec<u8>) -> Self {
        Self { width, height, rgba_data }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.width.to_be_bytes());
        bytes.extend_from_slice(&self.height.to_be_bytes());
        bytes.extend_from_slice(&self.rgba_data);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 8 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Payload too short"));
        }

        let width = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let height = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        let expected_data_len = (width * height * 4) as usize;
        if bytes.len() < 8 + expected_data_len {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Incomplete image data"));
        }

        let rgba_data = bytes[8..8 + expected_data_len].to_vec();

        Ok(Self { width, height, rgba_data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_message_roundtrip() {
        let original = TextMessage::new("Hello, World!".to_string(), 0x00FF00FF);
        let bytes = original.to_bytes();
        let decoded = TextMessage::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.content, original.content);
        assert_eq!(decoded.color_rgba, original.color_rgba);
    }

    #[test]
    fn test_image_message_roundtrip() {
        let rgba_data = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2 pixels
        let original = ImageMessage::new(2, 1, rgba_data.clone());
        let bytes = original.to_bytes();
        let decoded = ImageMessage::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.rgba_data, rgba_data);
    }
}
