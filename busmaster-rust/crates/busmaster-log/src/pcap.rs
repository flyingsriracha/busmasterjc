//! PCAP (Packet Capture) Format Implementation
//!
//! PCAP is the standard packet capture format used by Wireshark and tcpdump.
//! This implementation supports logging CAN frames encapsulated in SocketCAN format.
//!
//! # Format Overview
//!
//! PCAP files consist of:
//! - Global header (24 bytes)
//! - Packet records (16-byte header + packet data)
//!
//! # Link Types
//!
//! - `LINKTYPE_CAN_SOCKETCAN` (227): SocketCAN format for CAN frames
//! - `LINKTYPE_ETHERNET` (1): Standard Ethernet frames
//!
//! # Example
//!
//! ```no_run
//! use busmaster_log::pcap::{PcapWriter, PcapReader, LinkType};
//! use busmaster_core::CanFrame;
//!
//! // Writing CAN frames
//! let mut writer = PcapWriter::create("output.pcap", LinkType::CanSocketcan).unwrap();
//! let frame = CanFrame::new_standard(0x123, &[0x01, 0x02]).unwrap();
//! writer.write_can_frame(&frame, 1000000).unwrap();
//! writer.close().unwrap();
//!
//! // Reading
//! let mut reader = PcapReader::open("output.pcap").unwrap();
//! while let Some(packet) = reader.next_packet().unwrap() {
//!     println!("Timestamp: {}s {}us, {} bytes", packet.ts_sec, packet.ts_usec, packet.data.len());
//! }
//! ```

use busmaster_core::{BusmasterError, CanFrame, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// PCAP magic number (microsecond resolution)
pub const PCAP_MAGIC: u32 = 0xa1b2_c3d4;

/// PCAP magic number (nanosecond resolution)
pub const PCAP_MAGIC_NANO: u32 = 0xa1b2_3c4d;

/// PCAP magic number (swapped byte order, microsecond)
pub const PCAP_MAGIC_SWAPPED: u32 = 0xd4c3_b2a1;

/// PCAP magic number (swapped byte order, nanosecond)
pub const PCAP_MAGIC_NANO_SWAPPED: u32 = 0x4d3c_b2a1;

/// PCAP global header size
pub const PCAP_GLOBAL_HEADER_SIZE: usize = 24;

/// PCAP packet header size
pub const PCAP_PACKET_HEADER_SIZE: usize = 16;

/// SocketCAN frame size (CAN ID + DLC + padding + data)
pub const SOCKETCAN_FRAME_SIZE: usize = 16;

/// Link layer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LinkType {
    /// Ethernet (IEEE 802.3)
    Ethernet = 1,
    /// Raw IP
    RawIp = 101,
    /// Linux cooked capture
    LinuxSll = 113,
    /// SocketCAN format
    CanSocketcan = 227,
    /// CAN 2.0B
    Can20B = 190,
    /// CAN FD
    CanFd = 228,
}

impl LinkType {
    /// Create from raw u32 value
    #[must_use]
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Ethernet),
            101 => Some(Self::RawIp),
            113 => Some(Self::LinuxSll),
            227 => Some(Self::CanSocketcan),
            190 => Some(Self::Can20B),
            228 => Some(Self::CanFd),
            _ => None,
        }
    }
}

/// PCAP Global Header
#[derive(Debug, Clone)]
pub struct PcapGlobalHeader {
    /// Magic number (determines byte order and timestamp resolution)
    pub magic: u32,
    /// Major version (usually 2)
    pub version_major: u16,
    /// Minor version (usually 4)
    pub version_minor: u16,
    /// GMT to local correction (usually 0)
    pub thiszone: i32,
    /// Accuracy of timestamps (usually 0)
    pub sigfigs: u32,
    /// Max length of captured packets
    pub snaplen: u32,
    /// Data link type
    pub network: u32,
}

impl PcapGlobalHeader {
    /// Create a new global header
    #[must_use]
    pub fn new(link_type: LinkType) -> Self {
        Self {
            magic: PCAP_MAGIC,
            version_major: 2,
            version_minor: 4,
            thiszone: 0,
            sigfigs: 0,
            snaplen: 65535,
            network: link_type as u32,
        }
    }

