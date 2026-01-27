//! CAN protocol parsing and encoding
//!
//! This module provides functionality for parsing CAN frames from raw bytes
//! and encoding CAN frames to bytes. It supports both CAN 2.0 (standard and
//! extended) and CAN FD frames.
//!
//! # Wire Format
//!
//! The wire format used is a simple binary format suitable for inter-process
//! communication and logging:
//!
//! ## CAN 2.0 Frame Format (13+ bytes)
//! ```text
//! Offset  Size  Description
//! 0       4     CAN ID (little-endian, bit 31 = extended flag, bit 30 = RTR)
//! 4       1     DLC (0-8)
//! 5       1     Channel number
//! 6       1     Flags (bit 0 = TX, bits 1-7 reserved)
//! 7       1     Reserved (padding)
//! 8       8     Timestamp in microseconds (little-endian)
//! 16      0-8   Data bytes (DLC bytes)
//! ```
//!
//! ## CAN FD Frame Format (17+ bytes)
//! ```text
//! Offset  Size  Description
//! 0       4     CAN ID (little-endian, bit 31 = extended flag)
//! 4       1     DLC (0-15)
//! 5       1     Channel number
//! 6       1     Flags (bit 0 = TX, bit 1 = BRS, bit 2 = ESI, bit 7 = FD flag)
//! 7       1     Reserved (padding)
//! 8       8     Timestamp in microseconds (little-endian)
//! 16      0-64  Data bytes (actual length from DLC)
//! ```
//!
//! # Example
//!
//! ```
//! use busmaster_core::CanFrame;
//! use busmaster_proto::{CanParser, CanEncoder};
//!
//! // Create a frame
//! let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03, 0x04]).unwrap();
//!
//! // Encode to bytes
//! let bytes = CanEncoder::encode(&frame);
//!
//! // Parse back
//! let parsed = CanParser::parse(&bytes).unwrap();
//! assert_eq!(frame.id(), parsed.id());
//! ```

use busmaster_core::{
    BusmasterError, CanFdFrame, CanFrame, CanXlFrame, CanXlSdt, Result, MAX_CAN_DATA_LEN,
    MAX_CANXL_DATA_LEN, MAX_EXTENDED_ID, MAX_STANDARD_ID,
};

/// Minimum size for a CAN 2.0 frame (header only, no data)
pub const MIN_CAN_FRAME_SIZE: usize = 16;

/// Minimum size for a CAN FD frame (header only, no data)
pub const MIN_CANFD_FRAME_SIZE: usize = 16;

/// Minimum size for a CAN XL frame (header only, no data)
/// CAN XL has additional fields: priority, VCID, SDT, acceptance field
pub const MIN_CANXL_FRAME_SIZE: usize = 24;

/// Flag bit indicating extended ID
const FLAG_EXTENDED: u32 = 0x8000_0000;

/// Flag bit indicating RTR frame
const FLAG_RTR: u32 = 0x4000_0000;

/// ID mask (29 bits)
const ID_MASK: u32 = 0x1FFF_FFFF;

/// Frame flags byte positions
const FLAG_TX: u8 = 0x01;
const FLAG_BRS: u8 = 0x02;
#[allow(dead_code)] // Reserved for future CAN FD error state indicator
const FLAG_ESI: u8 = 0x04;
const FLAG_XL: u8 = 0x40;
const FLAG_FD: u8 = 0x80;

/// CAN frame parser
///
/// Parses raw bytes into CAN frame structures.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanParser;

impl CanParser {
    /// Create a new CAN parser
    pub fn new() -> Self {
        Self
    }

