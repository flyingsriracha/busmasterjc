//! J1939 Protocol Implementation
//!
//! J1939 is a higher-layer protocol built on top of CAN, commonly used in
//! heavy-duty vehicles, commercial trucks, and agricultural equipment.
//!
//! # Key Concepts
//!
//! ## Parameter Group Number (PGN)
//! J1939 uses PGNs instead of raw CAN IDs. A PGN identifies a specific
//! parameter group (set of related signals).
//!
//! ## 29-bit Extended ID Structure
//! ```text
//! | Priority (3) | Reserved (1) | Data Page (1) | PDU Format (8) | PDU Specific (8) | Source Address (8) |
//! |    28-26     |      25      |      24       |     23-16      |      15-8        |        7-0         |
//! ```
//!
//! ## PDU Format (PF)
//! - PF < 240: PDU1 format (peer-to-peer), PS = destination address
//! - PF >= 240: PDU2 format (broadcast), PS = group extension
//!
//! # Transport Protocol
//!
//! J1939 supports multi-frame messages up to 1785 bytes using:
//! - BAM (Broadcast Announce Message) for broadcast
//! - CMDT (Connection Mode Data Transfer) for peer-to-peer
//!
//! # Example
//!
//! ```
//! use busmaster_proto::j1939::{J1939Id, J1939Frame, PgnType};
//!
//! // Create a J1939 ID from components
//! let id = J1939Id::new(6, 0xFECA, 0x00);  // Priority 6, PGN 0xFECA, Source 0x00
//!
//! // Create a J1939 frame
//! let frame = J1939Frame::new(id, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
//!
//! // Check PGN type
//! assert_eq!(id.pgn_type(), PgnType::Broadcast);
//! ```

use busmaster_core::{BusmasterError, CanFrame, Result};
use serde::{Deserialize, Serialize};

/// Maximum J1939 data length (multi-frame)
pub const MAX_J1939_DATA_LEN: usize = 1785;

/// Default J1939 priority
pub const DEFAULT_PRIORITY: u8 = 6;

/// Null address (no address assigned)
pub const ADDRESS_NULL: u8 = 254;

/// Global/broadcast address
pub const ADDRESS_ALL: u8 = 255;

/// Transport Protocol - Connection Management PGN
pub const PGN_TP_CM: u32 = 0x00EC00;

/// Transport Protocol - Data Transfer PGN
pub const PGN_TP_DT: u32 = 0x00EB00;

/// Address Claimed PGN
pub const PGN_ADDRESS_CLAIMED: u32 = 0x00EE00;

/// Request PGN
pub const PGN_REQUEST: u32 = 0x00EA00;

/// Acknowledgment PGN
pub const PGN_ACKNOWLEDGMENT: u32 = 0x00E800;

/// Transport Protocol Control Bytes
pub mod tp_control {
    /// Request to Send
    pub const RTS: u8 = 0x10;
    /// Clear to Send
    pub const CTS: u8 = 0x11;
    /// End of Message Acknowledgment
    pub const EOM_ACK: u8 = 0x13;
    /// Broadcast Announce Message
    pub const BAM: u8 = 0x20;
    /// Connection Abort
    pub const CONN_ABORT: u8 = 0xFF;
}

/// J1939 message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum J1939MsgType {
    /// No specific type
    None,
    /// Command message
    Command,
    /// Request message
    Request,
    /// Data message
    Data,
    /// Broadcast message
    Broadcast,
    /// Acknowledgment message
    Acknowledgment,
    /// Group function message
    GroupFunction,
    /// Address claim
    AddressClaim,
    /// Request for address claim
    RequestAddressClaim,
    /// Commanded address
    CommandedAddress,
    /// Transport Protocol - BAM
    TpBam,
    /// Transport Protocol - RTS
    TpRts,
    /// Transport Protocol - CTS
    TpCts,
    /// Transport Protocol - End of Message ACK
    TpEomAck,
    /// Transport Protocol - Connection Abort
    TpConnAbort,
    /// Transport Protocol - Data Transfer
    TpDataTransfer,
}

impl Default for J1939MsgType {
    fn default() -> Self {
        Self::None
    }
}

/// PGN type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgnType {
    /// PDU1 format (PF < 240) - peer-to-peer with destination address
    PeerToPeer,
    /// PDU2 format (PF >= 240) - broadcast with group extension
    Broadcast,
}

/// J1939 29-bit identifier
///
/// Encodes priority, PGN, and source address in a 29-bit CAN extended ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct J1939Id {
    /// Raw 29-bit extended ID
    raw: u32,
}