    /// Create a new global header with nanosecond resolution
    #[must_use]
    pub fn new_nano(link_type: LinkType) -> Self {
        Self {
            magic: PCAP_MAGIC_NANO,
            version_major: 2,
            version_minor: 4,
            thiszone: 0,
            sigfigs: 0,
            snaplen: 65535,
            network: link_type as u32,
        }
    }

    /// Check if timestamps are in nanoseconds
    #[must_use]
    pub fn is_nanosecond(&self) -> bool {
        self.magic == PCAP_MAGIC_NANO || self.magic == PCAP_MAGIC_NANO_SWAPPED
    }

    /// Check if byte order is swapped
    #[must_use]
    pub fn is_swapped(&self) -> bool {
        self.magic == PCAP_MAGIC_SWAPPED || self.magic == PCAP_MAGIC_NANO_SWAPPED
    }

    /// Get the link type
    #[must_use]
    pub fn link_type(&self) -> Option<LinkType> {
        LinkType::from_u32(self.network)
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PCAP_GLOBAL_HEADER_SIZE);
        bytes.extend_from_slice(&self.magic.to_le_bytes());
        bytes.extend_from_slice(&self.version_major.to_le_bytes());
        bytes.extend_from_slice(&self.version_minor.to_le_bytes());
        bytes.extend_from_slice(&self.thiszone.to_le_bytes());
        bytes.extend_from_slice(&self.sigfigs.to_le_bytes());
        bytes.extend_from_slice(&self.snaplen.to_le_bytes());
        bytes.extend_from_slice(&self.network.to_le_bytes());
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < PCAP_GLOBAL_HEADER_SIZE {
            return Err(BusmasterError::Parse {
                message: "PCAP global header too short".into(),
            });
        }

        let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());

        // Validate magic number
        if magic != PCAP_MAGIC
            && magic != PCAP_MAGIC_NANO
            && magic != PCAP_MAGIC_SWAPPED
            && magic != PCAP_MAGIC_NANO_SWAPPED
        {
            return Err(BusmasterError::Parse {
                message: format!("Invalid PCAP magic number: 0x{:08x}", magic),
            });
        }

        let swapped = magic == PCAP_MAGIC_SWAPPED || magic == PCAP_MAGIC_NANO_SWAPPED;

        let read_u16 = |offset: usize| -> u16 {
            let val = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
            if swapped {
                val.swap_bytes()
            } else {
                val
            }
        };

        let read_u32 = |offset: usize| -> u32 {
            let val = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            if swapped {
                val.swap_bytes()
            } else {
                val
            }
        };

        let read_i32 = |offset: usize| -> i32 {
            let val = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            if swapped {
                val.swap_bytes()
            } else {
                val
            }
        };

        Ok(Self {
            magic,
            version_major: read_u16(4),
            version_minor: read_u16(6),
            thiszone: read_i32(8),
            sigfigs: read_u32(12),
            snaplen: read_u32(16),
            network: read_u32(20),
        })
    }
}

impl Default for PcapGlobalHeader {
    fn default() -> Self {
        Self::new(LinkType::CanSocketcan)
    }
}

/// PCAP Packet Header
#[derive(Debug, Clone)]
pub struct PcapPacketHeader {
    /// Timestamp seconds
    pub ts_sec: u32,
    /// Timestamp microseconds (or nanoseconds)
    pub ts_usec: u32,
    /// Number of bytes of packet data saved
    pub incl_len: u32,
    /// Actual length of packet
    pub orig_len: u32,
}

impl PcapPacketHeader {
    /// Create a new packet header
    #[must_use]
    pub fn new(timestamp_us: u64, length: u32) -> Self {
        Self {
            ts_sec: (timestamp_us / 1_000_000) as u32,
            ts_usec: (timestamp_us % 1_000_000) as u32,
            incl_len: length,
            orig_len: length,
        }
    }

