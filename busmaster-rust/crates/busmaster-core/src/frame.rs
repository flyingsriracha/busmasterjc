//! CAN frame types
//!
//! This module provides the core CAN frame types used throughout BUSMASTER:
//! - [`CanFrame`] - Standard CAN 2.0 frames (up to 8 bytes)
//! - [`CanFdFrame`] - CAN FD frames (up to 64 bytes)
//! - [`CanXlFrame`] - CAN XL frames (up to 2048 bytes)
//!
//! # Example
//!
//! ```
//! use busmaster_core::CanFrame;
//!
//! // Create a standard CAN frame
//! let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03, 0x04]).unwrap();
//! assert_eq!(frame.id(), 0x123);
//! assert_eq!(frame.dlc(), 4);
//!
//! // Create an extended CAN frame
//! let ext_frame = CanFrame::new_extended(0x12345678, &[0xAA, 0xBB]).unwrap();
//! assert!(ext_frame.is_extended());
//! ```

use crate::{BusmasterError, Result};
use serde::{Deserialize, Serialize};

/// Maximum standard CAN ID (11-bit)
pub const MAX_STANDARD_ID: u32 = 0x7FF;

/// Maximum extended CAN ID (29-bit)
pub const MAX_EXTENDED_ID: u32 = 0x1FFF_FFFF;

/// Maximum CAN 2.0 data length
pub const MAX_CAN_DATA_LEN: usize = 8;

/// Maximum CAN FD data length
pub const MAX_CANFD_DATA_LEN: usize = 64;

/// Maximum CAN XL data length
pub const MAX_CANXL_DATA_LEN: usize = 2048;

/// CAN 2.0 frame
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanFrame {
    /// CAN identifier
    id: u32,
    /// Extended ID flag
    extended: bool,
    /// Remote transmission request flag
    rtr: bool,
    /// Data bytes
    data: Vec<u8>,
    /// Timestamp in microseconds
    timestamp_us: u64,
    /// Channel number
    channel: u8,
}

impl CanFrame {
    /// Create a new standard CAN frame
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ID exceeds the maximum standard ID (0x7FF)
    /// - The data length exceeds 8 bytes
    pub fn new_standard(id: u32, data: &[u8]) -> Result<Self> {
        if id > MAX_STANDARD_ID {
            return Err(BusmasterError::InvalidCanId {
                id,
                max: MAX_STANDARD_ID,
            });
        }
        if data.len() > MAX_CAN_DATA_LEN {
            return Err(BusmasterError::InvalidDataLength {
                len: data.len(),
                max: MAX_CAN_DATA_LEN,
            });
        }
        Ok(Self {
            id,
            extended: false,
            rtr: false,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
        })
    }

    /// Create a new extended CAN frame
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ID exceeds the maximum extended ID (`0x1FFF_FFFF`)
    /// - The data length exceeds 8 bytes
    pub fn new_extended(id: u32, data: &[u8]) -> Result<Self> {
        if id > MAX_EXTENDED_ID {
            return Err(BusmasterError::InvalidCanId {
                id,
                max: MAX_EXTENDED_ID,
            });
        }
        if data.len() > MAX_CAN_DATA_LEN {
            return Err(BusmasterError::InvalidDataLength {
                len: data.len(),
                max: MAX_CAN_DATA_LEN,
            });
        }
        Ok(Self {
            id,
            extended: true,
            rtr: false,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
        })
    }

    /// Get the CAN ID
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Check if this is an extended frame
    #[must_use]
    pub fn is_extended(&self) -> bool {
        self.extended
    }

    /// Check if this is a remote transmission request
    #[must_use]
    pub fn is_rtr(&self) -> bool {
        self.rtr
    }

    /// Get the data bytes
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the data length code
    #[must_use]
    pub fn dlc(&self) -> u8 {
        #[allow(clippy::cast_possible_truncation)]
        {
            self.data.len() as u8
        }
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
}

/// CAN FD frame
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanFdFrame {
    /// CAN identifier
    id: u32,
    /// Extended ID flag
    extended: bool,
    /// Bit rate switch flag
    brs: bool,
    /// Error state indicator
    esi: bool,
    /// Data bytes
    data: Vec<u8>,
    /// Timestamp in microseconds
    timestamp_us: u64,
    /// Channel number
    channel: u8,
}

impl CanFdFrame {
    /// Create a new CAN FD frame
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ID exceeds the maximum for the frame type
    /// - The data length exceeds 64 bytes
    pub fn new(id: u32, extended: bool, data: &[u8]) -> Result<Self> {
        let max_id = if extended {
            MAX_EXTENDED_ID
        } else {
            MAX_STANDARD_ID
        };
        if id > max_id {
            return Err(BusmasterError::InvalidCanId { id, max: max_id });
        }
        if data.len() > MAX_CANFD_DATA_LEN {
            return Err(BusmasterError::InvalidDataLength {
                len: data.len(),
                max: MAX_CANFD_DATA_LEN,
            });
        }
        Ok(Self {
            id,
            extended,
            brs: false,
            esi: false,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
        })
    }