impl J1939Id {
    /// Create a new J1939 ID from components
    ///
    /// # Arguments
    /// * `priority` - Message priority (0-7, lower = higher priority)
    /// * `pgn` - Parameter Group Number (18 bits)
    /// * `source_address` - Source node address (0-255)
    ///
    /// # Example
    /// ```
    /// use busmaster_proto::j1939::J1939Id;
    ///
    /// let id = J1939Id::new(6, 0xFECA, 0x00);
    /// assert_eq!(id.priority(), 6);
    /// assert_eq!(id.source_address(), 0x00);
    /// ```
    #[must_use]
    pub fn new(priority: u8, pgn: u32, source_address: u8) -> Self {
        let priority = (priority & 0x07) as u32;
        let pgn = pgn & 0x3FFFF; // 18 bits
        let source = source_address as u32;

        let raw = (priority << 26) | (pgn << 8) | source;
        Self { raw }
    }

    /// Create a J1939 ID for peer-to-peer communication
    ///
    /// # Arguments
    /// * `priority` - Message priority (0-7)
    /// * `pdu_format` - PDU Format (must be < 240 for peer-to-peer)
    /// * `destination` - Destination address
    /// * `source_address` - Source node address
    #[must_use]
    pub fn new_peer_to_peer(
        priority: u8,
        pdu_format: u8,
        destination: u8,
        source_address: u8,
    ) -> Self {
        let pgn = ((pdu_format as u32) << 8) | (destination as u32);
        Self::new(priority, pgn, source_address)
    }

    /// Create a J1939 ID from a raw 29-bit extended CAN ID
    #[must_use]
    pub fn from_raw(raw: u32) -> Self {
        Self {
            raw: raw & 0x1FFF_FFFF,
        }
    }

    /// Get the raw 29-bit extended ID
    #[must_use]
    pub fn raw(&self) -> u32 {
        self.raw
    }

    /// Get the priority (0-7, lower = higher priority)
    #[must_use]
    pub fn priority(&self) -> u8 {
        ((self.raw >> 26) & 0x07) as u8
    }

    /// Get the reserved bit
    #[must_use]
    pub fn reserved(&self) -> bool {
        (self.raw >> 25) & 0x01 != 0
    }

    /// Get the data page bit
    #[must_use]
    pub fn data_page(&self) -> bool {
        (self.raw >> 24) & 0x01 != 0
    }

    /// Get the PDU Format (PF)
    #[must_use]
    pub fn pdu_format(&self) -> u8 {
        ((self.raw >> 16) & 0xFF) as u8
    }

    /// Get the PDU Specific (PS)
    ///
    /// For PDU1 (PF < 240): This is the destination address
    /// For PDU2 (PF >= 240): This is the group extension
    #[must_use]
    pub fn pdu_specific(&self) -> u8 {
        ((self.raw >> 8) & 0xFF) as u8
    }

    /// Get the source address
    #[must_use]
    pub fn source_address(&self) -> u8 {
        (self.raw & 0xFF) as u8
    }

    /// Get the Parameter Group Number (PGN)
    ///
    /// For PDU1 format (PF < 240), the PS field is NOT part of the PGN.
    /// For PDU2 format (PF >= 240), the PS field IS part of the PGN.
    #[must_use]
    pub fn pgn(&self) -> u32 {
        let pf = self.pdu_format();
        let dp = if self.data_page() { 1u32 } else { 0 };
        let r = if self.reserved() { 1u32 } else { 0 };

        if pf < 240 {
            // PDU1: PGN = R.DP.PF.00
            (r << 17) | (dp << 16) | ((pf as u32) << 8)
        } else {
            // PDU2: PGN = R.DP.PF.PS
            (r << 17) | (dp << 16) | ((pf as u32) << 8) | (self.pdu_specific() as u32)
        }
    }

    /// Get the destination address (only valid for PDU1 format)
    ///
    /// Returns `None` for PDU2 (broadcast) format.
    #[must_use]
    pub fn destination_address(&self) -> Option<u8> {
        if self.pdu_format() < 240 {
            Some(self.pdu_specific())
        } else {
            None
        }
    }

    /// Get the PGN type (peer-to-peer or broadcast)
    #[must_use]
    pub fn pgn_type(&self) -> PgnType {
        if self.pdu_format() < 240 {
            PgnType::PeerToPeer
        } else {
            PgnType::Broadcast
        }
    }

    /// Check if this is a broadcast message
    #[must_use]
    pub fn is_broadcast(&self) -> bool {
        self.pgn_type() == PgnType::Broadcast
    }

    /// Set the priority
    pub fn set_priority(&mut self, priority: u8) {
        let priority = (priority & 0x07) as u32;
        self.raw = (self.raw & 0x03FF_FFFF) | (priority << 26);
    }

    /// Set the source address
    pub fn set_source_address(&mut self, address: u8) {
        self.raw = (self.raw & 0x1FFF_FF00) | (address as u32);
    }
}

impl From<u32> for J1939Id {
    fn from(raw: u32) -> Self {
        Self::from_raw(raw)
    }
}