    /// Create a new packet header with nanosecond timestamp
    #[must_use]
    pub fn new_nano(timestamp_ns: u64, length: u32) -> Self {
        Self {
            ts_sec: (timestamp_ns / 1_000_000_000) as u32,
            ts_usec: (timestamp_ns % 1_000_000_000) as u32,
            incl_len: length,
            orig_len: length,
        }
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PCAP_PACKET_HEADER_SIZE);
        bytes.extend_from_slice(&self.ts_sec.to_le_bytes());
        bytes.extend_from_slice(&self.ts_usec.to_le_bytes());
        bytes.extend_from_slice(&self.incl_len.to_le_bytes());
        bytes.extend_from_slice(&self.orig_len.to_le_bytes());
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8], swapped: bool) -> Result<Self> {
        if bytes.len() < PCAP_PACKET_HEADER_SIZE {
            return Err(BusmasterError::Parse {
                message: "PCAP packet header too short".into(),
            });
        }

        let read_u32 = |offset: usize| -> u32 {
            let val = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            if swapped {
                val.swap_bytes()
            } else {
                val
            }
        };

        Ok(Self {
            ts_sec: read_u32(0),
            ts_usec: read_u32(4),
            incl_len: read_u32(8),
            orig_len: read_u32(12),
        })
    }
}

/// PCAP Packet (header + data)
#[derive(Debug, Clone)]
pub struct PcapPacket {
    /// Timestamp seconds
    pub ts_sec: u32,
    /// Timestamp microseconds (or nanoseconds)
    pub ts_usec: u32,
    /// Packet data
    pub data: Vec<u8>,
}

impl PcapPacket {
    /// Get timestamp in microseconds
    #[must_use]
    pub fn timestamp_us(&self) -> u64 {
        u64::from(self.ts_sec) * 1_000_000 + u64::from(self.ts_usec)
    }
}


/// SocketCAN frame format
///
/// This is the format used by Linux SocketCAN and PCAP link type 227.
#[derive(Debug, Clone)]
pub struct SocketCanFrame {
    /// CAN ID with flags (bit 31 = error, bit 30 = RTR, bit 29 = extended)
    pub can_id: u32,
    /// Data length code
    pub can_dlc: u8,
    /// Padding
    pub pad: u8,
    /// Reserved
    pub res0: u8,
    /// Reserved
    pub res1: u8,
    /// Data bytes (up to 8)
    pub data: [u8; 8],
}

impl SocketCanFrame {
    /// Extended ID flag
    pub const EFF_FLAG: u32 = 0x8000_0000;
    /// RTR flag
    pub const RTR_FLAG: u32 = 0x4000_0000;
    /// Error flag
    pub const ERR_FLAG: u32 = 0x2000_0000;
    /// Standard ID mask
    pub const SFF_MASK: u32 = 0x0000_07FF;
    /// Extended ID mask
    pub const EFF_MASK: u32 = 0x1FFF_FFFF;

    /// Create from a CanFrame
    #[must_use]
    pub fn from_can_frame(frame: &CanFrame) -> Self {
        let mut can_id = frame.id();
        if frame.is_extended() {
            can_id |= Self::EFF_FLAG;
        }
        if frame.is_rtr() {
            can_id |= Self::RTR_FLAG;
        }

        let mut data = [0u8; 8];
        let len = frame.data().len().min(8);
        data[..len].copy_from_slice(&frame.data()[..len]);

        Self {
            can_id,
            can_dlc: frame.dlc(),
            pad: 0,
            res0: 0,
            res1: 0,
            data,
        }
    }