    /// Get the CAN ID
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Check if this is an extended frame
    #[must_use]
    pub fn is_extended(&self) -> bool {
        self.extended
    }

    /// Check if bit rate switch is enabled
    #[must_use]
    pub fn is_brs(&self) -> bool {
        self.brs
    }

    /// Set bit rate switch flag
    pub fn set_brs(&mut self, brs: bool) {
        self.brs = brs;
    }

    /// Get the data bytes
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the data length code
    #[must_use]
    pub fn dlc(&self) -> u8 {
        dlc_from_len(self.data.len())
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
}

/// Convert data length to CAN FD DLC
#[allow(clippy::cast_possible_truncation)]
fn dlc_from_len(len: usize) -> u8 {
    match len {
        0..=8 => len as u8,
        9..=12 => 9,
        13..=16 => 10,
        17..=20 => 11,
        21..=24 => 12,
        25..=32 => 13,
        33..=48 => 14,
        _ => 15,
    }
}

// ============================================================================
// CAN XL Frame
// ============================================================================

/// CAN XL Service Data Unit Type (SDT)
///
/// Defines the type of content in the CAN XL frame's data field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CanXlSdt {
    /// Content type not specified (0x00)
    #[default]
    ContentBasedAddressing,
    /// Classical CAN mapped to CAN XL (0x01)
    ClassicalCanTunneling,
    /// CAN FD mapped to CAN XL (0x02)
    CanFdTunneling,
    /// IEEE 802.3 Ethernet frame (0x03)
    Ethernet,
    /// IPv4 packet (0x04)
    Ipv4,
    /// IPv6 packet (0x05)
    Ipv6,
    /// Reserved for future use (value stored)
    Reserved(u8),
}

impl CanXlSdt {
    /// Get the SDT value as a byte
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::ContentBasedAddressing => 0x00,
            Self::ClassicalCanTunneling => 0x01,
            Self::CanFdTunneling => 0x02,
            Self::Ethernet => 0x03,
            Self::Ipv4 => 0x04,
            Self::Ipv6 => 0x05,
            Self::Reserved(v) => *v,
        }
    }
}

impl From<u8> for CanXlSdt {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::ContentBasedAddressing,
            0x01 => Self::ClassicalCanTunneling,
            0x02 => Self::CanFdTunneling,
            0x03 => Self::Ethernet,
            0x04 => Self::Ipv4,
            0x05 => Self::Ipv6,
            v => Self::Reserved(v),
        }
    }
}

impl From<CanXlSdt> for u8 {
    fn from(sdt: CanXlSdt) -> Self {
        sdt.as_u8()
    }
}

/// CAN XL frame
///
/// CAN XL is the newest CAN standard, supporting:
/// - Up to 2048 bytes of data (vs 64 for CAN FD)
/// - Priority field for arbitration
/// - Virtual CAN Network ID (VCID) for network segmentation
/// - Service Data Unit Type (SDT) for content identification
/// - Acceptance Field (AF) for filtering
/// - Backward compatibility with CAN FD
///
/// # Example
///
/// ```
/// use busmaster_core::CanXlFrame;
///
/// // Create a CAN XL frame with large payload
/// let data = vec![0xAA; 1024]; // 1KB payload
/// let frame = CanXlFrame::new(0x123, false, &data).unwrap();
/// assert_eq!(frame.data().len(), 1024);
/// assert_eq!(frame.data_length(), 1024);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanXlFrame {
    /// CAN identifier (11-bit standard or 29-bit extended)
    id: u32,
    /// Extended ID flag
    extended: bool,
    /// Priority field (0-255, lower = higher priority)
    priority: u8,
    /// Virtual CAN Network ID (0-255)
    vcid: u8,
    /// Service Data Unit Type
    sdt: CanXlSdt,
    /// Acceptance Field (32-bit)
    acceptance_field: u32,
    /// Simple Extended Content (SEC) flag
    sec: bool,
    /// Data bytes (up to 2048)
    data: Vec<u8>,
    /// Timestamp in microseconds
    timestamp_us: u64,
    /// Channel number
    channel: u8,
}

