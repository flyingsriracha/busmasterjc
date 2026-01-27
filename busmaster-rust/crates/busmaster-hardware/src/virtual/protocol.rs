//! Virtual CAN Bus Protocol
//!
//! Defines the message protocol for communication between virtual bus clients and server.

use busmaster_core::CanFrame;
use serde::{Deserialize, Serialize};

/// Message sent between virtual bus clients and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusMessage {
    /// Client connects to the bus
    Connect {
        /// Client identifier
        client_id: String,
    },

    /// Client disconnects from the bus
    Disconnect {
        /// Client identifier
        client_id: String,
    },

    /// CAN frame transmission
    Frame {
        /// The CAN frame
        frame: CanFrame,
        /// Channel number
        channel: u8,
        /// Timestamp in microseconds
        timestamp: u64,
    },

    /// Acknowledgment
    Ack,

    /// Error message
    Error {
        /// Error description
        message: String,
    },
}

impl BusMessage {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    /// Get message length prefix (4 bytes, big-endian)
    pub fn length_prefix(len: usize) -> [u8; 4] {
        (len as u32).to_be_bytes()
    }

    /// Parse length prefix
    pub fn parse_length_prefix(bytes: &[u8; 4]) -> usize {
        u32::from_be_bytes(*bytes) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = BusMessage::Connect {
            client_id: "test-client".to_string(),
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = BusMessage::from_bytes(&bytes).unwrap();

        match decoded {
            BusMessage::Connect { client_id } => {
                assert_eq!(client_id, "test-client");
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_frame_message() {
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        let msg = BusMessage::Frame {
            frame: frame.clone(),
            channel: 0,
            timestamp: 1234567890,
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = BusMessage::from_bytes(&bytes).unwrap();

        match decoded {
            BusMessage::Frame {
                frame: f,
                channel,
                timestamp,
            } => {
                assert_eq!(f.id(), frame.id());
                assert_eq!(f.data(), frame.data());
                assert_eq!(channel, 0);
                assert_eq!(timestamp, 1234567890);
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_length_prefix() {
        let len = 1234;
        let prefix = BusMessage::length_prefix(len);
        let parsed = BusMessage::parse_length_prefix(&prefix);
        assert_eq!(parsed, len);
    }
}