    /// Parse CAN 2.0 frame from raw bytes
    ///
    /// # Arguments
    /// * `bytes` - Raw byte slice containing the frame data
    ///
    /// # Returns
    /// * `Ok(CanFrame)` - Successfully parsed frame
    /// * `Err(BusmasterError)` - Parse error
    ///
    /// # Errors
    /// Returns `BusmasterError::Parse` if:
    /// - Input is too short
    /// - DLC exceeds maximum (8)
    /// - ID exceeds maximum for frame type
    pub fn parse(bytes: &[u8]) -> Result<CanFrame> {
        if bytes.len() < MIN_CAN_FRAME_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Frame too short: {} bytes (minimum {})",
                    bytes.len(),
                    MIN_CAN_FRAME_SIZE
                ),
            });
        }

        // Parse header
        let id_flags = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let dlc = bytes[4];
        let channel = bytes[5];
        let flags = bytes[6];
        let timestamp = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);

        // Check if this is a CAN FD frame
        if flags & FLAG_FD != 0 {
            return Err(BusmasterError::Parse {
                message: "CAN FD frame detected, use parse_fd() instead".into(),
            });
        }

        // Validate DLC
        if dlc > MAX_CAN_DATA_LEN as u8 {
            return Err(BusmasterError::Parse {
                message: format!("Invalid DLC: {} (max {})", dlc, MAX_CAN_DATA_LEN),
            });
        }

        // Check we have enough data
        let expected_len = MIN_CAN_FRAME_SIZE + dlc as usize;
        if bytes.len() < expected_len {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Frame truncated: {} bytes (expected {} for DLC {})",
                    bytes.len(),
                    expected_len,
                    dlc
                ),
            });
        }

        // Extract ID and flags
        let is_extended = id_flags & FLAG_EXTENDED != 0;
        let is_rtr = id_flags & FLAG_RTR != 0;
        let id = id_flags & ID_MASK;

        // Validate ID range
        let max_id = if is_extended {
            MAX_EXTENDED_ID
        } else {
            MAX_STANDARD_ID
        };
        if id > max_id {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Invalid {} ID: 0x{:X} (max 0x{:X})",
                    if is_extended { "extended" } else { "standard" },
                    id,
                    max_id
                ),
            });
        }

        // Extract data
        let data = &bytes[MIN_CAN_FRAME_SIZE..MIN_CAN_FRAME_SIZE + dlc as usize];

        // Create frame
        let mut frame = if is_extended {
            CanFrame::new_extended(id, data)?
        } else {
            CanFrame::new_standard(id, data)?
        };

        frame.set_timestamp(timestamp);
        frame.set_channel(channel);

        // Note: RTR and TX flags are parsed but CanFrame doesn't expose setters for them
        // This is intentional - RTR frames are rare and TX is typically set by the sender
        let _ = is_rtr;
        let _ = flags & FLAG_TX;

        Ok(frame)
    }

    /// Parse CAN FD frame from raw bytes
    ///
    /// # Arguments
    /// * `bytes` - Raw byte slice containing the frame data
    ///
    /// # Returns
    /// * `Ok(CanFdFrame)` - Successfully parsed frame
    /// * `Err(BusmasterError)` - Parse error
    pub fn parse_fd(bytes: &[u8]) -> Result<CanFdFrame> {
        if bytes.len() < MIN_CANFD_FRAME_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Frame too short: {} bytes (minimum {})",
                    bytes.len(),
                    MIN_CANFD_FRAME_SIZE
                ),
            });
        }

        // Parse header
        let id_flags = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let dlc = bytes[4];
        let channel = bytes[5];
        let flags = bytes[6];
        let timestamp = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);

        // Validate DLC
        if dlc > 15 {
            return Err(BusmasterError::Parse {
                message: format!("Invalid CAN FD DLC: {} (max 15)", dlc),
            });
        }

        // Calculate actual data length from DLC
        let data_len = dlc_to_len(dlc);

        // Check we have enough data
        let expected_len = MIN_CANFD_FRAME_SIZE + data_len;
        if bytes.len() < expected_len {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Frame truncated: {} bytes (expected {} for DLC {})",
                    bytes.len(),
                    expected_len,
                    dlc
                ),
            });
        }

        // Extract ID and flags
        let is_extended = id_flags & FLAG_EXTENDED != 0;
        let id = id_flags & ID_MASK;

        // Validate ID range
        let max_id = if is_extended {
            MAX_EXTENDED_ID
        } else {
            MAX_STANDARD_ID
        };
        if id > max_id {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Invalid {} ID: 0x{:X} (max 0x{:X})",
                    if is_extended { "extended" } else { "standard" },
                    id,
                    max_id
                ),
            });
        }

        // Extract data
        let data = &bytes[MIN_CANFD_FRAME_SIZE..MIN_CANFD_FRAME_SIZE + data_len];

        // Create frame
        let mut frame = CanFdFrame::new(id, is_extended, data)?;
        frame.set_timestamp(timestamp);
        frame.set_channel(channel);
        frame.set_brs(flags & FLAG_BRS != 0);

        Ok(frame)
    }

    /// Parse CAN XL frame from raw bytes
    ///
    /// # Wire Format (CAN XL)
    /// ```text
    /// Offset  Size  Description
    /// 0       4     CAN ID (little-endian, bit 31 = extended flag)
    /// 4       2     Data length (little-endian, 1-2048)
    /// 6       1     Channel number
    /// 7       1     Flags (bit 6 = XL flag, bit 0 = TX)
    /// 8       8     Timestamp in microseconds (little-endian)
    /// 16      1     Priority (0-255)
    /// 17      1     VCID (Virtual CAN Network ID, 0-255)
    /// 18      1     SDT (Service Data Unit Type)
    /// 19      1     SEC flag and reserved
    /// 20      4     Acceptance Field (little-endian)
    /// 24      N     Data bytes (1-2048, word-aligned for >48)
    /// ```
    ///
    /// # Arguments
    /// * `bytes` - Raw byte slice containing the frame data
    ///
    /// # Returns
    /// * `Ok(CanXlFrame)` - Successfully parsed frame
    /// * `Err(BusmasterError)` - Parse error
    ///
    /// # Errors
    /// Returns `BusmasterError::Parse` if:
    /// - Input is too short
    /// - Data length exceeds 2048
    /// - Data length is not word-aligned for lengths > 48
    /// - ID exceeds maximum for frame type
    pub fn parse_xl(bytes: &[u8]) -> Result<CanXlFrame> {
        if bytes.len() < MIN_CANXL_FRAME_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "CAN XL frame too short: {} bytes (minimum {})",
                    bytes.len(),
                    MIN_CANXL_FRAME_SIZE
                ),
            });
        }

        // Parse header
        let id_flags = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let data_len = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
        let channel = bytes[6];
        let flags = bytes[7];
        let timestamp = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let priority = bytes[16];
        let vcid = bytes[17];
        let sdt = CanXlSdt::from(bytes[18]);
        let sec = bytes[19] & 0x01 != 0;
        let acceptance_field = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

        // Validate data length
        if data_len > MAX_CANXL_DATA_LEN {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Invalid CAN XL data length: {} (max {})",
                    data_len, MAX_CANXL_DATA_LEN
                ),
            });
        }

        // Check word alignment for lengths > 48
        if data_len > 48 && data_len % 4 != 0 {
            return Err(BusmasterError::Parse {
                message: format!(
                    "CAN XL data length {} must be word-aligned for lengths > 48",
                    data_len
                ),
            });
        }

        // Check we have enough data
        let expected_len = MIN_CANXL_FRAME_SIZE + data_len;
        if bytes.len() < expected_len {
            return Err(BusmasterError::Parse {
                message: format!(
                    "CAN XL frame truncated: {} bytes (expected {} for data length {})",
                    bytes.len(),
                    expected_len,
                    data_len
                ),
            });
        }

        // Extract ID and flags
        let is_extended = id_flags & FLAG_EXTENDED != 0;
        let id = id_flags & ID_MASK;

        // Validate ID range
        let max_id = if is_extended {
            MAX_EXTENDED_ID
        } else {
            MAX_STANDARD_ID
        };
        if id > max_id {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Invalid {} ID: 0x{:X} (max 0x{:X})",
                    if is_extended { "extended" } else { "standard" },
                    id,
                    max_id
                ),
            });
        }

        // Extract data
        let data = &bytes[MIN_CANXL_FRAME_SIZE..MIN_CANXL_FRAME_SIZE + data_len];

        // Create frame
        let mut frame = CanXlFrame::new_full(id, is_extended, priority, vcid, sdt, acceptance_field, data)?;
        frame.set_timestamp(timestamp);
        frame.set_channel(channel);
        frame.set_sec(sec);

        // Note: TX flag is parsed but not stored
        let _ = flags & FLAG_TX;

        Ok(frame)
    }

    /// Validate a CAN frame without fully parsing it
    ///
    /// This is a fast validation that checks basic structure without
    /// allocating memory for the data.
    pub fn validate(bytes: &[u8]) -> Result<()> {
        if bytes.len() < MIN_CAN_FRAME_SIZE {
            return Err(BusmasterError::Parse {
                message: "Frame too short".into(),
            });
        }

        let id_flags = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let dlc = bytes[4];
        let flags = bytes[6];

        let is_fd = flags & FLAG_FD != 0;
        let is_extended = id_flags & FLAG_EXTENDED != 0;
        let id = id_flags & ID_MASK;

        // Validate DLC
        let max_dlc = if is_fd { 15 } else { 8 };
        if dlc > max_dlc {
            return Err(BusmasterError::Parse {
                message: format!("Invalid DLC: {}", dlc),
            });
        }

        // Validate ID
        let max_id = if is_extended {
            MAX_EXTENDED_ID
        } else {
            MAX_STANDARD_ID
        };
        if id > max_id {
            return Err(BusmasterError::Parse {
                message: format!("Invalid ID: 0x{:X}", id),
            });
        }

        // Validate length
        let data_len = if is_fd { dlc_to_len(dlc) } else { dlc as usize };
        let expected_len = MIN_CAN_FRAME_SIZE + data_len;
        if bytes.len() < expected_len {
            return Err(BusmasterError::Parse {
                message: "Frame truncated".into(),
            });
        }

        Ok(())
    }
}