impl CanXlFrame {
    /// Create a new CAN XL frame
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ID exceeds the maximum for the frame type
    /// - The data length exceeds 2048 bytes
    /// - The data length is not a valid CAN XL length
    pub fn new(id: u32, extended: bool, data: &[u8]) -> Result<Self> {
        let max_id = if extended {
            MAX_EXTENDED_ID
        } else {
            MAX_STANDARD_ID
        };
        if id > max_id {
            return Err(BusmasterError::InvalidCanId { id, max: max_id });
        }
        if data.len() > MAX_CANXL_DATA_LEN {
            return Err(BusmasterError::InvalidDataLength {
                len: data.len(),
                max: MAX_CANXL_DATA_LEN,
            });
        }
        // CAN XL requires data length to be 1-2048 bytes (0 is not valid)
        // and must be word-aligned (multiple of 4) for lengths > 48
        if data.len() > 48 && data.len() % 4 != 0 {
            return Err(BusmasterError::protocol(format!(
                "CAN XL data length {} must be word-aligned (multiple of 4) for lengths > 48",
                data.len()
            )));
        }
        Ok(Self {
            id,
            extended,
            priority: 0,
            vcid: 0,
            sdt: CanXlSdt::default(),
            acceptance_field: 0,
            sec: false,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
        })
    }

    /// Create a CAN XL frame with full configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ID exceeds the maximum for the frame type
    /// - The data length exceeds 2048 bytes
    /// - The data length is not word-aligned for lengths > 48
    #[allow(clippy::too_many_arguments)]
    pub fn new_full(
        id: u32,
        extended: bool,
        priority: u8,
        vcid: u8,
        sdt: CanXlSdt,
        acceptance_field: u32,
        data: &[u8],
    ) -> Result<Self> {
        let mut frame = Self::new(id, extended, data)?;
        frame.priority = priority;
        frame.vcid = vcid;
        frame.sdt = sdt;
        frame.acceptance_field = acceptance_field;
        Ok(frame)
    }

    /// Get the CAN ID
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Check if this is an extended frame
    #[must_use]
    pub fn is_extended(&self) -> bool {
        self.extended
    }