impl From<J1939Id> for u32 {
    fn from(id: J1939Id) -> Self {
        id.raw
    }
}

/// J1939 frame (single CAN frame)
///
/// Represents a single J1939 message that fits in one CAN frame (up to 8 bytes).
/// For multi-frame messages, use the transport protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct J1939Frame {
    /// J1939 identifier
    id: J1939Id,
    /// Data bytes (up to 8 for single frame)
    data: Vec<u8>,
    /// Timestamp in microseconds
    timestamp_us: u64,
    /// Channel number
    channel: u8,
    /// Message type
    msg_type: J1939MsgType,
}

impl J1939Frame {
    /// Create a new J1939 frame
    ///
    /// # Arguments
    /// * `id` - J1939 identifier
    /// * `data` - Data bytes (up to 8 bytes)
    ///
    /// # Panics
    /// Panics if data length exceeds 8 bytes.
    #[must_use]
    pub fn new(id: J1939Id, data: &[u8]) -> Self {
        assert!(data.len() <= 8, "J1939 single frame max 8 bytes");
        Self {
            id,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
            msg_type: J1939MsgType::Data,
        }
    }

    /// Get the J1939 ID
    #[must_use]
    pub fn id(&self) -> J1939Id {
        self.id
    }

    /// Get the PGN
    #[must_use]
    pub fn pgn(&self) -> u32 {
        self.id.pgn()
    }

    /// Get the source address
    #[must_use]
    pub fn source_address(&self) -> u8 {
        self.id.source_address()
    }

    /// Get the destination address (if peer-to-peer)
    #[must_use]
    pub fn destination_address(&self) -> Option<u8> {
        self.id.destination_address()
    }

    /// Get the priority
    #[must_use]
    pub fn priority(&self) -> u8 {
        self.id.priority()
    }

    /// Get the data bytes
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the data length
    #[must_use]
    pub fn dlc(&self) -> u8 {
        self.data.len() as u8
    }

    /// Get the timestamp in microseconds
    #[must_use]
    pub fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }

    /// Set the timestamp
    pub fn set_timestamp(&mut self, timestamp_us: u64) {
        self.timestamp_us = timestamp_us;
    }

    /// Get the channel number
    #[must_use]
    pub fn channel(&self) -> u8 {
        self.channel
    }

    /// Set the channel number
    pub fn set_channel(&mut self, channel: u8) {
        self.channel = channel;
    }

    /// Get the message type
    #[must_use]
    pub fn msg_type(&self) -> J1939MsgType {
        self.msg_type
    }

    /// Set the message type
    pub fn set_msg_type(&mut self, msg_type: J1939MsgType) {
        self.msg_type = msg_type;
    }

    /// Convert to a CAN frame
    #[must_use]
    pub fn to_can_frame(&self) -> CanFrame {
        let mut frame = CanFrame::new_extended(self.id.raw(), &self.data)
            .expect("J1939 frame should always be valid CAN frame");
        frame.set_timestamp(self.timestamp_us);
        frame.set_channel(self.channel);
        frame
    }

    /// Create from a CAN frame
    ///
    /// # Errors
    /// Returns error if the CAN frame is not an extended frame.
    pub fn from_can_frame(frame: &CanFrame) -> Result<Self> {
        if !frame.is_extended() {
            return Err(BusmasterError::Parse {
                message: "J1939 requires extended CAN ID".into(),
            });
        }

        let id = J1939Id::from_raw(frame.id());
        let msg_type = detect_msg_type(&id, frame.data());

        Ok(Self {
            id,
            data: frame.data().to_vec(),
            timestamp_us: frame.timestamp_us(),
            channel: frame.channel(),
            msg_type,
        })
    }
}

/// Detect J1939 message type from ID and data
fn detect_msg_type(id: &J1939Id, data: &[u8]) -> J1939MsgType {
    let pgn = id.pgn();

    match pgn {
        PGN_ADDRESS_CLAIMED => J1939MsgType::AddressClaim,
        PGN_REQUEST => J1939MsgType::Request,
        PGN_ACKNOWLEDGMENT => J1939MsgType::Acknowledgment,
        PGN_TP_CM => {
            // Transport Protocol - Connection Management
            if data.is_empty() {
                return J1939MsgType::None;
            }
            match data[0] {
                tp_control::BAM => J1939MsgType::TpBam,
                tp_control::RTS => J1939MsgType::TpRts,
                tp_control::CTS => J1939MsgType::TpCts,
                tp_control::EOM_ACK => J1939MsgType::TpEomAck,
                tp_control::CONN_ABORT => J1939MsgType::TpConnAbort,
                _ => J1939MsgType::None,
            }
        },
        PGN_TP_DT => J1939MsgType::TpDataTransfer,
        _ => {
            if id.is_broadcast() {
                J1939MsgType::Broadcast
            } else {
                J1939MsgType::Data
            }
        },
    }
}

