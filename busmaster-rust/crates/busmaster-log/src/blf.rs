//! BLF (Binary Logging Format) Implementation
//!
//! BLF is Vector's binary log format used by CANoe/CANalyzer.
//! It supports compressed storage of CAN, LIN, and other bus data.
//!
//! # Format Overview
//!
//! BLF files consist of:
//! - File header (144 bytes)
//! - Object headers followed by object data
//! - Objects can be compressed with zlib
//!
//! # Example
//!
//! ```no_run
//! use busmaster_log::blf::{BlfWriter, BlfReader};
//! use busmaster_core::CanFrame;
//!
//! // Writing
//! let mut writer = BlfWriter::create("output.blf").unwrap();
//! let frame = CanFrame::new_standard(0x123, &[0x01, 0x02]).unwrap();
//! writer.log_can_frame(&frame, 1000000, 1, true).unwrap();
//! writer.close().unwrap();
//!
//! // Reading
//! let mut reader = BlfReader::open("output.blf").unwrap();
//! while let Some(obj) = reader.next_object().unwrap() {
//!     println!("{:?}", obj);
//! }
//! ```

use busmaster_core::{BusmasterError, CanFrame, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// BLF file signature "LOGG"
pub const BLF_SIGNATURE: [u8; 4] = [b'L', b'O', b'G', b'G'];

/// BLF object signature "LOBJ"
pub const BLF_OBJECT_SIGNATURE: [u8; 4] = [b'L', b'O', b'B', b'J'];

/// BLF file header size
pub const BLF_HEADER_SIZE: usize = 144;

/// BLF object header size
pub const BLF_OBJECT_HEADER_SIZE: usize = 16;

/// BLF Object Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BlfObjectType {
    /// Unknown object type
    Unknown = 0,
    /// CAN message
    CanMessage = 1,
    /// CAN error frame
    CanErrorFrame = 2,
    /// CAN overload frame
    CanOverloadFrame = 3,
    /// CAN statistic
    CanStatistic = 4,
    /// Application trigger
    AppTrigger = 5,
    /// Environment variable
    EnvInteger = 6,
    /// Environment variable
    EnvDouble = 7,
    /// Environment variable
    EnvString = 8,
    /// Environment variable
    EnvData = 9,
    /// Log container (compressed data)
    LogContainer = 10,
    /// LIN message
    LinMessage = 11,
    /// LIN CRC error
    LinCrcError = 12,
    /// LIN DLC info
    LinDlcInfo = 13,
    /// LIN receive error
    LinReceiveError = 14,
    /// LIN send error
    LinSendError = 15,
    /// LIN slave timeout
    LinSlaveTimeout = 16,
    /// LIN scheduler mode change
    LinSchedulerModeChange = 17,
    /// LIN sync error
    LinSyncError = 18,
    /// LIN baudrate event
    LinBaudrateEvent = 19,
    /// LIN sleep mode event
    LinSleepModeEvent = 20,
    /// LIN wakeup event
    LinWakeupEvent = 21,
    /// Most spy message
    MostSpyMessage = 22,
    /// Most control message
    MostCtrlMessage = 23,
    /// Most light lock
    MostLightLock = 24,
    /// Most statistic
    MostStatistic = 25,
    /// FlexRay data
    FlexRayData = 29,
    /// FlexRay sync
    FlexRaySync = 30,
    /// CAN driver error
    CanDriverError = 31,
    /// CAN driver sync
    CanDriverSync = 32,
    /// CAN driver hardware sync
    CanDriverHwSync = 33,
    /// CAN FD message
    CanFdMessage = 100,
    /// CAN FD message 64
    CanFdMessage64 = 101,
    /// CAN FD error frame 64
    CanFdErrorFrame64 = 104,
}