    /// Get the priority field (0-255, lower = higher priority)
    #[must_use]
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// Set the priority field
    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority;
    }

    /// Get the Virtual CAN Network ID
    #[must_use]
    pub fn vcid(&self) -> u8 {
        self.vcid
    }

    /// Set the Virtual CAN Network ID
    pub fn set_vcid(&mut self, vcid: u8) {
        self.vcid = vcid;
    }

    /// Get the Service Data Unit Type
    #[must_use]
    pub fn sdt(&self) -> CanXlSdt {
        self.sdt
    }

    /// Set the Service Data Unit Type
    pub fn set_sdt(&mut self, sdt: CanXlSdt) {
        self.sdt = sdt;
    }

    /// Get the Acceptance Field
    #[must_use]
    pub fn acceptance_field(&self) -> u32 {
        self.acceptance_field
    }

    /// Set the Acceptance Field
    pub fn set_acceptance_field(&mut self, af: u32) {
        self.acceptance_field = af;
    }

    /// Check if Simple Extended Content (SEC) flag is set
    #[must_use]
    pub fn is_sec(&self) -> bool {
        self.sec
    }

    /// Set the Simple Extended Content (SEC) flag
    pub fn set_sec(&mut self, sec: bool) {
        self.sec = sec;
    }

    /// Get the data bytes
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the actual data length in bytes
    #[must_use]
    pub fn data_length(&self) -> u16 {
        #[allow(clippy::cast_possible_truncation)]
        {
            self.data.len() as u16
        }
    }

    /// Get the Data Length Code (DLC) for CAN XL
    ///
    /// CAN XL uses an 11-bit data length field directly encoding the byte count.
    /// Valid lengths are 1-2048 bytes.
    #[must_use]
    pub fn dlc(&self) -> u16 {
        self.data_length()
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

    /// Check if this frame can be represented as CAN FD
    ///
    /// Returns true if data length <= 64 bytes
    #[must_use]
    pub fn is_canfd_compatible(&self) -> bool {
        self.data.len() <= MAX_CANFD_DATA_LEN
    }

    /// Check if this frame can be represented as classical CAN
    ///
    /// Returns true if data length <= 8 bytes
    #[must_use]
    pub fn is_can_compatible(&self) -> bool {
        self.data.len() <= MAX_CAN_DATA_LEN
    }

    /// Convert to CAN FD frame if compatible
    ///
    /// # Errors
    ///
    /// Returns an error if data length exceeds 64 bytes
    pub fn to_canfd(&self) -> Result<CanFdFrame> {
        if !self.is_canfd_compatible() {
            return Err(BusmasterError::InvalidDataLength {
                len: self.data.len(),
                max: MAX_CANFD_DATA_LEN,
            });
        }
        let mut frame = CanFdFrame::new(self.id, self.extended, &self.data)?;
        frame.set_timestamp(self.timestamp_us);
        frame.set_channel(self.channel);
        Ok(frame)
    }

    /// Convert to classical CAN frame if compatible
    ///
    /// # Errors
    ///
    /// Returns an error if data length exceeds 8 bytes
    pub fn to_can(&self) -> Result<CanFrame> {
        if !self.is_can_compatible() {
            return Err(BusmasterError::InvalidDataLength {
                len: self.data.len(),
                max: MAX_CAN_DATA_LEN,
            });
        }
        let frame = if self.extended {
            CanFrame::new_extended(self.id, &self.data)?
        } else {
            CanFrame::new_standard(self.id, &self.data)?
        };
        Ok(frame)
    }

    /// Create CAN XL frame from CAN FD frame
    #[must_use]
    pub fn from_canfd(frame: &CanFdFrame) -> Self {
        Self {
            id: frame.id(),
            extended: frame.is_extended(),
            priority: 0,
            vcid: 0,
            sdt: CanXlSdt::CanFdTunneling,
            acceptance_field: 0,
            sec: false,
            data: frame.data().to_vec(),
            timestamp_us: frame.timestamp_us(),
            channel: frame.channel(),
        }
    }

    /// Create CAN XL frame from classical CAN frame
    #[must_use]
    pub fn from_can(frame: &CanFrame) -> Self {
        Self {
            id: frame.id(),
            extended: frame.is_extended(),
            priority: 0,
            vcid: 0,
            sdt: CanXlSdt::ClassicalCanTunneling,
            acceptance_field: 0,
            sec: false,
            data: frame.data().to_vec(),
            timestamp_us: frame.timestamp_us(),
            channel: frame.channel(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_frame_creation() {
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        assert_eq!(frame.id(), 0x123);
        assert!(!frame.is_extended());
        assert_eq!(frame.data(), &[1, 2, 3, 4]);
        assert_eq!(frame.dlc(), 4);
    }

    #[test]
    fn test_extended_frame_creation() {
        let frame = CanFrame::new_extended(0x12345678, &[1, 2, 3]).unwrap();
        assert_eq!(frame.id(), 0x12345678);
        assert!(frame.is_extended());
    }

    #[test]
    fn test_invalid_standard_id() {
        let result = CanFrame::new_standard(0x800, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_extended_id() {
        let result = CanFrame::new_extended(0x20000000, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_data_length() {
        let result = CanFrame::new_standard(0x100, &[0; 9]);
        assert!(result.is_err());
    }

    #[test]
    fn test_canfd_frame_creation() {
        let frame = CanFdFrame::new(0x123, false, &[0; 64]).unwrap();
        assert_eq!(frame.id(), 0x123);
        assert_eq!(frame.data().len(), 64);
        assert_eq!(frame.dlc(), 15);
    }

    #[test]
    fn test_canfd_dlc_mapping() {
        assert_eq!(dlc_from_len(8), 8);
        assert_eq!(dlc_from_len(12), 9);
        assert_eq!(dlc_from_len(16), 10);
        assert_eq!(dlc_from_len(20), 11);
        assert_eq!(dlc_from_len(24), 12);
        assert_eq!(dlc_from_len(32), 13);
        assert_eq!(dlc_from_len(48), 14);
        assert_eq!(dlc_from_len(64), 15);
    }

    #[test]
    fn test_frame_timestamp() {
        let mut frame = CanFrame::new_standard(0x100, &[1, 2]).unwrap();
        assert_eq!(frame.timestamp_us(), 0);
        frame.set_timestamp(12345678);
        assert_eq!(frame.timestamp_us(), 12345678);
    }

    #[test]
    fn test_frame_channel() {
        let mut frame = CanFrame::new_standard(0x100, &[1, 2]).unwrap();
        assert_eq!(frame.channel(), 0);
        frame.set_channel(2);
        assert_eq!(frame.channel(), 2);
    }

    #[test]
    fn test_canfd_brs_flag() {
        let mut frame = CanFdFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        assert!(!frame.is_brs());
        frame.set_brs(true);
        assert!(frame.is_brs());
    }

    #[test]
    fn test_empty_frame() {
        let frame = CanFrame::new_standard(0x000, &[]).unwrap();
        assert_eq!(frame.id(), 0);
        assert_eq!(frame.dlc(), 0);
        assert!(frame.data().is_empty());
    }

    #[test]
    fn test_max_standard_id() {
        let frame = CanFrame::new_standard(MAX_STANDARD_ID, &[0xFF]).unwrap();
        assert_eq!(frame.id(), 0x7FF);
    }

    #[test]
    fn test_max_extended_id() {
        let frame = CanFrame::new_extended(MAX_EXTENDED_ID, &[0xFF]).unwrap();
        assert_eq!(frame.id(), 0x1FFFFFFF);
    }

    #[test]
    fn test_rtr_flag() {
        let frame = CanFrame::new_standard(0x100, &[]).unwrap();
        assert!(!frame.is_rtr());
    }

    #[test]
    fn test_frame_equality() {
        let frame1 = CanFrame::new_standard(0x123, &[1, 2, 3]).unwrap();
        let frame2 = CanFrame::new_standard(0x123, &[1, 2, 3]).unwrap();
        assert_eq!(frame1, frame2);
    }

    #[test]
    fn test_frame_clone() {
        let frame1 = CanFrame::new_standard(0x123, &[1, 2, 3]).unwrap();
        let frame2 = frame1.clone();
        assert_eq!(frame1, frame2);
    }

    // CAN XL Tests
    #[test]
    fn test_canxl_frame_creation() {
        let data = vec![0xAA; 1024];
        let frame = CanXlFrame::new(0x123, false, &data).unwrap();
        assert_eq!(frame.id(), 0x123);
        assert!(!frame.is_extended());
        assert_eq!(frame.data().len(), 1024);
        assert_eq!(frame.data_length(), 1024);
        assert_eq!(frame.dlc(), 1024);
    }

    #[test]
    fn test_canxl_frame_max_size() {
        let data = vec![0xFF; MAX_CANXL_DATA_LEN];
        let frame = CanXlFrame::new(0x100, false, &data).unwrap();
        assert_eq!(frame.data().len(), 2048);
    }

    #[test]
    fn test_canxl_frame_invalid_size() {
        let data = vec![0xFF; MAX_CANXL_DATA_LEN + 1];
        let result = CanXlFrame::new(0x100, false, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_canxl_frame_word_alignment() {
        // 49 bytes is > 48 and not word-aligned
        let data = vec![0xFF; 49];
        let result = CanXlFrame::new(0x100, false, &data);
        assert!(result.is_err());

        // 50 bytes is > 48 and not word-aligned
        let data = vec![0xFF; 50];
        let result = CanXlFrame::new(0x100, false, &data);
        assert!(result.is_err());

        // 52 bytes is > 48 and word-aligned (52 / 4 = 13)
        let data = vec![0xFF; 52];
        let result = CanXlFrame::new(0x100, false, &data);
        assert!(result.is_ok());

        // 56 bytes is > 48 and word-aligned
        let data = vec![0xFF; 56];
        let result = CanXlFrame::new(0x100, false, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_canxl_frame_small_data() {
        // Small data (<=48) doesn't need word alignment
        let data = vec![0xFF; 47];
        let result = CanXlFrame::new(0x100, false, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_canxl_priority() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        assert_eq!(frame.priority(), 0);
        frame.set_priority(128);
        assert_eq!(frame.priority(), 128);
    }

    #[test]
    fn test_canxl_vcid() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        assert_eq!(frame.vcid(), 0);
        frame.set_vcid(42);
        assert_eq!(frame.vcid(), 42);
    }

    #[test]
    fn test_canxl_sdt() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        assert_eq!(frame.sdt(), CanXlSdt::ContentBasedAddressing);
        frame.set_sdt(CanXlSdt::Ethernet);
        assert_eq!(frame.sdt(), CanXlSdt::Ethernet);
    }

    #[test]
    fn test_canxl_sdt_conversion() {
        assert_eq!(u8::from(CanXlSdt::ContentBasedAddressing), 0x00);
        assert_eq!(u8::from(CanXlSdt::ClassicalCanTunneling), 0x01);
        assert_eq!(u8::from(CanXlSdt::CanFdTunneling), 0x02);
        assert_eq!(u8::from(CanXlSdt::Ethernet), 0x03);
        assert_eq!(u8::from(CanXlSdt::Ipv4), 0x04);
        assert_eq!(u8::from(CanXlSdt::Ipv6), 0x05);
        assert_eq!(u8::from(CanXlSdt::Reserved(0x10)), 0x10);

        assert_eq!(CanXlSdt::from(0x00), CanXlSdt::ContentBasedAddressing);
        assert_eq!(CanXlSdt::from(0x03), CanXlSdt::Ethernet);
        assert_eq!(CanXlSdt::from(0x10), CanXlSdt::Reserved(0x10));
    }

    #[test]
    fn test_canxl_acceptance_field() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        assert_eq!(frame.acceptance_field(), 0);
        frame.set_acceptance_field(0x12345678);
        assert_eq!(frame.acceptance_field(), 0x12345678);
    }

    #[test]
    fn test_canxl_sec_flag() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        assert!(!frame.is_sec());
        frame.set_sec(true);
        assert!(frame.is_sec());
    }

    #[test]
    fn test_canxl_compatibility_checks() {
        // Small frame is compatible with all
        let small = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        assert!(small.is_can_compatible());
        assert!(small.is_canfd_compatible());

        // Medium frame is CAN FD compatible only
        let medium = CanXlFrame::new(0x100, false, &[0; 32]).unwrap();
        assert!(!medium.is_can_compatible());
        assert!(medium.is_canfd_compatible());

        // Large frame is CAN XL only
        let large = CanXlFrame::new(0x100, false, &[0; 128]).unwrap();
        assert!(!large.is_can_compatible());
        assert!(!large.is_canfd_compatible());
    }

    #[test]
    fn test_canxl_to_canfd() {
        let xl = CanXlFrame::new(0x123, true, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        let fd = xl.to_canfd().unwrap();
        assert_eq!(fd.id(), 0x123);
        assert!(fd.is_extended());
        assert_eq!(fd.data(), &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_canxl_to_canfd_too_large() {
        let xl = CanXlFrame::new(0x123, false, &[0; 128]).unwrap();
        assert!(xl.to_canfd().is_err());
    }

    #[test]
    fn test_canxl_to_can() {
        let xl = CanXlFrame::new(0x123, false, &[1, 2, 3, 4]).unwrap();
        let can = xl.to_can().unwrap();
        assert_eq!(can.id(), 0x123);
        assert!(!can.is_extended());
        assert_eq!(can.data(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_canxl_to_can_too_large() {
        let xl = CanXlFrame::new(0x123, false, &[0; 16]).unwrap();
        assert!(xl.to_can().is_err());
    }

    #[test]
    fn test_canxl_from_canfd() {
        let fd = CanFdFrame::new(0x456, true, &[0xAA; 32]).unwrap();
        let xl = CanXlFrame::from_canfd(&fd);
        assert_eq!(xl.id(), 0x456);
        assert!(xl.is_extended());
        assert_eq!(xl.data().len(), 32);
        assert_eq!(xl.sdt(), CanXlSdt::CanFdTunneling);
    }

    #[test]
    fn test_canxl_from_can() {
        let can = CanFrame::new_standard(0x789, &[1, 2, 3]).unwrap();
        let xl = CanXlFrame::from_can(&can);
        assert_eq!(xl.id(), 0x789);
        assert!(!xl.is_extended());
        assert_eq!(xl.data(), &[1, 2, 3]);
        assert_eq!(xl.sdt(), CanXlSdt::ClassicalCanTunneling);
    }

    #[test]
    fn test_canxl_new_full() {
        let frame = CanXlFrame::new_full(
            0x100,
            true,
            64,
            10,
            CanXlSdt::Ipv4,
            0xDEADBEEF,
            &[1, 2, 3, 4],
        )
        .unwrap();
        assert_eq!(frame.id(), 0x100);
        assert!(frame.is_extended());
        assert_eq!(frame.priority(), 64);
        assert_eq!(frame.vcid(), 10);
        assert_eq!(frame.sdt(), CanXlSdt::Ipv4);
        assert_eq!(frame.acceptance_field(), 0xDEADBEEF);
    }

    #[test]
    fn test_canxl_timestamp_channel() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2]).unwrap();
        assert_eq!(frame.timestamp_us(), 0);
        assert_eq!(frame.channel(), 0);

        frame.set_timestamp(123456789);
        frame.set_channel(3);

        assert_eq!(frame.timestamp_us(), 123456789);
        assert_eq!(frame.channel(), 3);
    }

    #[test]
    fn test_canxl_extended_id() {
        let frame = CanXlFrame::new(0x12345678, true, &[1, 2, 3, 4]).unwrap();
        assert_eq!(frame.id(), 0x12345678);
        assert!(frame.is_extended());
    }

    #[test]
    fn test_canxl_invalid_standard_id() {
        let result = CanXlFrame::new(0x800, false, &[1, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_canxl_invalid_extended_id() {
        let result = CanXlFrame::new(0x20000000, true, &[1, 2]);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Any valid standard ID (0-0x7FF) creates a valid frame
        #[test]
        fn prop_valid_standard_id_creates_frame(id in 0u32..=MAX_STANDARD_ID) {
            let result = CanFrame::new_standard(id, &[]);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap().id(), id);
        }

        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Any valid extended ID (0-0x1FFFFFFF) creates a valid frame
        #[test]
        fn prop_valid_extended_id_creates_frame(id in 0u32..=MAX_EXTENDED_ID) {
            let result = CanFrame::new_extended(id, &[]);
            prop_assert!(result.is_ok());
            let frame = result.unwrap();
            prop_assert_eq!(frame.id(), id);
            prop_assert!(frame.is_extended());
        }

        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Invalid standard IDs are rejected
        #[test]
        fn prop_invalid_standard_id_rejected(id in (MAX_STANDARD_ID + 1)..=u32::MAX) {
            let result = CanFrame::new_standard(id, &[]);
            prop_assert!(result.is_err());
        }

        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Invalid extended IDs are rejected
        #[test]
        fn prop_invalid_extended_id_rejected(id in (MAX_EXTENDED_ID + 1)..=u32::MAX) {
            let result = CanFrame::new_extended(id, &[]);
            prop_assert!(result.is_err());
        }

        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Valid data lengths (0-8) are accepted for CAN 2.0
        #[test]
        fn prop_valid_data_length_accepted(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CAN_DATA_LEN)
        ) {
            let result = CanFrame::new_standard(id, &data);
            prop_assert!(result.is_ok());
            let frame = result.unwrap();
            prop_assert_eq!(frame.data(), data.as_slice());
            prop_assert_eq!(frame.dlc() as usize, data.len());
        }

        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Invalid data lengths (>8) are rejected for CAN 2.0
        #[test]
        fn prop_invalid_data_length_rejected(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), (MAX_CAN_DATA_LEN + 1)..=20)
        ) {
            let result = CanFrame::new_standard(id, &data);
            prop_assert!(result.is_err());
        }

        /// **Validates: Requirements 1.2.3, 1.2.9**
        /// Property: Valid CAN FD data lengths (0-64) are accepted
        #[test]
        fn prop_canfd_valid_data_length(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CANFD_DATA_LEN)
        ) {
            let result = CanFdFrame::new(id, false, &data);
            prop_assert!(result.is_ok());
            let frame = result.unwrap();
            prop_assert_eq!(frame.data(), data.as_slice());
        }

        /// **Validates: Requirements 1.2.3, 1.2.9**
        /// Property: Invalid CAN FD data lengths (>64) are rejected
        #[test]
        fn prop_canfd_invalid_data_length_rejected(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), (MAX_CANFD_DATA_LEN + 1)..=100)
        ) {
            let result = CanFdFrame::new(id, false, &data);
            prop_assert!(result.is_err());
        }

        /// **Validates: Requirements 1.2.3, 1.2.9**
        /// Property: CAN FD DLC mapping is consistent
        #[test]
        fn prop_canfd_dlc_consistent(len in 0usize..=MAX_CANFD_DATA_LEN) {
            let dlc = dlc_from_len(len);
            // DLC should be in valid range
            prop_assert!(dlc <= 15);
            // DLC should be at least as large as needed for the data
            let actual_capacity = len_from_dlc(dlc);
            prop_assert!(actual_capacity >= len);
        }

        /// **Validates: Requirements 1.2.7, 1.2.9**
        /// Property: Frame serialization roundtrip preserves data
        #[test]
        fn prop_frame_serde_roundtrip(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CAN_DATA_LEN)
        ) {
            let frame = CanFrame::new_standard(id, &data).unwrap();
            let json = serde_json::to_string(&frame).unwrap();
            let decoded: CanFrame = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(frame, decoded);
        }

        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Timestamp can be set to any value
        #[test]
        fn prop_timestamp_any_value(timestamp in any::<u64>()) {
            let mut frame = CanFrame::new_standard(0x100, &[]).unwrap();
            frame.set_timestamp(timestamp);
            prop_assert_eq!(frame.timestamp_us(), timestamp);
        }

        /// **Validates: Requirements 1.2.2, 1.2.9**
        /// Property: Channel can be set to any value
        #[test]
        fn prop_channel_any_value(channel in any::<u8>()) {
            let mut frame = CanFrame::new_standard(0x100, &[]).unwrap();
            frame.set_channel(channel);
            prop_assert_eq!(frame.channel(), channel);
        }

        // CAN XL Property Tests

        /// **Validates: Requirements 6.7.1**
        /// Property: Valid CAN XL data lengths (word-aligned for >48) are accepted
        #[test]
        fn prop_canxl_valid_data_length(
            id in 0u32..=MAX_STANDARD_ID,
            // Generate word-aligned lengths for >48
            len in prop::strategy::Union::new(vec![
                (0usize..=48).boxed(),
                (49usize..=512).prop_map(|l| (l / 4) * 4).boxed(),
            ])
        ) {
            let data = vec![0u8; len];
            let result = CanXlFrame::new(id, false, &data);
            prop_assert!(result.is_ok());
            let frame = result.unwrap();
            prop_assert_eq!(frame.data().len(), len);
        }

        /// **Validates: Requirements 6.7.1**
        /// Property: CAN XL priority can be any u8 value
        #[test]
        fn prop_canxl_priority_any_value(priority in any::<u8>()) {
            let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
            frame.set_priority(priority);
            prop_assert_eq!(frame.priority(), priority);
        }

        /// **Validates: Requirements 6.7.1**
        /// Property: CAN XL VCID can be any u8 value
        #[test]
        fn prop_canxl_vcid_any_value(vcid in any::<u8>()) {
            let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
            frame.set_vcid(vcid);
            prop_assert_eq!(frame.vcid(), vcid);
        }

        /// **Validates: Requirements 6.7.1**
        /// Property: CAN XL acceptance field can be any u32 value
        #[test]
        fn prop_canxl_acceptance_field_any_value(af in any::<u32>()) {
            let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
            frame.set_acceptance_field(af);
            prop_assert_eq!(frame.acceptance_field(), af);
        }

        /// **Validates: Requirements 6.7.5**
        /// Property: CAN XL to CAN FD conversion preserves data for compatible frames
        #[test]
        fn prop_canxl_to_canfd_roundtrip(
            id in 0u32..=MAX_STANDARD_ID,
            // Generate valid CAN XL lengths: 0-48 (any), or 49-64 word-aligned (52, 56, 60, 64)
            len in prop::strategy::Union::new(vec![
                (0usize..=48).boxed(),
                proptest::strategy::Just(52usize).boxed(),
                proptest::strategy::Just(56usize).boxed(),
                proptest::strategy::Just(60usize).boxed(),
                proptest::strategy::Just(64usize).boxed(),
            ])
        ) {
            let data: Vec<u8> = vec![0xAA; len];
            let xl = CanXlFrame::new(id, false, &data).unwrap();
            let fd = xl.to_canfd().unwrap();
            prop_assert_eq!(fd.id(), id);
            prop_assert_eq!(fd.data(), data.as_slice());
        }

        /// **Validates: Requirements 6.7.5**
        /// Property: CAN XL from CAN FD preserves all data
        #[test]
        fn prop_canxl_from_canfd_preserves_data(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CANFD_DATA_LEN)
        ) {
            let fd = CanFdFrame::new(id, false, &data).unwrap();
            let xl = CanXlFrame::from_canfd(&fd);
            prop_assert_eq!(xl.id(), id);
            prop_assert_eq!(xl.data(), data.as_slice());
            prop_assert_eq!(xl.sdt(), CanXlSdt::CanFdTunneling);
        }

        /// **Validates: Requirements 6.7.1**
        /// Property: CAN XL serialization roundtrip preserves data
        #[test]
        fn prop_canxl_serde_roundtrip(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=48)
        ) {
            let frame = CanXlFrame::new(id, false, &data).unwrap();
            let json = serde_json::to_string(&frame).unwrap();
            let decoded: CanXlFrame = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(frame, decoded);
        }
    }
}

/// Convert CAN FD DLC to data length (inverse of dlc_from_len)
#[cfg(test)]
fn len_from_dlc(dlc: u8) -> usize {
    match dlc {
        0..=8 => dlc as usize,
        9 => 12,
        10 => 16,
        11 => 20,
        12 => 24,
        13 => 32,
        14 => 48,
        15 => 64,
        _ => 64,
    }
}