/// ECU Name (64-bit NAME field per J1939-81)
///
/// The NAME uniquely identifies an ECU on the J1939 network.
/// It's used during address claiming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EcuName {
    /// Raw 64-bit NAME value
    raw: u64,
}

impl EcuName {
    /// Create an ECU name from raw 64-bit value
    #[must_use]
    pub fn from_raw(raw: u64) -> Self {
        Self { raw }
    }

    /// Create an ECU name from components
    ///
    /// # Arguments
    /// * `identity_number` - 21-bit identity number
    /// * `manufacturer_code` - 11-bit SAE manufacturer code
    /// * `ecu_instance` - 3-bit ECU instance
    /// * `function_instance` - 5-bit function instance
    /// * `function` - 8-bit SAE function
    /// * `vehicle_system` - 7-bit SAE vehicle system
    /// * `vehicle_system_instance` - 4-bit vehicle system instance
    /// * `industry_group` - 3-bit SAE industry group
    /// * `arbitrary_address_capable` - 1-bit arbitrary address capable flag
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity_number: u32,
        manufacturer_code: u16,
        ecu_instance: u8,
        function_instance: u8,
        function: u8,
        vehicle_system: u8,
        vehicle_system_instance: u8,
        industry_group: u8,
        arbitrary_address_capable: bool,
    ) -> Self {
        let mut raw: u64 = 0;

        raw |= (identity_number as u64) & 0x1F_FFFF;
        raw |= ((manufacturer_code as u64) & 0x7FF) << 21;
        raw |= ((ecu_instance as u64) & 0x07) << 32;
        raw |= ((function_instance as u64) & 0x1F) << 35;
        raw |= (function as u64) << 40;
        // Reserved bit at 48
        raw |= ((vehicle_system as u64) & 0x7F) << 49;
        raw |= ((vehicle_system_instance as u64) & 0x0F) << 56;
        raw |= ((industry_group as u64) & 0x07) << 60;
        raw |= if arbitrary_address_capable {
            1u64 << 63
        } else {
            0
        };

        Self { raw }
    }

    /// Get the raw 64-bit value
    #[must_use]
    pub fn raw(&self) -> u64 {
        self.raw
    }

    /// Get the identity number (21 bits)
    #[must_use]
    pub fn identity_number(&self) -> u32 {
        (self.raw & 0x1F_FFFF) as u32
    }

    /// Get the manufacturer code (11 bits)
    #[must_use]
    pub fn manufacturer_code(&self) -> u16 {
        ((self.raw >> 21) & 0x7FF) as u16
    }

    /// Get the ECU instance (3 bits)
    #[must_use]
    pub fn ecu_instance(&self) -> u8 {
        ((self.raw >> 32) & 0x07) as u8
    }

    /// Get the function instance (5 bits)
    #[must_use]
    pub fn function_instance(&self) -> u8 {
        ((self.raw >> 35) & 0x1F) as u8
    }

    /// Get the function (8 bits)
    #[must_use]
    pub fn function(&self) -> u8 {
        ((self.raw >> 40) & 0xFF) as u8
    }

    /// Get the vehicle system (7 bits)
    #[must_use]
    pub fn vehicle_system(&self) -> u8 {
        ((self.raw >> 49) & 0x7F) as u8
    }

    /// Get the vehicle system instance (4 bits)
    #[must_use]
    pub fn vehicle_system_instance(&self) -> u8 {
        ((self.raw >> 56) & 0x0F) as u8
    }

    /// Get the industry group (3 bits)
    #[must_use]
    pub fn industry_group(&self) -> u8 {
        ((self.raw >> 60) & 0x07) as u8
    }

    /// Check if arbitrary address capable
    #[must_use]
    pub fn is_arbitrary_address_capable(&self) -> bool {
        (self.raw >> 63) & 0x01 != 0
    }

    /// Convert to bytes (little-endian)
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 8] {
        self.raw.to_le_bytes()
    }

    /// Create from bytes (little-endian)
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        Self {
            raw: u64::from_le_bytes(*bytes),
        }
    }
}

impl From<u64> for EcuName {
    fn from(raw: u64) -> Self {
        Self::from_raw(raw)
    }
}

impl From<EcuName> for u64 {
    fn from(name: EcuName) -> Self {
        name.raw
    }
}

/// J1939 Transport Protocol state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TpState {
    /// Idle, no transfer in progress
    Idle,
    /// Waiting for CTS (after sending RTS)
    WaitingForCts,
    /// Sending data packets
    SendingData,
    /// Receiving data packets
    ReceivingData,
    /// Transfer complete
    Complete,
    /// Transfer aborted
    Aborted,
}