impl BlfObjectType {
    /// Create from raw u32 value
    #[must_use]
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::CanMessage,
            2 => Self::CanErrorFrame,
            3 => Self::CanOverloadFrame,
            4 => Self::CanStatistic,
            5 => Self::AppTrigger,
            6 => Self::EnvInteger,
            7 => Self::EnvDouble,
            8 => Self::EnvString,
            9 => Self::EnvData,
            10 => Self::LogContainer,
            11 => Self::LinMessage,
            12 => Self::LinCrcError,
            13 => Self::LinDlcInfo,
            14 => Self::LinReceiveError,
            15 => Self::LinSendError,
            16 => Self::LinSlaveTimeout,
            17 => Self::LinSchedulerModeChange,
            18 => Self::LinSyncError,
            19 => Self::LinBaudrateEvent,
            20 => Self::LinSleepModeEvent,
            21 => Self::LinWakeupEvent,
            22 => Self::MostSpyMessage,
            23 => Self::MostCtrlMessage,
            24 => Self::MostLightLock,
            25 => Self::MostStatistic,
            29 => Self::FlexRayData,
            30 => Self::FlexRaySync,
            31 => Self::CanDriverError,
            32 => Self::CanDriverSync,
            33 => Self::CanDriverHwSync,
            100 => Self::CanFdMessage,
            101 => Self::CanFdMessage64,
            104 => Self::CanFdErrorFrame64,
            _ => Self::Unknown,
        }
    }
}

/// BLF Object Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlfObjectFlags(pub u32);

impl BlfObjectFlags {
    /// Time stamp is 10 microseconds
    pub const TIME_TEN_MICS: u32 = 0x0000_0001;
    /// Time stamp is 1 nanosecond
    pub const TIME_ONE_NANS: u32 = 0x0000_0002;
}

/// BLF File Header
#[derive(Debug, Clone)]
pub struct BlfFileHeader {
    /// File signature (should be "LOGG")
    pub signature: [u8; 4],
    /// Header size
    pub header_size: u32,
    /// API version (major, minor, build, patch)
    pub api_version: [u8; 4],
    /// Application ID
    pub application_id: u32,
    /// Application version (major, minor, build, patch)
    pub application_version: [u8; 4],
    /// File size
    pub file_size: u64,
    /// Uncompressed file size
    pub uncompressed_size: u64,
    /// Object count
    pub object_count: u32,
    /// Object read count
    pub object_read_count: u32,
    /// Time stamp (Windows FILETIME)
    pub time_stamp: u64,
    /// Time stamp resolution (10us or 1ns)
    pub time_stamp_resolution: u32,
}

impl BlfFileHeader {
    /// Create a new file header with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            signature: BLF_SIGNATURE,
            header_size: BLF_HEADER_SIZE as u32,
            api_version: [3, 9, 6, 0], // Version 3.9.6.0
            application_id: 0,
            application_version: [0, 0, 0, 0],
            file_size: 0,
            uncompressed_size: 0,
            object_count: 0,
            object_read_count: 0,
            time_stamp: 0,
            time_stamp_resolution: BlfObjectFlags::TIME_TEN_MICS,
        }
    }

    /// Write the header to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; BLF_HEADER_SIZE];

        // Signature
        bytes[0..4].copy_from_slice(&self.signature);
        // Header size
        bytes[4..8].copy_from_slice(&self.header_size.to_le_bytes());
        // API version
        bytes[8..12].copy_from_slice(&self.api_version);
        // Application ID
        bytes[12..16].copy_from_slice(&self.application_id.to_le_bytes());
        // Application version
        bytes[16..20].copy_from_slice(&self.application_version);
        // File size
        bytes[20..28].copy_from_slice(&self.file_size.to_le_bytes());
        // Uncompressed size
        bytes[28..36].copy_from_slice(&self.uncompressed_size.to_le_bytes());
        // Object count
        bytes[36..40].copy_from_slice(&self.object_count.to_le_bytes());
        // Object read count
        bytes[40..44].copy_from_slice(&self.object_read_count.to_le_bytes());
        // Time stamp
        bytes[44..52].copy_from_slice(&self.time_stamp.to_le_bytes());
        // Time stamp resolution
        bytes[52..56].copy_from_slice(&self.time_stamp_resolution.to_le_bytes());
        // Rest is reserved/padding

        bytes
    }

    /// Parse header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < BLF_HEADER_SIZE {
            return Err(BusmasterError::Parse {
                message: "BLF header too short".into(),
            });
        }

        let signature: [u8; 4] = bytes[0..4].try_into().unwrap();
        if signature != BLF_SIGNATURE {
            return Err(BusmasterError::Parse {
                message: "Invalid BLF signature".into(),
            });
        }

        Ok(Self {
            signature,
            header_size: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            api_version: bytes[8..12].try_into().unwrap(),
            application_id: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
            application_version: bytes[16..20].try_into().unwrap(),
            file_size: u64::from_le_bytes(bytes[20..28].try_into().unwrap()),
            uncompressed_size: u64::from_le_bytes(bytes[28..36].try_into().unwrap()),
            object_count: u32::from_le_bytes(bytes[36..40].try_into().unwrap()),
            object_read_count: u32::from_le_bytes(bytes[40..44].try_into().unwrap()),
            time_stamp: u64::from_le_bytes(bytes[44..52].try_into().unwrap()),
            time_stamp_resolution: u32::from_le_bytes(bytes[52..56].try_into().unwrap()),
        })
    }
}