    /// Convert to CanFrame
    #[must_use]
    pub fn to_can_frame(&self) -> Option<CanFrame> {
        let is_extended = self.can_id & Self::EFF_FLAG != 0;
        let id = if is_extended {
            self.can_id & Self::EFF_MASK
        } else {
            self.can_id & Self::SFF_MASK
        };

        let data_len = (self.can_dlc as usize).min(8);
        let data = &self.data[..data_len];

        if is_extended {
            CanFrame::new_extended(id, data).ok()
        } else {
            CanFrame::new_standard(id, data).ok()
        }
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SOCKETCAN_FRAME_SIZE);
        bytes.extend_from_slice(&self.can_id.to_le_bytes());
        bytes.push(self.can_dlc);
        bytes.push(self.pad);
        bytes.push(self.res0);
        bytes.push(self.res1);
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < SOCKETCAN_FRAME_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "SocketCAN frame too short: {} bytes, expected {}",
                    bytes.len(),
                    SOCKETCAN_FRAME_SIZE
                ),
            });
        }

        let can_id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let can_dlc = bytes[4];
        let pad = bytes[5];
        let res0 = bytes[6];
        let res1 = bytes[7];
        let mut data = [0u8; 8];
        data.copy_from_slice(&bytes[8..16]);

        Ok(Self {
            can_id,
            can_dlc,
            pad,
            res0,
            res1,
            data,
        })
    }
}

/// PCAP Writer
pub struct PcapWriter {
    writer: BufWriter<File>,
    header: PcapGlobalHeader,
    packet_count: u64,
}

impl PcapWriter {
    /// Create a new PCAP file with microsecond timestamps
    pub fn create<P: AsRef<Path>>(path: P, link_type: LinkType) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let header = PcapGlobalHeader::new(link_type);
        writer.write_all(&header.to_bytes())?;