/// Transport Protocol session for multi-frame messages
#[derive(Debug, Clone)]
pub struct TpSession {
    /// PGN being transferred
    pgn: u32,
    /// Source address
    source: u8,
    /// Destination address (255 for broadcast)
    destination: u8,
    /// Total message size in bytes
    total_size: u16,
    /// Number of packets expected
    expected_packets: u8,
    /// Current packet number (1-based)
    current_packet: u8,
    /// Accumulated data
    data: Vec<u8>,
    /// Session state
    state: TpState,
    /// Is this a BAM (broadcast) session?
    is_bam: bool,
}

impl TpSession {
    /// Create a new BAM (broadcast) session from a BAM message
    ///
    /// # Arguments
    /// * `source` - Source address
    /// * `data` - BAM message data (8 bytes)
    ///
    /// # Returns
    /// `None` if the data is not a valid BAM message
    #[must_use]
    pub fn from_bam(source: u8, data: &[u8]) -> Option<Self> {
        if data.len() < 8 || data[0] != tp_control::BAM {
            return None;
        }

        let total_size = u16::from_le_bytes([data[1], data[2]]);
        let num_packets = data[3];
        let pgn = u32::from_le_bytes([data[5], data[6], data[7], 0]);

        Some(Self {
            pgn,
            source,
            destination: ADDRESS_ALL,
            total_size,
            expected_packets: num_packets,
            current_packet: 0,
            data: Vec::with_capacity(total_size as usize),
            state: TpState::ReceivingData,
            is_bam: true,
        })
    }

    /// Create a new RTS (peer-to-peer) session from an RTS message
    ///
    /// # Arguments
    /// * `source` - Source address
    /// * `destination` - Destination address
    /// * `data` - RTS message data (8 bytes)
    ///
    /// # Returns
    /// `None` if the data is not a valid RTS message
    #[must_use]
    pub fn from_rts(source: u8, destination: u8, data: &[u8]) -> Option<Self> {
        if data.len() < 8 || data[0] != tp_control::RTS {
            return None;
        }

        let total_size = u16::from_le_bytes([data[1], data[2]]);
        let num_packets = data[3];
        let pgn = u32::from_le_bytes([data[5], data[6], data[7], 0]);

        Some(Self {
            pgn,
            source,
            destination,
            total_size,
            expected_packets: num_packets,
            current_packet: 0,
            data: Vec::with_capacity(total_size as usize),
            state: TpState::ReceivingData,
            is_bam: false,
        })
    }

    /// Process a data transfer packet
    ///
    /// # Arguments
    /// * `sequence_number` - Packet sequence number (1-based)
    /// * `data` - Packet data (7 bytes of payload)
    ///
    /// # Returns
    /// `true` if the transfer is complete
    pub fn process_data_packet(&mut self, sequence_number: u8, data: &[u8]) -> bool {
        if self.state != TpState::ReceivingData {
            return false;
        }

        // Sequence numbers are 1-based
        if sequence_number != self.current_packet + 1 {
            // Out of sequence - abort
            self.state = TpState::Aborted;
            return false;
        }

        self.current_packet = sequence_number;

        // Data packets have 7 bytes of payload (first byte is sequence number)
        let payload = if data.len() > 1 { &data[1..] } else { &[] };

        // Calculate how many bytes to take from this packet
        let remaining = self.total_size as usize - self.data.len();
        let to_take = remaining.min(payload.len());

        self.data.extend_from_slice(&payload[..to_take]);

        // Check if complete
        if self.data.len() >= self.total_size as usize {
            self.state = TpState::Complete;
            return true;
        }

        false
    }

    /// Get the PGN being transferred
    #[must_use]
    pub fn pgn(&self) -> u32 {
        self.pgn
    }

    /// Get the source address
    #[must_use]
    pub fn source(&self) -> u8 {
        self.source
    }

    /// Get the destination address
    #[must_use]
    pub fn destination(&self) -> u8 {
        self.destination
    }

    /// Get the total message size
    #[must_use]
    pub fn total_size(&self) -> u16 {
        self.total_size
    }

    /// Get the current state
    #[must_use]
    pub fn state(&self) -> &TpState {
        &self.state
    }

    /// Get the accumulated data
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Check if this is a BAM session
    #[must_use]
    pub fn is_bam(&self) -> bool {
        self.is_bam
    }

    /// Get the expected number of packets
    #[must_use]
    pub fn expected_packets(&self) -> u8 {
        self.expected_packets
    }