impl Default for BlfFileHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// BLF Object Header (base header for all objects)
#[derive(Debug, Clone)]
pub struct BlfObjectHeader {
    /// Object signature (should be "LOBJ")
    pub signature: [u8; 4],
    /// Header size
    pub header_size: u16,
    /// Header version
    pub header_version: u16,
    /// Object size (including header)
    pub object_size: u32,
    /// Object type
    pub object_type: BlfObjectType,
}

impl BlfObjectHeader {
    /// Create a new object header
    #[must_use]
    pub fn new(object_type: BlfObjectType, object_size: u32) -> Self {
        Self {
            signature: BLF_OBJECT_SIGNATURE,
            header_size: BLF_OBJECT_HEADER_SIZE as u16,
            header_version: 1,
            object_size,
            object_type,
        }
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; BLF_OBJECT_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.signature);
        bytes[4..6].copy_from_slice(&self.header_size.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.header_version.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.object_size.to_le_bytes());
        bytes[12..16].copy_from_slice(&(self.object_type as u32).to_le_bytes());
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < BLF_OBJECT_HEADER_SIZE {
            return Err(BusmasterError::Parse {
                message: "BLF object header too short".into(),
            });
        }

        let signature: [u8; 4] = bytes[0..4].try_into().unwrap();
        if signature != BLF_OBJECT_SIGNATURE {
            return Err(BusmasterError::Parse {
                message: "Invalid BLF object signature".into(),
            });
        }

        Ok(Self {
            signature,
            header_size: u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
            header_version: u16::from_le_bytes(bytes[6..8].try_into().unwrap()),
            object_size: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            object_type: BlfObjectType::from_u32(u32::from_le_bytes(
                bytes[12..16].try_into().unwrap(),
            )),
        })
    }
}

/// CAN Message Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanMessageFlags(pub u8);

impl CanMessageFlags {
    /// TX message
    pub const TX: u8 = 0x01;
    /// TX request
    pub const TX_REQ: u8 = 0x02;
    /// Remote frame
    pub const REMOTE: u8 = 0x04;
    /// Extended ID
    pub const EXTENDED: u8 = 0x08;
    /// Wakeup message
    pub const WAKEUP: u8 = 0x10;
    /// NERR active during message
    pub const NERR: u8 = 0x20;
}

/// BLF CAN Message Object
#[derive(Debug, Clone)]
pub struct BlfCanMessage {
    /// Channel (1-based)
    pub channel: u16,
    /// Flags
    pub flags: u8,
    /// DLC
    pub dlc: u8,
    /// CAN ID
    pub id: u32,
    /// Data bytes
    pub data: Vec<u8>,
    /// Timestamp in 10us units
    pub timestamp: u64,
}