        Ok(Self {
            writer,
            header,
            packet_count: 0,
        })
    }

    /// Create a new PCAP file with nanosecond timestamps
    pub fn create_nano<P: AsRef<Path>>(path: P, link_type: LinkType) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let header = PcapGlobalHeader::new_nano(link_type);
        writer.write_all(&header.to_bytes())?;

        Ok(Self {
            writer,
            header,
            packet_count: 0,
        })
    }

    /// Get the link type
    #[must_use]
    pub fn link_type(&self) -> Option<LinkType> {
        self.header.link_type()
    }

    /// Get the packet count
    #[must_use]
    pub fn packet_count(&self) -> u64 {
        self.packet_count
    }

    /// Write a raw packet
    pub fn write_packet(&mut self, timestamp_us: u64, data: &[u8]) -> Result<()> {
        let pkt_header = if self.header.is_nanosecond() {
            PcapPacketHeader::new_nano(timestamp_us * 1000, data.len() as u32)
        } else {
            PcapPacketHeader::new(timestamp_us, data.len() as u32)
        };

        self.writer.write_all(&pkt_header.to_bytes())?;
        self.writer.write_all(data)?;
        self.packet_count += 1;

        Ok(())
    }

    /// Write a CAN frame (SocketCAN format)
    pub fn write_can_frame(&mut self, frame: &CanFrame, timestamp_us: u64) -> Result<()> {
        if self.header.link_type() != Some(LinkType::CanSocketcan) {
            return Err(BusmasterError::Config {
                message: "PCAP file is not configured for SocketCAN format".into(),
            });
        }

        let socketcan = SocketCanFrame::from_can_frame(frame);
        self.write_packet(timestamp_us, &socketcan.to_bytes())
    }

    /// Write raw Ethernet frame
    pub fn write_ethernet_frame(&mut self, timestamp_us: u64, data: &[u8]) -> Result<()> {
        if self.header.link_type() != Some(LinkType::Ethernet) {
            return Err(BusmasterError::Config {
                message: "PCAP file is not configured for Ethernet format".into(),
            });
        }

        self.write_packet(timestamp_us, data)
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    /// Close the file
    pub fn close(mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

/// PCAP Reader
pub struct PcapReader {
    reader: BufReader<File>,
    header: PcapGlobalHeader,
}

impl PcapReader {
    /// Open a PCAP file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read global header
        let mut header_bytes = vec![0u8; PCAP_GLOBAL_HEADER_SIZE];
        reader.read_exact(&mut header_bytes)?;
        let header = PcapGlobalHeader::from_bytes(&header_bytes)?;

        Ok(Self { reader, header })
    }

    /// Get the global header
    #[must_use]
    pub fn header(&self) -> &PcapGlobalHeader {
        &self.header
    }

    /// Get the link type
    #[must_use]
    pub fn link_type(&self) -> Option<LinkType> {
        self.header.link_type()
    }

    /// Check if timestamps are in nanoseconds
    #[must_use]
    pub fn is_nanosecond(&self) -> bool {
        self.header.is_nanosecond()
    }

    /// Read the next packet
    pub fn next_packet(&mut self) -> Result<Option<PcapPacket>> {
        // Read packet header
        let mut header_bytes = vec![0u8; PCAP_PACKET_HEADER_SIZE];
        match self.reader.read_exact(&mut header_bytes) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        let pkt_header = PcapPacketHeader::from_bytes(&header_bytes, self.header.is_swapped())?;

        // Read packet data
        let mut data = vec![0u8; pkt_header.incl_len as usize];
        self.reader.read_exact(&mut data)?;

        Ok(Some(PcapPacket {
            ts_sec: pkt_header.ts_sec,
            ts_usec: pkt_header.ts_usec,
            data,
        }))
    }

    /// Read the next CAN frame (if link type is SocketCAN)
    pub fn next_can_frame(&mut self) -> Result<Option<(u64, CanFrame)>> {
        if self.header.link_type() != Some(LinkType::CanSocketcan) {
            return Err(BusmasterError::Config {
                message: "PCAP file is not SocketCAN format".into(),
            });
        }

        match self.next_packet()? {
            Some(packet) => {
                let socketcan = SocketCanFrame::from_bytes(&packet.data)?;
                match socketcan.to_can_frame() {
                    Some(mut frame) => {
                        let timestamp = packet.timestamp_us();
                        frame.set_timestamp(timestamp);
                        Ok(Some((timestamp, frame)))
                    },
                    None => Err(BusmasterError::Parse {
                        message: "Failed to convert SocketCAN frame to CanFrame".into(),
                    }),
                }
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pcap_global_header() {
        let header = PcapGlobalHeader::new(LinkType::CanSocketcan);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), PCAP_GLOBAL_HEADER_SIZE);

        let parsed = PcapGlobalHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.magic, PCAP_MAGIC);
        assert_eq!(parsed.version_major, 2);
        assert_eq!(parsed.version_minor, 4);
        assert_eq!(parsed.network, LinkType::CanSocketcan as u32);
        assert!(!parsed.is_nanosecond());
        assert!(!parsed.is_swapped());
    }

    #[test]
    fn test_pcap_global_header_nano() {
        let header = PcapGlobalHeader::new_nano(LinkType::Ethernet);
        assert!(header.is_nanosecond());
        assert_eq!(header.link_type(), Some(LinkType::Ethernet));
    }

    #[test]
    fn test_pcap_packet_header() {
        let header = PcapPacketHeader::new(1_500_000, 64);
        assert_eq!(header.ts_sec, 1);
        assert_eq!(header.ts_usec, 500_000);
        assert_eq!(header.incl_len, 64);
        assert_eq!(header.orig_len, 64);

        let bytes = header.to_bytes();
        let parsed = PcapPacketHeader::from_bytes(&bytes, false).unwrap();
        assert_eq!(parsed.ts_sec, 1);
        assert_eq!(parsed.ts_usec, 500_000);
    }

    #[test]
    fn test_socketcan_frame_standard() {
        let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03]).unwrap();
        let socketcan = SocketCanFrame::from_can_frame(&frame);

        assert_eq!(socketcan.can_id, 0x123);
        assert_eq!(socketcan.can_dlc, 3);
        assert_eq!(&socketcan.data[..3], &[0x01, 0x02, 0x03]);

        let recovered = socketcan.to_can_frame().unwrap();
        assert_eq!(recovered.id(), 0x123);
        assert!(!recovered.is_extended());
        assert_eq!(recovered.data(), &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_socketcan_frame_extended() {
        let frame = CanFrame::new_extended(0x12345678, &[0xAA, 0xBB]).unwrap();
        let socketcan = SocketCanFrame::from_can_frame(&frame);

        assert_eq!(socketcan.can_id & SocketCanFrame::EFF_MASK, 0x12345678);
        assert!(socketcan.can_id & SocketCanFrame::EFF_FLAG != 0);

        let recovered = socketcan.to_can_frame().unwrap();
        assert_eq!(recovered.id(), 0x12345678);
        assert!(recovered.is_extended());
    }

    #[test]
    fn test_socketcan_frame_roundtrip() {
        let original = CanFrame::new_standard(0x456, &[0x11, 0x22, 0x33, 0x44]).unwrap();
        let socketcan = SocketCanFrame::from_can_frame(&original);
        let bytes = socketcan.to_bytes();
        let parsed = SocketCanFrame::from_bytes(&bytes).unwrap();
        let recovered = parsed.to_can_frame().unwrap();

        assert_eq!(recovered.id(), original.id());
        assert_eq!(recovered.data(), original.data());
    }

    #[test]
    fn test_pcap_write_read_can() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write
        {
            let mut writer = PcapWriter::create(path, LinkType::CanSocketcan).unwrap();
            let frame1 = CanFrame::new_standard(0x100, &[0x01, 0x02]).unwrap();
            let frame2 = CanFrame::new_extended(0x12345, &[0x03, 0x04, 0x05]).unwrap();

            writer.write_can_frame(&frame1, 1_000_000).unwrap();
            writer.write_can_frame(&frame2, 2_000_000).unwrap();
            writer.close().unwrap();
        }

        // Read
        {
            let mut reader = PcapReader::open(path).unwrap();
            assert_eq!(reader.link_type(), Some(LinkType::CanSocketcan));

            let (ts1, frame1) = reader.next_can_frame().unwrap().unwrap();
            assert_eq!(ts1, 1_000_000);
            assert_eq!(frame1.id(), 0x100);
            assert!(!frame1.is_extended());

            let (ts2, frame2) = reader.next_can_frame().unwrap().unwrap();
            assert_eq!(ts2, 2_000_000);
            assert_eq!(frame2.id(), 0x12345);
            assert!(frame2.is_extended());

            assert!(reader.next_can_frame().unwrap().is_none());
        }
    }

    #[test]
    fn test_pcap_write_read_raw() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write
        {
            let mut writer = PcapWriter::create(path, LinkType::Ethernet).unwrap();
            writer.write_packet(1_000_000, &[0x01, 0x02, 0x03]).unwrap();
            writer.write_packet(2_000_000, &[0x04, 0x05]).unwrap();
            assert_eq!(writer.packet_count(), 2);
            writer.close().unwrap();
        }

        // Read
        {
            let mut reader = PcapReader::open(path).unwrap();
            assert_eq!(reader.link_type(), Some(LinkType::Ethernet));

            let pkt1 = reader.next_packet().unwrap().unwrap();
            assert_eq!(pkt1.ts_sec, 1);
            assert_eq!(pkt1.ts_usec, 0);
            assert_eq!(pkt1.data, vec![0x01, 0x02, 0x03]);

            let pkt2 = reader.next_packet().unwrap().unwrap();
            assert_eq!(pkt2.ts_sec, 2);
            assert_eq!(pkt2.data, vec![0x04, 0x05]);

            assert!(reader.next_packet().unwrap().is_none());
        }
    }

    #[test]
    fn test_invalid_pcap_magic() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = PcapGlobalHeader::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_link_type_from_u32() {
        assert_eq!(LinkType::from_u32(1), Some(LinkType::Ethernet));
        assert_eq!(LinkType::from_u32(227), Some(LinkType::CanSocketcan));
        assert_eq!(LinkType::from_u32(228), Some(LinkType::CanFd));
        assert_eq!(LinkType::from_u32(999), None);
    }

    #[test]
    fn test_pcap_packet_timestamp() {
        let packet = PcapPacket {
            ts_sec: 5,
            ts_usec: 500_000,
            data: vec![],
        };
        assert_eq!(packet.timestamp_us(), 5_500_000);
    }
}