/// CAN frame encoder
///
/// Encodes CAN frame structures to raw bytes.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanEncoder;

impl CanEncoder {
    /// Create a new CAN encoder
    pub fn new() -> Self {
        Self
    }

    /// Encode CAN 2.0 frame to bytes
    ///
    /// # Arguments
    /// * `frame` - The CAN frame to encode
    ///
    /// # Returns
    /// A `Vec<u8>` containing the encoded frame
    pub fn encode(frame: &CanFrame) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(MIN_CAN_FRAME_SIZE + frame.data().len());

        // Build ID with flags
        let mut id_flags = frame.id();
        if frame.is_extended() {
            id_flags |= FLAG_EXTENDED;
        }
        if frame.is_rtr() {
            id_flags |= FLAG_RTR;
        }

        // Write header
        bytes.extend_from_slice(&id_flags.to_le_bytes());
        bytes.push(frame.dlc());
        bytes.push(frame.channel());
        bytes.push(0); // flags (TX would be set by sender)
        bytes.push(0); // reserved
        bytes.extend_from_slice(&frame.timestamp_us().to_le_bytes());

        // Write data
        bytes.extend_from_slice(frame.data());

        bytes
    }

    /// Encode CAN FD frame to bytes
    ///
    /// # Arguments
    /// * `frame` - The CAN FD frame to encode
    ///
    /// # Returns
    /// A `Vec<u8>` containing the encoded frame
    ///
    /// # Note
    /// CAN FD frames are padded to the next valid CAN FD data length
    /// (8, 12, 16, 20, 24, 32, 48, or 64 bytes) for wire compatibility.
    pub fn encode_fd(frame: &CanFdFrame) -> Vec<u8> {
        let dlc = frame.dlc();
        let padded_len = dlc_to_len(dlc);
        let mut bytes = Vec::with_capacity(MIN_CANFD_FRAME_SIZE + padded_len);

        // Build ID with flags
        let mut id_flags = frame.id();
        if frame.is_extended() {
            id_flags |= FLAG_EXTENDED;
        }

        // Build flags byte
        let mut flags = FLAG_FD;
        if frame.is_brs() {
            flags |= FLAG_BRS;
        }

        // Write header
        bytes.extend_from_slice(&id_flags.to_le_bytes());
        bytes.push(dlc);
        bytes.push(frame.channel());
        bytes.push(flags);
        bytes.push(0); // reserved
        bytes.extend_from_slice(&frame.timestamp_us().to_le_bytes());

        // Write data padded to valid CAN FD length
        bytes.extend_from_slice(frame.data());
        // Pad with zeros to reach the DLC-mapped length
        let padding_needed = padded_len.saturating_sub(frame.data().len());
        bytes.extend(std::iter::repeat(0u8).take(padding_needed));

        bytes
    }

    /// Encode CAN XL frame to bytes
    ///
    /// # Wire Format (CAN XL)
    /// ```text
    /// Offset  Size  Description
    /// 0       4     CAN ID (little-endian, bit 31 = extended flag)
    /// 4       2     Data length (little-endian, 1-2048)
    /// 6       1     Channel number
    /// 7       1     Flags (bit 6 = XL flag, bit 0 = TX)
    /// 8       8     Timestamp in microseconds (little-endian)
    /// 16      1     Priority (0-255)
    /// 17      1     VCID (Virtual CAN Network ID, 0-255)
    /// 18      1     SDT (Service Data Unit Type)
    /// 19      1     SEC flag and reserved
    /// 20      4     Acceptance Field (little-endian)
    /// 24      N     Data bytes (1-2048, word-aligned for >48)
    /// ```
    ///
    /// # Arguments
    /// * `frame` - The CAN XL frame to encode
    ///
    /// # Returns
    /// A `Vec<u8>` containing the encoded frame
    pub fn encode_xl(frame: &CanXlFrame) -> Vec<u8> {
        let data_len = frame.data_length() as usize;
        let mut bytes = Vec::with_capacity(MIN_CANXL_FRAME_SIZE + data_len);

        // Build ID with flags
        let mut id_flags = frame.id();
        if frame.is_extended() {
            id_flags |= FLAG_EXTENDED;
        }

        // Build flags byte
        let flags = FLAG_XL;

        // Write header
        bytes.extend_from_slice(&id_flags.to_le_bytes());
        bytes.extend_from_slice(&(data_len as u16).to_le_bytes());
        bytes.push(frame.channel());
        bytes.push(flags);
        bytes.extend_from_slice(&frame.timestamp_us().to_le_bytes());

        // Write CAN XL specific fields
        bytes.push(frame.priority());
        bytes.push(frame.vcid());
        bytes.push(frame.sdt().as_u8());
        bytes.push(if frame.is_sec() { 0x01 } else { 0x00 });
        bytes.extend_from_slice(&frame.acceptance_field().to_le_bytes());

        // Write data
        bytes.extend_from_slice(frame.data());

        bytes
    }

    /// Calculate the encoded size of a CAN frame
    pub fn encoded_size(frame: &CanFrame) -> usize {
        MIN_CAN_FRAME_SIZE + frame.data().len()
    }

    /// Calculate the encoded size of a CAN FD frame
    ///
    /// Returns the size including padding to the next valid CAN FD data length.
    pub fn encoded_size_fd(frame: &CanFdFrame) -> usize {
        MIN_CANFD_FRAME_SIZE + dlc_to_len(frame.dlc())
    }

    /// Calculate the encoded size of a CAN XL frame
    pub fn encoded_size_xl(frame: &CanXlFrame) -> usize {
        MIN_CANXL_FRAME_SIZE + frame.data_length() as usize
    }
}