impl BlfCanMessage {
    /// Create from a CanFrame
    #[must_use]
    pub fn from_can_frame(frame: &CanFrame, timestamp_us: u64, channel: u16, is_tx: bool) -> Self {
        let mut flags = 0u8;
        if is_tx {
            flags |= CanMessageFlags::TX;
        }
        if frame.is_extended() {
            flags |= CanMessageFlags::EXTENDED;
        }
        if frame.is_rtr() {
            flags |= CanMessageFlags::REMOTE;
        }

        Self {
            channel,
            flags,
            dlc: frame.dlc(),
            id: frame.id(),
            data: frame.data().to_vec(),
            timestamp: timestamp_us / 10, // Convert to 10us units
        }
    }

    /// Convert to CanFrame
    #[must_use]
    pub fn to_can_frame(&self) -> Option<CanFrame> {
        let is_extended = self.flags & CanMessageFlags::EXTENDED != 0;
        // Note: RTR flag is read but not set back since CanFrame doesn't have set_rtr
        // let _is_remote = self.flags & CanMessageFlags::REMOTE != 0;

        let mut frame = if is_extended {
            CanFrame::new_extended(self.id, &self.data).ok()?
        } else {
            CanFrame::new_standard(self.id, &self.data).ok()?
        };

        frame.set_timestamp(self.timestamp * 10); // Convert back to us
        frame.set_channel(self.channel as u8);

        Some(frame)
    }

    /// Object size (header + data)
    #[must_use]
    pub fn object_size(&self) -> u32 {
        // Object header (16) + object header 2 (16) + CAN specific (8) + data (8 for CAN)
        // Total: 48 bytes for standard CAN message
        48
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let obj_size = self.object_size();
        let mut bytes = Vec::with_capacity(obj_size as usize);

        // Object header (16 bytes)
        let header = BlfObjectHeader::new(BlfObjectType::CanMessage, obj_size);
        bytes.extend_from_slice(&header.to_bytes());

        // Object header 2 (16 bytes: flags, client index, object version, timestamp)
        bytes.extend_from_slice(&(0u32).to_le_bytes()); // flags
        bytes.extend_from_slice(&(0u16).to_le_bytes()); // client index
        bytes.extend_from_slice(&(0u16).to_le_bytes()); // object version
        bytes.extend_from_slice(&self.timestamp.to_le_bytes()); // timestamp (8 bytes)

        // CAN message specific (8 bytes: channel, flags, dlc, id)
        bytes.extend_from_slice(&self.channel.to_le_bytes()); // 2 bytes
        bytes.push(self.flags); // 1 byte
        bytes.push(self.dlc); // 1 byte
        bytes.extend_from_slice(&self.id.to_le_bytes()); // 4 bytes

        // Data (8 bytes, padded)
        let mut data = self.data.clone();
        data.resize(8, 0);
        bytes.extend_from_slice(&data);

        bytes
    }
}

/// BLF Object (parsed)
#[derive(Debug, Clone)]
pub enum BlfObject {
    /// CAN message
    CanMessage(BlfCanMessage),
    /// Unknown object type
    Unknown {
        /// Object type
        object_type: BlfObjectType,
        /// Raw data
        data: Vec<u8>,
    },
}

/// BLF Writer
pub struct BlfWriter {
    writer: BufWriter<File>,
    header: BlfFileHeader,
    object_count: u32,
    uncompressed_size: u64,
    buffer: Vec<u8>,
    compress: bool,
}