    /// Check if the transfer is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.state == TpState::Complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_j1939_id_new() {
        let id = J1939Id::new(6, 0xFECA, 0x00);
        assert_eq!(id.priority(), 6);
        assert_eq!(id.pgn(), 0xFECA);
        assert_eq!(id.source_address(), 0x00);
    }

    #[test]
    fn test_j1939_id_peer_to_peer() {
        // PDU1 format: PF < 240
        let id = J1939Id::new_peer_to_peer(6, 0xEA, 0x00, 0x21);
        assert_eq!(id.priority(), 6);
        assert_eq!(id.pdu_format(), 0xEA);
        assert_eq!(id.pdu_specific(), 0x00);
        assert_eq!(id.source_address(), 0x21);
        assert_eq!(id.pgn_type(), PgnType::PeerToPeer);
        assert_eq!(id.destination_address(), Some(0x00));
        // PGN for PDU1 doesn't include PS
        assert_eq!(id.pgn(), 0xEA00);
    }

    #[test]
    fn test_j1939_id_broadcast() {
        // PDU2 format: PF >= 240
        let id = J1939Id::new(6, 0xFECA, 0x00);
        assert_eq!(id.pdu_format(), 0xFE);
        assert_eq!(id.pdu_specific(), 0xCA);
        assert_eq!(id.pgn_type(), PgnType::Broadcast);
        assert!(id.is_broadcast());
        assert_eq!(id.destination_address(), None);
        assert_eq!(id.pgn(), 0xFECA);
    }

    #[test]
    fn test_j1939_id_from_raw() {
        // Example: Priority 6, PGN 0xFECA, Source 0x21
        // Raw: 0x18FECA21
        let id = J1939Id::from_raw(0x18FE_CA21);
        assert_eq!(id.priority(), 6);
        assert_eq!(id.pgn(), 0xFECA);
        assert_eq!(id.source_address(), 0x21);
    }

    #[test]
    fn test_j1939_frame_creation() {
        let id = J1939Id::new(6, 0xFECA, 0x00);
        let frame = J1939Frame::new(id, &[0x01, 0x02, 0x03, 0x04]);

        assert_eq!(frame.pgn(), 0xFECA);
        assert_eq!(frame.source_address(), 0x00);
        assert_eq!(frame.priority(), 6);
        assert_eq!(frame.data(), &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(frame.dlc(), 4);
    }

    #[test]
    fn test_j1939_frame_to_can() {
        let id = J1939Id::new(6, 0xFECA, 0x00);
        let mut frame = J1939Frame::new(id, &[0x01, 0x02, 0x03, 0x04]);
        frame.set_timestamp(12345);
        frame.set_channel(1);

        let can_frame = frame.to_can_frame();
        assert!(can_frame.is_extended());
        assert_eq!(can_frame.id(), id.raw());
        assert_eq!(can_frame.data(), &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(can_frame.timestamp_us(), 12345);
        assert_eq!(can_frame.channel(), 1);
    }

    #[test]
    fn test_j1939_frame_from_can() {
        let can_frame = CanFrame::new_extended(0x18FE_CA21, &[0x01, 0x02, 0x03]).unwrap();
        let j1939_frame = J1939Frame::from_can_frame(&can_frame).unwrap();

        assert_eq!(j1939_frame.pgn(), 0xFECA);
        assert_eq!(j1939_frame.source_address(), 0x21);
        assert_eq!(j1939_frame.data(), &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_j1939_frame_from_standard_can_fails() {
        let can_frame = CanFrame::new_standard(0x123, &[0x01, 0x02]).unwrap();
        let result = J1939Frame::from_can_frame(&can_frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_msg_type_address_claim() {
        let id = J1939Id::new(6, PGN_ADDRESS_CLAIMED, 0x00);
        let msg_type = detect_msg_type(&id, &[0; 8]);
        assert_eq!(msg_type, J1939MsgType::AddressClaim);
    }

    #[test]
    fn test_detect_msg_type_tp_bam() {
        let id = J1939Id::new(6, PGN_TP_CM, 0x00);
        let data = [tp_control::BAM, 0, 0, 0, 0, 0, 0, 0];
        let msg_type = detect_msg_type(&id, &data);
        assert_eq!(msg_type, J1939MsgType::TpBam);
    }

    #[test]
    fn test_detect_msg_type_tp_rts() {
        let id = J1939Id::new(6, PGN_TP_CM, 0x00);
        let data = [tp_control::RTS, 0, 0, 0, 0, 0, 0, 0];
        let msg_type = detect_msg_type(&id, &data);
        assert_eq!(msg_type, J1939MsgType::TpRts);
    }

    #[test]
    fn test_ecu_name_creation() {
        let name = EcuName::new(
            0x12345, // identity_number
            0x123,   // manufacturer_code
            1,       // ecu_instance
            2,       // function_instance
            0x80,    // function
            0x40,    // vehicle_system
            3,       // vehicle_system_instance
            2,       // industry_group
            true,    // arbitrary_address_capable
        );

        assert_eq!(name.identity_number(), 0x12345);
        assert_eq!(name.manufacturer_code(), 0x123);
        assert_eq!(name.ecu_instance(), 1);
        assert_eq!(name.function_instance(), 2);
        assert_eq!(name.function(), 0x80);
        assert_eq!(name.vehicle_system(), 0x40);
        assert_eq!(name.vehicle_system_instance(), 3);
        assert_eq!(name.industry_group(), 2);
        assert!(name.is_arbitrary_address_capable());
    }

    #[test]
    fn test_ecu_name_bytes_roundtrip() {
        let name = EcuName::from_raw(0x8000_0000_0000_0001);
        let bytes = name.to_bytes();
        let name2 = EcuName::from_bytes(&bytes);
        assert_eq!(name.raw(), name2.raw());
    }

    #[test]
    fn test_tp_session_bam() {
        // BAM message: control byte, total size (16 bytes), num packets (3), reserved, PGN
        let bam_data = [
            tp_control::BAM,
            16,
            0,    // total size = 16
            3,    // num packets = 3
            0xFF, // reserved
            0xCA,
            0xFE,
            0x00, // PGN = 0xFECA
        ];

        let session = TpSession::from_bam(0x21, &bam_data).unwrap();
        assert_eq!(session.pgn(), 0x00FE_CA);
        assert_eq!(session.source(), 0x21);
        assert_eq!(session.destination(), ADDRESS_ALL);
        assert_eq!(session.total_size(), 16);
        assert!(session.is_bam());
    }

    #[test]
    fn test_tp_session_data_packets() {
        let bam_data = [
            tp_control::BAM,
            16,
            0, // total size = 16
            3, // num packets = 3
            0xFF,
            0xCA,
            0xFE,
            0x00,
        ];

        let mut session = TpSession::from_bam(0x21, &bam_data).unwrap();

        // First packet (7 bytes of data)
        let packet1 = [1, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        assert!(!session.process_data_packet(1, &packet1));
        assert_eq!(session.data().len(), 7);

        // Second packet
        let packet2 = [2, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E];
        assert!(!session.process_data_packet(2, &packet2));
        assert_eq!(session.data().len(), 14);

        // Third packet (only 2 bytes needed)
        let packet3 = [3, 0x0F, 0x10, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(session.process_data_packet(3, &packet3));
        assert_eq!(session.data().len(), 16);
        assert!(session.is_complete());
    }
}

/// Address claim state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressClaimState {
    /// No address claimed
    Unclaimed,
    /// Address claim in progress
    Claiming,
    /// Address successfully claimed
    Claimed,
    /// Address claim failed (contention lost)
    Failed,
}

/// J1939 Address Claim Manager
///
/// Manages the address claiming process per J1939-81.
/// Each ECU must claim an address before participating on the network.
#[derive(Debug, Clone)]
pub struct AddressClaimManager {
    /// ECU NAME
    ecu_name: EcuName,
    /// Preferred address
    preferred_address: u8,
    /// Current address (if claimed)
    current_address: Option<u8>,
    /// Claim state
    state: AddressClaimState,
}

impl AddressClaimManager {
    /// Create a new address claim manager
    ///
    /// # Arguments
    /// * `ecu_name` - The ECU's 64-bit NAME
    /// * `preferred_address` - The preferred address to claim (0-253)
    #[must_use]
    pub fn new(ecu_name: EcuName, preferred_address: u8) -> Self {
        Self {
            ecu_name,
            preferred_address,
            current_address: None,
            state: AddressClaimState::Unclaimed,
        }
    }

    /// Get the ECU NAME
    #[must_use]
    pub fn ecu_name(&self) -> EcuName {
        self.ecu_name
    }

    /// Get the preferred address
    #[must_use]
    pub fn preferred_address(&self) -> u8 {
        self.preferred_address
    }

    /// Get the current claimed address
    #[must_use]
    pub fn current_address(&self) -> Option<u8> {
        self.current_address
    }

    /// Get the claim state
    #[must_use]
    pub fn state(&self) -> AddressClaimState {
        self.state
    }

    /// Create an address claim message
    ///
    /// Returns a J1939 frame containing the address claim.
    #[must_use]
    pub fn create_claim_message(&self) -> J1939Frame {
        let address = self.current_address.unwrap_or(self.preferred_address);
        let id = J1939Id::new(DEFAULT_PRIORITY, PGN_ADDRESS_CLAIMED, address);
        let mut frame = J1939Frame::new(id, &self.ecu_name.to_bytes());
        frame.set_msg_type(J1939MsgType::AddressClaim);
        frame
    }

    /// Create a request for address claim message
    ///
    /// This requests all nodes to send their address claims.
    #[must_use]
    pub fn create_request_address_claim() -> J1939Frame {
        // Request PGN with destination = global (255)
        let id = J1939Id::new_peer_to_peer(DEFAULT_PRIORITY, 0xEA, ADDRESS_ALL, ADDRESS_NULL);
        // Request for PGN 0x00EE00 (Address Claimed)
        let pgn_bytes = PGN_ADDRESS_CLAIMED.to_le_bytes();
        let mut frame = J1939Frame::new(id, &pgn_bytes[..3]);
        frame.set_msg_type(J1939MsgType::RequestAddressClaim);
        frame
    }

    /// Start the address claim process
    pub fn start_claim(&mut self) {
        self.state = AddressClaimState::Claiming;
    }

    /// Process a received address claim message
    ///
    /// # Arguments
    /// * `source_address` - The address being claimed
    /// * `name` - The ECU NAME of the claimer
    ///
    /// # Returns
    /// `true` if we need to send our own claim (contention), `false` otherwise
    pub fn process_address_claim(&mut self, source_address: u8, name: EcuName) -> bool {
        let our_address = self.current_address.unwrap_or(self.preferred_address);

        // If someone else is claiming our address
        if source_address == our_address {
            // Compare NAMEs - lower NAME wins
            if name.raw() < self.ecu_name.raw() {
                // We lose - need to find a new address or give up
                if self.ecu_name.is_arbitrary_address_capable() {
                    // Try to find another address
                    self.state = AddressClaimState::Claiming;
                    // In a real implementation, we'd search for an available address
                    self.current_address = None;
                } else {
                    // Cannot claim another address
                    self.state = AddressClaimState::Failed;
                    self.current_address = None;
                }
                return false;
            } else {
                // We win - send our claim again
                return true;
            }
        }

        false
    }

    /// Mark the address as successfully claimed
    pub fn claim_successful(&mut self) {
        self.current_address = Some(self.preferred_address);
        self.state = AddressClaimState::Claimed;
    }

    /// Check if we have a valid claimed address
    #[must_use]
    pub fn is_claimed(&self) -> bool {
        self.state == AddressClaimState::Claimed && self.current_address.is_some()
    }
}

#[cfg(test)]
mod address_claim_tests {
    use super::*;

    #[test]
    fn test_address_claim_manager_creation() {
        let name = EcuName::from_raw(0x8000_0000_0000_0001);
        let manager = AddressClaimManager::new(name, 0x21);

        assert_eq!(manager.preferred_address(), 0x21);
        assert_eq!(manager.current_address(), None);
        assert_eq!(manager.state(), AddressClaimState::Unclaimed);
    }

    #[test]
    fn test_create_claim_message() {
        let name = EcuName::from_raw(0x8000_0000_0000_0001);
        let manager = AddressClaimManager::new(name, 0x21);

        let frame = manager.create_claim_message();
        assert_eq!(frame.pgn(), PGN_ADDRESS_CLAIMED);
        assert_eq!(frame.source_address(), 0x21);
        assert_eq!(frame.data().len(), 8);
        assert_eq!(frame.msg_type(), J1939MsgType::AddressClaim);
    }

    #[test]
    fn test_claim_successful() {
        let name = EcuName::from_raw(0x8000_0000_0000_0001);
        let mut manager = AddressClaimManager::new(name, 0x21);

        manager.start_claim();
        assert_eq!(manager.state(), AddressClaimState::Claiming);

        manager.claim_successful();
        assert_eq!(manager.state(), AddressClaimState::Claimed);
        assert_eq!(manager.current_address(), Some(0x21));
        assert!(manager.is_claimed());
    }

    #[test]
    fn test_address_contention_we_win() {
        let our_name = EcuName::from_raw(0x0000_0000_0000_0001); // Lower = higher priority
        let their_name = EcuName::from_raw(0x8000_0000_0000_0002);

        let mut manager = AddressClaimManager::new(our_name, 0x21);
        manager.start_claim();

        // They claim our address but we have lower NAME
        let need_reclaim = manager.process_address_claim(0x21, their_name);
        assert!(need_reclaim); // We should reclaim
    }

    #[test]
    fn test_address_contention_we_lose() {
        let our_name = EcuName::from_raw(0x8000_0000_0000_0002); // Higher = lower priority
        let their_name = EcuName::from_raw(0x0000_0000_0000_0001);

        let mut manager = AddressClaimManager::new(our_name, 0x21);
        manager.start_claim();

        // They claim our address and have lower NAME
        let need_reclaim = manager.process_address_claim(0x21, their_name);
        assert!(!need_reclaim);

        // Since we're arbitrary address capable, we should be in claiming state
        // looking for a new address
        assert_eq!(manager.state(), AddressClaimState::Claiming);
    }

    #[test]
    fn test_request_address_claim() {
        let frame = AddressClaimManager::create_request_address_claim();
        assert_eq!(frame.pgn(), PGN_REQUEST);
        assert_eq!(frame.msg_type(), J1939MsgType::RequestAddressClaim);
    }
}