/// Convert DLC to actual data length for CAN FD
///
/// CAN FD uses a non-linear DLC mapping for values > 8.
#[inline]
pub fn dlc_to_len(dlc: u8) -> usize {
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

/// Convert data length to DLC for CAN FD
///
/// Returns the minimum DLC that can hold the given data length.
#[inline]
pub fn len_to_dlc(len: usize) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_standard_frame() {
        let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03, 0x04]).unwrap();
        let bytes = CanEncoder::encode(&frame);
        let decoded = CanParser::parse(&bytes).unwrap();

        assert_eq!(frame.id(), decoded.id());
        assert_eq!(frame.is_extended(), decoded.is_extended());
        assert_eq!(frame.data(), decoded.data());
    }

    #[test]
    fn test_encode_decode_extended_frame() {
        let frame = CanFrame::new_extended(0x12345678, &[0xAA, 0xBB, 0xCC]).unwrap();
        let bytes = CanEncoder::encode(&frame);
        let decoded = CanParser::parse(&bytes).unwrap();

        assert_eq!(frame.id(), decoded.id());
        assert!(decoded.is_extended());
        assert_eq!(frame.data(), decoded.data());
    }

    #[test]
    fn test_encode_decode_empty_frame() {
        let frame = CanFrame::new_standard(0x7FF, &[]).unwrap();
        let bytes = CanEncoder::encode(&frame);
        let decoded = CanParser::parse(&bytes).unwrap();

        assert_eq!(frame.id(), decoded.id());
        assert!(decoded.data().is_empty());
    }

    #[test]
    fn test_encode_decode_max_data() {
        let data = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let frame = CanFrame::new_standard(0x100, &data).unwrap();
        let bytes = CanEncoder::encode(&frame);
        let decoded = CanParser::parse(&bytes).unwrap();

        assert_eq!(decoded.data(), &data);
    }

    #[test]
    fn test_encode_decode_fd_frame() {
        let data = vec![0u8; 64];
        let frame = CanFdFrame::new(0x456, false, &data).unwrap();
        let bytes = CanEncoder::encode_fd(&frame);
        let decoded = CanParser::parse_fd(&bytes).unwrap();

        assert_eq!(frame.id(), decoded.id());
        assert_eq!(frame.data().len(), decoded.data().len());
    }

    #[test]
    fn test_encode_decode_fd_with_brs() {
        let mut frame =
            CanFdFrame::new(0x789, true, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]).unwrap();
        frame.set_brs(true);
        let bytes = CanEncoder::encode_fd(&frame);
        let decoded = CanParser::parse_fd(&bytes).unwrap();

        assert!(decoded.is_brs());
        assert!(decoded.is_extended());
    }

    #[test]
    fn test_timestamp_preserved() {
        let mut frame = CanFrame::new_standard(0x100, &[1, 2, 3]).unwrap();
        frame.set_timestamp(123456789);
        let bytes = CanEncoder::encode(&frame);
        let decoded = CanParser::parse(&bytes).unwrap();

        assert_eq!(decoded.timestamp_us(), 123456789);
    }

    #[test]
    fn test_channel_preserved() {
        let mut frame = CanFrame::new_standard(0x100, &[1, 2, 3]).unwrap();
        frame.set_channel(5);
        let bytes = CanEncoder::encode(&frame);
        let decoded = CanParser::parse(&bytes).unwrap();

        assert_eq!(decoded.channel(), 5);
    }

    #[test]
    fn test_parse_too_short() {
        let bytes = [0u8; 10];
        let result = CanParser::parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_dlc() {
        let mut bytes = vec![0u8; 24];
        bytes[4] = 9; // Invalid DLC for CAN 2.0
        let result = CanParser::parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_truncated_data() {
        let frame = CanFrame::new_standard(0x100, &[1, 2, 3, 4]).unwrap();
        let mut bytes = CanEncoder::encode(&frame);
        bytes.truncate(18); // Remove some data bytes
        let result = CanParser::parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_frame() {
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3]).unwrap();
        let bytes = CanEncoder::encode(&frame);
        assert!(CanParser::validate(&bytes).is_ok());
    }

    #[test]
    fn test_validate_invalid_frame() {
        let bytes = [0u8; 5];
        assert!(CanParser::validate(&bytes).is_err());
    }

    #[test]
    fn test_dlc_to_len() {
        assert_eq!(dlc_to_len(0), 0);
        assert_eq!(dlc_to_len(8), 8);
        assert_eq!(dlc_to_len(9), 12);
        assert_eq!(dlc_to_len(10), 16);
        assert_eq!(dlc_to_len(11), 20);
        assert_eq!(dlc_to_len(12), 24);
        assert_eq!(dlc_to_len(13), 32);
        assert_eq!(dlc_to_len(14), 48);
        assert_eq!(dlc_to_len(15), 64);
    }

    #[test]
    fn test_len_to_dlc() {
        assert_eq!(len_to_dlc(0), 0);
        assert_eq!(len_to_dlc(8), 8);
        assert_eq!(len_to_dlc(12), 9);
        assert_eq!(len_to_dlc(16), 10);
        assert_eq!(len_to_dlc(20), 11);
        assert_eq!(len_to_dlc(24), 12);
        assert_eq!(len_to_dlc(32), 13);
        assert_eq!(len_to_dlc(48), 14);
        assert_eq!(len_to_dlc(64), 15);
    }

    #[test]
    fn test_encoded_size() {
        let frame = CanFrame::new_standard(0x100, &[1, 2, 3, 4]).unwrap();
        assert_eq!(CanEncoder::encoded_size(&frame), MIN_CAN_FRAME_SIZE + 4);
    }

    #[test]
    fn test_encoded_size_fd() {
        let frame = CanFdFrame::new(0x100, false, &[0u8; 32]).unwrap();
        // 32 bytes maps to DLC 13, which maps back to 32 bytes
        assert_eq!(
            CanEncoder::encoded_size_fd(&frame),
            MIN_CANFD_FRAME_SIZE + 32
        );

        // Test with non-standard length (10 bytes -> DLC 9 -> 12 bytes padded)
        let frame2 = CanFdFrame::new(0x100, false, &[0u8; 10]).unwrap();
        assert_eq!(
            CanEncoder::encoded_size_fd(&frame2),
            MIN_CANFD_FRAME_SIZE + 12
        );
    }

    // CAN XL Tests

    #[test]
    fn test_encode_decode_xl_frame() {
        let data = vec![0xAA; 256];
        let frame = CanXlFrame::new_full(
            0x123,
            false,
            64,
            10,
            CanXlSdt::Ethernet,
            0xDEADBEEF,
            &data,
        )
        .unwrap();
        let bytes = CanEncoder::encode_xl(&frame);
        let decoded = CanParser::parse_xl(&bytes).unwrap();

        assert_eq!(frame.id(), decoded.id());
        assert_eq!(frame.is_extended(), decoded.is_extended());
        assert_eq!(frame.priority(), decoded.priority());
        assert_eq!(frame.vcid(), decoded.vcid());
        assert_eq!(frame.sdt(), decoded.sdt());
        assert_eq!(frame.acceptance_field(), decoded.acceptance_field());
        assert_eq!(frame.data(), decoded.data());
    }

    #[test]
    fn test_encode_decode_xl_extended() {
        let data = vec![0xBB; 128];
        let frame = CanXlFrame::new(0x12345678, true, &data).unwrap();
        let bytes = CanEncoder::encode_xl(&frame);
        let decoded = CanParser::parse_xl(&bytes).unwrap();

        assert_eq!(frame.id(), decoded.id());
        assert!(decoded.is_extended());
        assert_eq!(frame.data(), decoded.data());
    }

    #[test]
    fn test_encode_decode_xl_max_size() {
        let data = vec![0xCC; MAX_CANXL_DATA_LEN];
        let frame = CanXlFrame::new(0x100, false, &data).unwrap();
        let bytes = CanEncoder::encode_xl(&frame);
        let decoded = CanParser::parse_xl(&bytes).unwrap();

        assert_eq!(decoded.data().len(), MAX_CANXL_DATA_LEN);
    }

    #[test]
    fn test_xl_timestamp_preserved() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        frame.set_timestamp(987654321);
        let bytes = CanEncoder::encode_xl(&frame);
        let decoded = CanParser::parse_xl(&bytes).unwrap();

        assert_eq!(decoded.timestamp_us(), 987654321);
    }

    #[test]
    fn test_xl_channel_preserved() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        frame.set_channel(7);
        let bytes = CanEncoder::encode_xl(&frame);
        let decoded = CanParser::parse_xl(&bytes).unwrap();

        assert_eq!(decoded.channel(), 7);
    }

    #[test]
    fn test_xl_sec_flag_preserved() {
        let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
        frame.set_sec(true);
        let bytes = CanEncoder::encode_xl(&frame);
        let decoded = CanParser::parse_xl(&bytes).unwrap();

        assert!(decoded.is_sec());
    }

    #[test]
    fn test_parse_xl_too_short() {
        let bytes = [0u8; 20];
        let result = CanParser::parse_xl(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_xl_truncated_data() {
        let frame = CanXlFrame::new(0x100, false, &[0u8; 64]).unwrap();
        let mut bytes = CanEncoder::encode_xl(&frame);
        bytes.truncate(50); // Remove some data bytes
        let result = CanParser::parse_xl(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_encoded_size_xl() {
        let frame = CanXlFrame::new(0x100, false, &[0u8; 256]).unwrap();
        assert_eq!(
            CanEncoder::encoded_size_xl(&frame),
            MIN_CANXL_FRAME_SIZE + 256
        );
    }

    #[test]
    fn test_xl_all_sdt_types() {
        for sdt in [
            CanXlSdt::ContentBasedAddressing,
            CanXlSdt::ClassicalCanTunneling,
            CanXlSdt::CanFdTunneling,
            CanXlSdt::Ethernet,
            CanXlSdt::Ipv4,
            CanXlSdt::Ipv6,
            CanXlSdt::Reserved(0x10),
        ] {
            let mut frame = CanXlFrame::new(0x100, false, &[1, 2, 3, 4]).unwrap();
            frame.set_sdt(sdt);
            let bytes = CanEncoder::encode_xl(&frame);
            let decoded = CanParser::parse_xl(&bytes).unwrap();
            assert_eq!(decoded.sdt(), sdt);
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use busmaster_core::MAX_CANFD_DATA_LEN;
    use proptest::prelude::*;

    proptest! {
        /// **Validates: Requirements 1.3.7**
        /// Property: Roundtrip encoding/decoding preserves standard frame data
        #[test]
        fn prop_roundtrip_standard_frame(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CAN_DATA_LEN),
            timestamp in any::<u64>(),
            channel in any::<u8>()
        ) {
            let mut frame = CanFrame::new_standard(id, &data).unwrap();
            frame.set_timestamp(timestamp);
            frame.set_channel(channel);

            let bytes = CanEncoder::encode(&frame);
            let decoded = CanParser::parse(&bytes).unwrap();

            prop_assert_eq!(frame.id(), decoded.id());
            prop_assert_eq!(frame.is_extended(), decoded.is_extended());
            prop_assert_eq!(frame.data(), decoded.data());
            prop_assert_eq!(frame.timestamp_us(), decoded.timestamp_us());
            prop_assert_eq!(frame.channel(), decoded.channel());
        }

        /// **Validates: Requirements 1.3.7**
        /// Property: Roundtrip encoding/decoding preserves extended frame data
        #[test]
        fn prop_roundtrip_extended_frame(
            id in 0u32..=MAX_EXTENDED_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CAN_DATA_LEN),
            timestamp in any::<u64>(),
            channel in any::<u8>()
        ) {
            let mut frame = CanFrame::new_extended(id, &data).unwrap();
            frame.set_timestamp(timestamp);
            frame.set_channel(channel);

            let bytes = CanEncoder::encode(&frame);
            let decoded = CanParser::parse(&bytes).unwrap();

            prop_assert_eq!(frame.id(), decoded.id());
            prop_assert!(decoded.is_extended());
            prop_assert_eq!(frame.data(), decoded.data());
            prop_assert_eq!(frame.timestamp_us(), decoded.timestamp_us());
            prop_assert_eq!(frame.channel(), decoded.channel());
        }

        /// **Validates: Requirements 1.3.7**
        /// Property: Roundtrip encoding/decoding preserves CAN FD frame data
        ///
        /// Note: CAN FD frames are padded to valid lengths (8, 12, 16, 20, 24, 32, 48, 64).
        /// The decoded frame will have the padded length, so we compare only the original
        /// data portion.
        #[test]
        fn prop_roundtrip_fd_frame(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CANFD_DATA_LEN),
            timestamp in any::<u64>(),
            channel in any::<u8>(),
            brs in any::<bool>()
        ) {
            let mut frame = CanFdFrame::new(id, false, &data).unwrap();
            frame.set_timestamp(timestamp);
            frame.set_channel(channel);
            frame.set_brs(brs);

            let bytes = CanEncoder::encode_fd(&frame);
            let decoded = CanParser::parse_fd(&bytes).unwrap();

            prop_assert_eq!(frame.id(), decoded.id());
            // Compare original data portion (decoded may have padding zeros)
            prop_assert_eq!(frame.data(), &decoded.data()[..data.len()]);
            // Verify padding is zeros
            for &byte in &decoded.data()[data.len()..] {
                prop_assert_eq!(byte, 0, "Padding should be zeros");
            }
            prop_assert_eq!(frame.timestamp_us(), decoded.timestamp_us());
            prop_assert_eq!(frame.channel(), decoded.channel());
            prop_assert_eq!(frame.is_brs(), decoded.is_brs());
        }

        /// **Validates: Requirements 1.3.4**
        /// Property: Validation accepts all valid frames
        #[test]
        fn prop_validate_accepts_valid(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CAN_DATA_LEN)
        ) {
            let frame = CanFrame::new_standard(id, &data).unwrap();
            let bytes = CanEncoder::encode(&frame);
            prop_assert!(CanParser::validate(&bytes).is_ok());
        }

        /// **Validates: Requirements 1.3.4**
        /// Property: DLC to length and back is consistent
        #[test]
        fn prop_dlc_len_consistent(dlc in 0u8..=15) {
            let len = dlc_to_len(dlc);
            let back = len_to_dlc(len);
            // For DLC 0-8, should be exact
            // For DLC 9-15, len_to_dlc(dlc_to_len(dlc)) == dlc
            prop_assert_eq!(back, dlc);
        }

        /// **Validates: Requirements 1.3.4**
        /// Property: Encoded size matches actual encoded length
        #[test]
        fn prop_encoded_size_accurate(
            id in 0u32..=MAX_STANDARD_ID,
            data in prop::collection::vec(any::<u8>(), 0..=MAX_CAN_DATA_LEN)
        ) {
            let frame = CanFrame::new_standard(id, &data).unwrap();
            let bytes = CanEncoder::encode(&frame);
            prop_assert_eq!(CanEncoder::encoded_size(&frame), bytes.len());
        }

        /// **Validates: Requirements 6.7.2**
        /// Property: Roundtrip encoding/decoding preserves CAN XL frame data
        #[test]
        fn prop_roundtrip_xl_frame(
            id in 0u32..=MAX_STANDARD_ID,
            // Generate valid CAN XL lengths: 0-48 (any), or word-aligned for >48
            len in prop::strategy::Union::new(vec![
                (0usize..=48).boxed(),
                proptest::strategy::Just(52usize).boxed(),
                proptest::strategy::Just(64usize).boxed(),
                proptest::strategy::Just(128usize).boxed(),
                proptest::strategy::Just(256usize).boxed(),
            ]),
            timestamp in any::<u64>(),
            channel in any::<u8>(),
            priority in any::<u8>(),
            vcid in any::<u8>(),
            acceptance_field in any::<u32>(),
            sec in any::<bool>()
        ) {
            let data: Vec<u8> = vec![0xAA; len];
            let mut frame = CanXlFrame::new_full(
                id,
                false,
                priority,
                vcid,
                CanXlSdt::ContentBasedAddressing,
                acceptance_field,
                &data,
            ).unwrap();
            frame.set_timestamp(timestamp);
            frame.set_channel(channel);
            frame.set_sec(sec);

            let bytes = CanEncoder::encode_xl(&frame);
            let decoded = CanParser::parse_xl(&bytes).unwrap();

            prop_assert_eq!(frame.id(), decoded.id());
            prop_assert_eq!(frame.is_extended(), decoded.is_extended());
            prop_assert_eq!(frame.priority(), decoded.priority());
            prop_assert_eq!(frame.vcid(), decoded.vcid());
            prop_assert_eq!(frame.sdt(), decoded.sdt());
            prop_assert_eq!(frame.acceptance_field(), decoded.acceptance_field());
            prop_assert_eq!(frame.data(), decoded.data());
            prop_assert_eq!(frame.timestamp_us(), decoded.timestamp_us());
            prop_assert_eq!(frame.channel(), decoded.channel());
            prop_assert_eq!(frame.is_sec(), decoded.is_sec());
        }

        /// **Validates: Requirements 6.7.2**
        /// Property: CAN XL encoded size matches actual encoded length
        #[test]
        fn prop_encoded_size_xl_accurate(
            id in 0u32..=MAX_STANDARD_ID,
            len in prop::strategy::Union::new(vec![
                (0usize..=48).boxed(),
                proptest::strategy::Just(52usize).boxed(),
                proptest::strategy::Just(64usize).boxed(),
            ])
        ) {
            let data: Vec<u8> = vec![0xBB; len];
            let frame = CanXlFrame::new(id, false, &data).unwrap();
            let bytes = CanEncoder::encode_xl(&frame);
            prop_assert_eq!(CanEncoder::encoded_size_xl(&frame), bytes.len());
        }
    }
}