impl BlfWriter {
    /// Create a new BLF file
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::create_with_options(path, true)
    }

    /// Create a new BLF file with compression option
    pub fn create_with_options<P: AsRef<Path>>(path: P, compress: bool) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let header = BlfFileHeader::new();

        // Write placeholder header (will be updated on close)
        writer.write_all(&header.to_bytes())?;

        Ok(Self {
            writer,
            header,
            object_count: 0,
            uncompressed_size: BLF_HEADER_SIZE as u64,
            buffer: Vec::new(),
            compress,
        })
    }

    /// Log a CAN frame
    pub fn log_can_frame(
        &mut self,
        frame: &CanFrame,
        timestamp_us: u64,
        channel: u16,
        is_tx: bool,
    ) -> Result<()> {
        let msg = BlfCanMessage::from_can_frame(frame, timestamp_us, channel, is_tx);
        let bytes = msg.to_bytes();

        self.uncompressed_size += bytes.len() as u64;
        self.object_count += 1;

        if self.compress {
            self.buffer.extend_from_slice(&bytes);

            // Flush buffer when it gets large enough
            if self.buffer.len() >= 65536 {
                self.flush_buffer()?;
            }
        } else {
            self.writer.write_all(&bytes)?;
        }

        Ok(())
    }

    /// Flush the compression buffer
    fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Compress the buffer
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.buffer)?;
        let compressed = encoder.finish()?;

        // Write log container object
        let container_size = BLF_OBJECT_HEADER_SIZE + 8 + compressed.len();
        let header = BlfObjectHeader::new(BlfObjectType::LogContainer, container_size as u32);

        self.writer.write_all(&header.to_bytes())?;
        // Compression method (2 = zlib)
        self.writer.write_all(&2u16.to_le_bytes())?;
        // Reserved
        self.writer.write_all(&[0u8; 2])?;
        // Uncompressed size
        self.writer
            .write_all(&(self.buffer.len() as u32).to_le_bytes())?;
        // Compressed data
        self.writer.write_all(&compressed)?;

        self.buffer.clear();
        Ok(())
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> Result<()> {
        if self.compress {
            self.flush_buffer()?;
        }
        self.writer.flush()?;
        Ok(())
    }

    /// Close the file and update the header
    pub fn close(mut self) -> Result<()> {
        // Flush any remaining buffered data
        if self.compress && !self.buffer.is_empty() {
            self.flush_buffer()?;
        }

        // Get final file size
        self.writer.flush()?;
        let file_size = self.writer.seek(SeekFrom::End(0))?;

        // Update header
        self.header.file_size = file_size;
        self.header.uncompressed_size = self.uncompressed_size;
        self.header.object_count = self.object_count;

        // Write updated header
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(&self.header.to_bytes())?;
        self.writer.flush()?;

        Ok(())
    }
}

/// BLF Reader
pub struct BlfReader {
    reader: BufReader<File>,
    header: BlfFileHeader,
    decompressed_buffer: Vec<u8>,
    buffer_pos: usize,
}

impl BlfReader {
    /// Open a BLF file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read file header
        let mut header_bytes = vec![0u8; BLF_HEADER_SIZE];
        reader.read_exact(&mut header_bytes)?;
        let header = BlfFileHeader::from_bytes(&header_bytes)?;

        Ok(Self {
            reader,
            header,
            decompressed_buffer: Vec::new(),
            buffer_pos: 0,
        })
    }

    /// Get the file header
    #[must_use]
    pub fn header(&self) -> &BlfFileHeader {
        &self.header
    }

    /// Get the object count
    #[must_use]
    pub fn object_count(&self) -> u32 {
        self.header.object_count
    }

    /// Read the next object
    pub fn next_object(&mut self) -> Result<Option<BlfObject>> {
        // Try to read from decompressed buffer first
        if self.buffer_pos < self.decompressed_buffer.len() {
            return self.read_object_from_buffer();
        }

        // Read object header
        let mut header_bytes = vec![0u8; BLF_OBJECT_HEADER_SIZE];
        match self.reader.read_exact(&mut header_bytes) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        let header = BlfObjectHeader::from_bytes(&header_bytes)?;
        let data_size = header.object_size as usize - BLF_OBJECT_HEADER_SIZE;

        // Read object data
        let mut data = vec![0u8; data_size];
        self.reader.read_exact(&mut data)?;

        // Handle log container (compressed data)
        if header.object_type == BlfObjectType::LogContainer {
            self.decompress_container(&data)?;
            return self.read_object_from_buffer();
        }

        // Parse the object
        self.parse_object(header.object_type, &data)
    }

    /// Decompress a log container
    fn decompress_container(&mut self, data: &[u8]) -> Result<()> {
        if data.len() < 8 {
            return Err(BusmasterError::Parse {
                message: "Log container too short".into(),
            });
        }

        let compression_method = u16::from_le_bytes(data[0..2].try_into().unwrap());
        let uncompressed_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let compressed_data = &data[8..];

        if compression_method == 2 {
            // zlib
            let mut decoder = ZlibDecoder::new(compressed_data);
            self.decompressed_buffer = vec![0u8; uncompressed_size as usize];
            decoder.read_exact(&mut self.decompressed_buffer)?;
            self.buffer_pos = 0;
        } else if compression_method == 0 {
            // No compression
            self.decompressed_buffer = compressed_data.to_vec();
            self.buffer_pos = 0;
        } else {
            return Err(BusmasterError::Parse {
                message: format!("Unknown compression method: {}", compression_method),
            });
        }

        Ok(())
    }

    /// Read an object from the decompressed buffer
    fn read_object_from_buffer(&mut self) -> Result<Option<BlfObject>> {
        if self.buffer_pos + BLF_OBJECT_HEADER_SIZE > self.decompressed_buffer.len() {
            self.decompressed_buffer.clear();
            self.buffer_pos = 0;
            return Ok(None);
        }

        let header_bytes = &self.decompressed_buffer[self.buffer_pos..];
        let header = BlfObjectHeader::from_bytes(header_bytes)?;

        let data_start = self.buffer_pos + BLF_OBJECT_HEADER_SIZE;
        let data_end = self.buffer_pos + header.object_size as usize;

        if data_end > self.decompressed_buffer.len() {
            return Err(BusmasterError::Parse {
                message: "Object extends beyond buffer".into(),
            });
        }

        let data = self.decompressed_buffer[data_start..data_end].to_vec();
        self.buffer_pos = data_end;

        self.parse_object(header.object_type, &data)
    }

    /// Parse an object from its data
    fn parse_object(&self, object_type: BlfObjectType, data: &[u8]) -> Result<Option<BlfObject>> {
        match object_type {
            BlfObjectType::CanMessage => {
                // Data layout (after object header):
                // Object header 2: flags (4) + client index (2) + object version (2) + timestamp (8) = 16 bytes
                // CAN specific: channel (2) + flags (1) + dlc (1) + id (4) = 8 bytes
                // Data: 8 bytes
                // Total: 32 bytes
                if data.len() < 32 {
                    return Err(BusmasterError::Parse {
                        message: format!("CAN message object too short: {} bytes", data.len()),
                    });
                }

                // Object header 2 (16 bytes)
                let timestamp = u64::from_le_bytes(data[8..16].try_into().unwrap());

                // CAN specific (starts at offset 16)
                let channel = u16::from_le_bytes(data[16..18].try_into().unwrap());
                let flags = data[18];
                let dlc = data[19];
                let id = u32::from_le_bytes(data[20..24].try_into().unwrap());

                // Data (starts at offset 24)
                let data_len = dlc.min(8) as usize;
                let msg_data = data[24..24 + data_len].to_vec();

                Ok(Some(BlfObject::CanMessage(BlfCanMessage {
                    channel,
                    flags,
                    dlc,
                    id,
                    data: msg_data,
                    timestamp,
                })))
            },
            _ => Ok(Some(BlfObject::Unknown {
                object_type,
                data: data.to_vec(),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_blf_file_header() {
        let header = BlfFileHeader::new();
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), BLF_HEADER_SIZE);
        assert_eq!(&bytes[0..4], b"LOGG");

        let parsed = BlfFileHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.signature, BLF_SIGNATURE);
        assert_eq!(parsed.header_size, BLF_HEADER_SIZE as u32);
    }

    #[test]
    fn test_blf_object_header() {
        let header = BlfObjectHeader::new(BlfObjectType::CanMessage, 64);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), BLF_OBJECT_HEADER_SIZE);
        assert_eq!(&bytes[0..4], b"LOBJ");

        let parsed = BlfObjectHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.object_type, BlfObjectType::CanMessage);
        assert_eq!(parsed.object_size, 64);
    }

    #[test]
    fn test_blf_can_message_from_frame() {
        let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03, 0x04]).unwrap();
        let msg = BlfCanMessage::from_can_frame(&frame, 1000000, 1, true);

        assert_eq!(msg.channel, 1);
        assert_eq!(msg.id, 0x123);
        assert_eq!(msg.dlc, 4);
        assert!(msg.flags & CanMessageFlags::TX != 0);
        assert!(msg.flags & CanMessageFlags::EXTENDED == 0);
        assert_eq!(msg.timestamp, 100000); // 1000000us / 10
    }

    #[test]
    fn test_blf_can_message_extended() {
        let frame = CanFrame::new_extended(0x12345678, &[0xAA, 0xBB]).unwrap();
        let msg = BlfCanMessage::from_can_frame(&frame, 500000, 2, false);

        assert!(msg.flags & CanMessageFlags::EXTENDED != 0);
        assert!(msg.flags & CanMessageFlags::TX == 0);
        assert_eq!(msg.id, 0x12345678);
    }

    #[test]
    fn test_blf_can_message_roundtrip() {
        let original = CanFrame::new_standard(0x456, &[0x11, 0x22, 0x33]).unwrap();
        let msg = BlfCanMessage::from_can_frame(&original, 2000000, 1, true);
        let recovered = msg.to_can_frame().unwrap();

        assert_eq!(recovered.id(), original.id());
        assert_eq!(recovered.data(), original.data());
        assert_eq!(recovered.is_extended(), original.is_extended());
    }

    #[test]
    fn test_blf_write_read_uncompressed() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write
        {
            let mut writer = BlfWriter::create_with_options(path, false).unwrap();
            let frame1 = CanFrame::new_standard(0x100, &[0x01, 0x02]).unwrap();
            let frame2 = CanFrame::new_standard(0x200, &[0x03, 0x04, 0x05]).unwrap();

            writer.log_can_frame(&frame1, 1000000, 1, true).unwrap();
            writer.log_can_frame(&frame2, 2000000, 1, false).unwrap();
            writer.close().unwrap();
        }

        // Read
        {
            let mut reader = BlfReader::open(path).unwrap();
            assert_eq!(reader.object_count(), 2);

            let obj1 = reader.next_object().unwrap().unwrap();
            match obj1 {
                BlfObject::CanMessage(msg) => {
                    assert_eq!(msg.id, 0x100);
                    assert_eq!(msg.data[..2], [0x01, 0x02]);
                },
                _ => panic!("Expected CAN message"),
            }

            let obj2 = reader.next_object().unwrap().unwrap();
            match obj2 {
                BlfObject::CanMessage(msg) => {
                    assert_eq!(msg.id, 0x200);
                    assert_eq!(msg.data[..3], [0x03, 0x04, 0x05]);
                },
                _ => panic!("Expected CAN message"),
            }

            assert!(reader.next_object().unwrap().is_none());
        }
    }

    #[test]
    fn test_blf_object_type_from_u32() {
        assert_eq!(BlfObjectType::from_u32(1), BlfObjectType::CanMessage);
        assert_eq!(BlfObjectType::from_u32(10), BlfObjectType::LogContainer);
        assert_eq!(BlfObjectType::from_u32(100), BlfObjectType::CanFdMessage);
        assert_eq!(BlfObjectType::from_u32(999), BlfObjectType::Unknown);
    }

    #[test]
    fn test_invalid_blf_signature() {
        let bytes = vec![b'X', b'X', b'X', b'X', 0, 0, 0, 0];
        let result = BlfFileHeader::from_bytes(&bytes);
        assert!(result.is_err());
    }
}
