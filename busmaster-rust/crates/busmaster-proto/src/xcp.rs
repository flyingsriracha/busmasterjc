//! XCP (Universal Measurement and Calibration Protocol) Implementation
//!
//! XCP is defined in ASAM and provides measurement and calibration
//! capabilities for ECU development and testing.
//!
//! # Protocol Overview
//!
//! XCP supports:
//! - Command/Response communication
//! - DAQ (Data Acquisition) for measurement
//! - STIM (Stimulation) for bypassing
//! - Multiple transport layers (CAN, Ethernet, USB, etc.)
//!
//! # Example
//!
//! ```
//! use busmaster_proto::xcp::{XcpCommand, XcpCommandCode};
//!
//! // Create a CONNECT command
//! let cmd = XcpCommand::connect(0x00);
//! let bytes = cmd.to_bytes();
//! assert_eq!(bytes[0], XcpCommandCode::Connect as u8);
//! ```

use busmaster_core::{BusmasterError, Result};
use serde::{Deserialize, Serialize};

/// XCP command codes (CTOs - Command Transfer Objects)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum XcpCommandCode {
    // Standard commands
    /// Connect to slave
    Connect = 0xFF,
    /// Disconnect from slave
    Disconnect = 0xFE,
    /// Get status
    GetStatus = 0xFD,
    /// Synchronize
    Synch = 0xFC,
    /// Get communication mode info
    GetCommModeInfo = 0xFB,
    /// Get ID
    GetId = 0xFA,
    /// Set request
    SetRequest = 0xF9,
    /// Get seed
    GetSeed = 0xF8,
    /// Unlock
    Unlock = 0xF7,
    /// Set MTA (Memory Transfer Address)
    SetMta = 0xF6,
    /// Upload
    Upload = 0xF5,
    /// Short upload
    ShortUpload = 0xF4,
    /// Build checksum
    BuildChecksum = 0xF3,
    /// Transport layer command
    TransportLayerCmd = 0xF2,
    /// User command
    UserCmd = 0xF1,

    // Calibration commands
    /// Download
    Download = 0xF0,
    /// Download next
    DownloadNext = 0xEF,
    /// Download max
    DownloadMax = 0xEE,
    /// Short download
    ShortDownload = 0xED,
    /// Modify bits
    ModifyBits = 0xEC,

    // Page switching commands
    /// Set calibration page
    SetCalPage = 0xEB,
    /// Get calibration page
    GetCalPage = 0xEA,
    /// Get PAG processor info
    GetPagProcessorInfo = 0xE9,
    /// Get segment info
    GetSegmentInfo = 0xE8,
    /// Get page info
    GetPageInfo = 0xE7,
    /// Set segment mode
    SetSegmentMode = 0xE6,
    /// Get segment mode
    GetSegmentMode = 0xE5,
    /// Copy calibration page
    CopyCalPage = 0xE4,

    // DAQ commands
    /// Clear DAQ list
    ClearDaqList = 0xE3,
    /// Set DAQ pointer
    SetDaqPtr = 0xE2,
    /// Write DAQ
    WriteDaq = 0xE1,
    /// Set DAQ list mode
    SetDaqListMode = 0xE0,
    /// Get DAQ list mode
    GetDaqListMode = 0xDF,
    /// Start/stop DAQ list
    StartStopDaqList = 0xDE,
    /// Start/stop synchronized
    StartStopSynch = 0xDD,
    /// Get DAQ clock
    GetDaqClock = 0xDC,
    /// Read DAQ
    ReadDaq = 0xDB,
    /// Get DAQ processor info
    GetDaqProcessorInfo = 0xDA,
    /// Get DAQ resolution info
    GetDaqResolutionInfo = 0xD9,
    /// Get DAQ list info
    GetDaqListInfo = 0xD8,
    /// Get DAQ event info
    GetDaqEventInfo = 0xD7,
    /// Free DAQ
    FreeDaq = 0xD6,
    /// Alloc DAQ
    AllocDaq = 0xD5,
    /// Alloc ODT
    AllocOdt = 0xD4,
    /// Alloc ODT entry
    AllocOdtEntry = 0xD3,

    // Programming commands
    /// Program start
    ProgramStart = 0xD2,
    /// Program clear
    ProgramClear = 0xD1,
    /// Program
    Program = 0xD0,
    /// Program reset
    ProgramReset = 0xCF,
    /// Get PGM processor info
    GetPgmProcessorInfo = 0xCE,
    /// Get sector info
    GetSectorInfo = 0xCD,
    /// Program prepare
    ProgramPrepare = 0xCC,
    /// Program format
    ProgramFormat = 0xCB,
    /// Program next
    ProgramNext = 0xCA,
    /// Program max
    ProgramMax = 0xC9,
    /// Program verify
    ProgramVerify = 0xC8,
}

impl XcpCommandCode {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0xFF => Some(Self::Connect),
            0xFE => Some(Self::Disconnect),
            0xFD => Some(Self::GetStatus),
            0xFC => Some(Self::Synch),
            0xFB => Some(Self::GetCommModeInfo),
            0xFA => Some(Self::GetId),
            0xF9 => Some(Self::SetRequest),
            0xF8 => Some(Self::GetSeed),
            0xF7 => Some(Self::Unlock),
            0xF6 => Some(Self::SetMta),
            0xF5 => Some(Self::Upload),
            0xF4 => Some(Self::ShortUpload),
            0xF3 => Some(Self::BuildChecksum),
            0xF2 => Some(Self::TransportLayerCmd),
            0xF1 => Some(Self::UserCmd),
            0xF0 => Some(Self::Download),
            0xEF => Some(Self::DownloadNext),
            0xEE => Some(Self::DownloadMax),
            0xED => Some(Self::ShortDownload),
            0xEC => Some(Self::ModifyBits),
            0xEB => Some(Self::SetCalPage),
            0xEA => Some(Self::GetCalPage),
            0xE9 => Some(Self::GetPagProcessorInfo),
            0xE8 => Some(Self::GetSegmentInfo),
            0xE7 => Some(Self::GetPageInfo),
            0xE6 => Some(Self::SetSegmentMode),
            0xE5 => Some(Self::GetSegmentMode),
            0xE4 => Some(Self::CopyCalPage),
            0xE3 => Some(Self::ClearDaqList),
            0xE2 => Some(Self::SetDaqPtr),
            0xE1 => Some(Self::WriteDaq),
            0xE0 => Some(Self::SetDaqListMode),
            0xDF => Some(Self::GetDaqListMode),
            0xDE => Some(Self::StartStopDaqList),
            0xDD => Some(Self::StartStopSynch),
            0xDC => Some(Self::GetDaqClock),
            0xDB => Some(Self::ReadDaq),
            0xDA => Some(Self::GetDaqProcessorInfo),
            0xD9 => Some(Self::GetDaqResolutionInfo),
            0xD8 => Some(Self::GetDaqListInfo),
            0xD7 => Some(Self::GetDaqEventInfo),
            0xD6 => Some(Self::FreeDaq),
            0xD5 => Some(Self::AllocDaq),
            0xD4 => Some(Self::AllocOdt),
            0xD3 => Some(Self::AllocOdtEntry),
            0xD2 => Some(Self::ProgramStart),
            0xD1 => Some(Self::ProgramClear),
            0xD0 => Some(Self::Program),
            0xCF => Some(Self::ProgramReset),
            0xCE => Some(Self::GetPgmProcessorInfo),
            0xCD => Some(Self::GetSectorInfo),
            0xCC => Some(Self::ProgramPrepare),
            0xCB => Some(Self::ProgramFormat),
            0xCA => Some(Self::ProgramNext),
            0xC9 => Some(Self::ProgramMax),
            0xC8 => Some(Self::ProgramVerify),
            _ => None,
        }
    }
}

/// XCP response packet ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum XcpResponsePid {
    /// Positive response
    Response = 0xFF,
    /// Error response
    Error = 0xFE,
    /// Event packet
    Event = 0xFD,
    /// Service request
    ServiceRequest = 0xFC,
}

impl XcpResponsePid {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0xFF => Some(Self::Response),
            0xFE => Some(Self::Error),
            0xFD => Some(Self::Event),
            0xFC => Some(Self::ServiceRequest),
            _ => None,
        }
    }
}

/// XCP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum XcpErrorCode {
    /// Command processor synchronization
    CmdSynch = 0x00,
    /// Command busy
    CmdBusy = 0x10,
    /// DAQ active
    DaqActive = 0x11,
    /// PGM active
    PgmActive = 0x12,
    /// Unknown command
    CmdUnknown = 0x20,
    /// Command syntax error
    CmdSyntax = 0x21,
    /// Out of range
    OutOfRange = 0x22,
    /// Write protected
    WriteProtected = 0x23,
    /// Access denied
    AccessDenied = 0x24,
    /// Access locked
    AccessLocked = 0x25,
    /// Page not valid
    PageNotValid = 0x26,
    /// Mode not valid
    ModeNotValid = 0x27,
    /// Segment not valid
    SegmentNotValid = 0x28,
    /// Sequence error
    Sequence = 0x29,
    /// DAQ config error
    DaqConfig = 0x2A,
    /// Memory overflow
    MemoryOverflow = 0x30,
    /// Generic error
    Generic = 0x31,
    /// Verify error
    Verify = 0x32,
    /// Resource temporary not available
    ResourceTempNotAvailable = 0x33,
    /// Subcmd unknown
    SubcmdUnknown = 0x34,
}

impl XcpErrorCode {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::CmdSynch),
            0x10 => Some(Self::CmdBusy),
            0x11 => Some(Self::DaqActive),
            0x12 => Some(Self::PgmActive),
            0x20 => Some(Self::CmdUnknown),
            0x21 => Some(Self::CmdSyntax),
            0x22 => Some(Self::OutOfRange),
            0x23 => Some(Self::WriteProtected),
            0x24 => Some(Self::AccessDenied),
            0x25 => Some(Self::AccessLocked),
            0x26 => Some(Self::PageNotValid),
            0x27 => Some(Self::ModeNotValid),
            0x28 => Some(Self::SegmentNotValid),
            0x29 => Some(Self::Sequence),
            0x2A => Some(Self::DaqConfig),
            0x30 => Some(Self::MemoryOverflow),
            0x31 => Some(Self::Generic),
            0x32 => Some(Self::Verify),
            0x33 => Some(Self::ResourceTempNotAvailable),
            0x34 => Some(Self::SubcmdUnknown),
            _ => None,
        }
    }
}

/// XCP Command (CTO - Command Transfer Object)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XcpCommand {
    /// Command code
    pub code: XcpCommandCode,
    /// Command data
    pub data: Vec<u8>,
}

impl XcpCommand {
    /// Create a new XCP command
    #[must_use]
    pub fn new(code: XcpCommandCode, data: Vec<u8>) -> Self {
        Self { code, data }
    }

    /// Create a CONNECT command
    #[must_use]
    pub fn connect(mode: u8) -> Self {
        Self::new(XcpCommandCode::Connect, vec![mode])
    }

    /// Create a DISCONNECT command
    #[must_use]
    pub fn disconnect() -> Self {
        Self::new(XcpCommandCode::Disconnect, vec![])
    }

    /// Create a GET_STATUS command
    #[must_use]
    pub fn get_status() -> Self {
        Self::new(XcpCommandCode::GetStatus, vec![])
    }

    /// Create a SYNCH command
    #[must_use]
    pub fn synch() -> Self {
        Self::new(XcpCommandCode::Synch, vec![])
    }

    /// Create a GET_COMM_MODE_INFO command
    #[must_use]
    pub fn get_comm_mode_info() -> Self {
        Self::new(XcpCommandCode::GetCommModeInfo, vec![])
    }

    /// Create a GET_ID command
    #[must_use]
    pub fn get_id(id_type: u8) -> Self {
        Self::new(XcpCommandCode::GetId, vec![id_type])
    }

    /// Create a SET_MTA command
    #[must_use]
    pub fn set_mta(address_extension: u8, address: u32) -> Self {
        let mut data = vec![0, 0, address_extension];
        data.extend_from_slice(&address.to_le_bytes());
        Self::new(XcpCommandCode::SetMta, data)
    }

    /// Create an UPLOAD command
    #[must_use]
    pub fn upload(num_elements: u8) -> Self {
        Self::new(XcpCommandCode::Upload, vec![num_elements])
    }

    /// Create a SHORT_UPLOAD command
    #[must_use]
    pub fn short_upload(num_elements: u8, address_extension: u8, address: u32) -> Self {
        let mut data = vec![num_elements, 0, address_extension];
        data.extend_from_slice(&address.to_le_bytes());
        Self::new(XcpCommandCode::ShortUpload, data)
    }

    /// Create a DOWNLOAD command
    #[must_use]
    pub fn download(data_elements: &[u8]) -> Self {
        let mut data = vec![data_elements.len() as u8];
        data.extend_from_slice(data_elements);
        Self::new(XcpCommandCode::Download, data)
    }

    /// Create a SHORT_DOWNLOAD command
    #[must_use]
    pub fn short_download(address_extension: u8, address: u32, data_elements: &[u8]) -> Self {
        let mut data = vec![data_elements.len() as u8, 0, address_extension];
        data.extend_from_slice(&address.to_le_bytes());
        data.extend_from_slice(data_elements);
        Self::new(XcpCommandCode::ShortDownload, data)
    }

    /// Encode command to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.code as u8];
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Parse command from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(BusmasterError::Parse {
                message: "XCP command too short".into(),
            });
        }

        let code = XcpCommandCode::from_u8(bytes[0]).ok_or_else(|| BusmasterError::Parse {
            message: format!("Unknown XCP command code: 0x{:02X}", bytes[0]),
        })?;

        let data = bytes[1..].to_vec();
        Ok(Self { code, data })
    }
}

/// XCP Response (DTO - Data Transfer Object)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XcpResponse {
    /// Response packet ID
    pub pid: XcpResponsePid,
    /// Response data
    pub data: Vec<u8>,
}

impl XcpResponse {
    /// Create a new XCP response
    #[must_use]
    pub fn new(pid: XcpResponsePid, data: Vec<u8>) -> Self {
        Self { pid, data }
    }

    /// Create a positive response
    #[must_use]
    pub fn positive(data: Vec<u8>) -> Self {
        Self::new(XcpResponsePid::Response, data)
    }

    /// Create an error response
    #[must_use]
    pub fn error(error_code: XcpErrorCode) -> Self {
        Self::new(XcpResponsePid::Error, vec![error_code as u8])
    }

    /// Check if this is a positive response
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.pid == XcpResponsePid::Response
    }

    /// Check if this is an error response
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.pid == XcpResponsePid::Error
    }

    /// Get error code if this is an error response
    #[must_use]
    pub fn error_code(&self) -> Option<XcpErrorCode> {
        if self.is_error() && !self.data.is_empty() {
            XcpErrorCode::from_u8(self.data[0])
        } else {
            None
        }
    }

    /// Encode response to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.pid as u8];
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Parse response from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(BusmasterError::Parse {
                message: "XCP response too short".into(),
            });
        }

        let pid = XcpResponsePid::from_u8(bytes[0]).ok_or_else(|| BusmasterError::Parse {
            message: format!("Unknown XCP response PID: 0x{:02X}", bytes[0]),
        })?;

        let data = bytes[1..].to_vec();
        Ok(Self { pid, data })
    }
}

/// XCP connection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XcpConnectionMode {
    /// Normal mode
    #[default]
    Normal = 0x00,
    /// User-defined mode
    UserDefined = 0x01,
}

/// XCP resource flags (from CONNECT response)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct XcpResources {
    /// Calibration/paging available
    pub cal_pag: bool,
    /// DAQ available
    pub daq: bool,
    /// STIM available
    pub stim: bool,
    /// Programming available
    pub pgm: bool,
}

impl XcpResources {
    /// Parse from resource byte
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        Self {
            cal_pag: (byte & 0x01) != 0,
            daq: (byte & 0x04) != 0,
            stim: (byte & 0x08) != 0,
            pgm: (byte & 0x10) != 0,
        }
    }

    /// Encode to resource byte
    #[must_use]
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.cal_pag {
            byte |= 0x01;
        }
        if self.daq {
            byte |= 0x04;
        }
        if self.stim {
            byte |= 0x08;
        }
        if self.pgm {
            byte |= 0x10;
        }
        byte
    }
}

/// DAQ list mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DaqListMode {
    /// DAQ direction (measurement)
    #[default]
    Daq = 0x00,
    /// STIM direction (stimulation)
    Stim = 0x01,
}

/// ODT Entry (Object Descriptor Table Entry)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OdtEntry {
    /// Address extension
    pub address_extension: u8,
    /// Memory address
    pub address: u32,
    /// Size in bytes
    pub size: u8,
}

impl OdtEntry {
    /// Create a new ODT entry
    #[must_use]
    pub fn new(address_extension: u8, address: u32, size: u8) -> Self {
        Self {
            address_extension,
            address,
            size,
        }
    }
}

/// ODT (Object Descriptor Table)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Odt {
    /// ODT entries
    pub entries: Vec<OdtEntry>,
}

impl Odt {
    /// Create a new empty ODT
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the ODT
    pub fn add_entry(&mut self, entry: OdtEntry) {
        self.entries.push(entry);
    }

    /// Get total data size of all entries
    #[must_use]
    pub fn total_size(&self) -> usize {
        self.entries.iter().map(|e| e.size as usize).sum()
    }
}

/// DAQ List
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaqList {
    /// DAQ list number
    pub number: u16,
    /// DAQ list mode (DAQ or STIM)
    pub mode: DaqListMode,
    /// Event channel number
    pub event_channel: u16,
    /// Prescaler
    pub prescaler: u8,
    /// Priority
    pub priority: u8,
    /// ODTs in this DAQ list
    pub odts: Vec<Odt>,
}

impl DaqList {
    /// Create a new DAQ list
    #[must_use]
    pub fn new(number: u16) -> Self {
        Self {
            number,
            mode: DaqListMode::Daq,
            event_channel: 0,
            prescaler: 1,
            priority: 0,
            odts: Vec::new(),
        }
    }

    /// Add an ODT to the DAQ list
    pub fn add_odt(&mut self, odt: Odt) {
        self.odts.push(odt);
    }

    /// Get number of ODTs
    #[must_use]
    pub fn odt_count(&self) -> usize {
        self.odts.len()
    }
}

/// XCP event information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaqEvent {
    /// Event channel number
    pub channel: u16,
    /// Event name
    pub name: String,
    /// Cycle time in 100us units (0 = not cyclic)
    pub cycle_time: u16,
    /// Time unit (0 = 1us, 1 = 10us, 2 = 100us, etc.)
    pub time_unit: u8,
    /// Priority
    pub priority: u8,
}

impl DaqEvent {
    /// Create a new DAQ event
    #[must_use]
    pub fn new(channel: u16, name: &str) -> Self {
        Self {
            channel,
            name: name.to_string(),
            cycle_time: 0,
            time_unit: 2, // 100us default
            priority: 0,
        }
    }
}

/// XCP client state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XcpClientState {
    /// Not connected
    #[default]
    Disconnected,
    /// Connected to slave
    Connected,
    /// DAQ running
    DaqRunning,
    /// Programming mode
    Programming,
}

/// XCP Client for managing XCP sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XcpClient {
    /// Client state
    #[serde(skip)]
    state: XcpClientState,
    /// Available resources
    pub resources: XcpResources,
    /// Maximum CTO size (command)
    pub max_cto: u8,
    /// Maximum DTO size (data)
    pub max_dto: u16,
    /// Byte order (true = big endian)
    pub byte_order_msb: bool,
    /// Address granularity (1, 2, or 4 bytes)
    pub address_granularity: u8,
    /// DAQ lists
    pub daq_lists: Vec<DaqList>,
    /// Current MTA (Memory Transfer Address)
    pub mta: Option<(u8, u32)>,
}

impl Default for XcpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl XcpClient {
    /// Create a new XCP client
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: XcpClientState::Disconnected,
            resources: XcpResources::default(),
            max_cto: 8,
            max_dto: 8,
            byte_order_msb: false,
            address_granularity: 1,
            daq_lists: Vec::new(),
            mta: None,
        }
    }

    /// Get current state
    #[must_use]
    pub fn state(&self) -> XcpClientState {
        self.state
    }

    /// Check if connected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.state != XcpClientState::Disconnected
    }

    /// Create CONNECT command
    #[must_use]
    pub fn connect_cmd(&self, mode: XcpConnectionMode) -> XcpCommand {
        XcpCommand::connect(mode as u8)
    }

    /// Process CONNECT response
    pub fn process_connect_response(&mut self, response: &XcpResponse) -> Result<()> {
        if !response.is_positive() {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "CONNECT failed: {:?}",
                    response.error_code().unwrap_or(XcpErrorCode::Generic)
                ),
            });
        }

        if response.data.len() < 7 {
            return Err(BusmasterError::Parse {
                message: "CONNECT response too short".into(),
            });
        }

        self.resources = XcpResources::from_byte(response.data[0]);
        // data[1] = COMM_MODE_BASIC
        let comm_mode = response.data[1];
        self.byte_order_msb = (comm_mode & 0x01) != 0;
        self.address_granularity = match (comm_mode >> 1) & 0x03 {
            0 => 1,
            1 => 2,
            2 => 4,
            _ => 1,
        };
        self.max_cto = response.data[2];
        self.max_dto = u16::from_le_bytes([response.data[3], response.data[4]]);

        self.state = XcpClientState::Connected;
        Ok(())
    }

    /// Create DISCONNECT command
    #[must_use]
    pub fn disconnect_cmd(&self) -> XcpCommand {
        XcpCommand::disconnect()
    }

    /// Process DISCONNECT response
    pub fn process_disconnect_response(&mut self, response: &XcpResponse) -> Result<()> {
        if !response.is_positive() {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "DISCONNECT failed: {:?}",
                    response.error_code().unwrap_or(XcpErrorCode::Generic)
                ),
            });
        }
        self.state = XcpClientState::Disconnected;
        self.mta = None;
        Ok(())
    }

    /// Create SET_MTA command
    #[must_use]
    pub fn set_mta_cmd(&self, address_extension: u8, address: u32) -> XcpCommand {
        XcpCommand::set_mta(address_extension, address)
    }

    /// Process SET_MTA response
    pub fn process_set_mta_response(
        &mut self,
        response: &XcpResponse,
        address_extension: u8,
        address: u32,
    ) -> Result<()> {
        if !response.is_positive() {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "SET_MTA failed: {:?}",
                    response.error_code().unwrap_or(XcpErrorCode::Generic)
                ),
            });
        }
        self.mta = Some((address_extension, address));
        Ok(())
    }

    /// Create UPLOAD command
    #[must_use]
    pub fn upload_cmd(&self, num_elements: u8) -> XcpCommand {
        XcpCommand::upload(num_elements)
    }

    /// Create SHORT_UPLOAD command
    #[must_use]
    pub fn short_upload_cmd(
        &self,
        num_elements: u8,
        address_extension: u8,
        address: u32,
    ) -> XcpCommand {
        XcpCommand::short_upload(num_elements, address_extension, address)
    }

    /// Create DOWNLOAD command
    #[must_use]
    pub fn download_cmd(&self, data: &[u8]) -> XcpCommand {
        XcpCommand::download(data)
    }

    /// Create SHORT_DOWNLOAD command
    #[must_use]
    pub fn short_download_cmd(
        &self,
        address_extension: u8,
        address: u32,
        data: &[u8],
    ) -> XcpCommand {
        XcpCommand::short_download(address_extension, address, data)
    }

    /// Allocate a new DAQ list
    pub fn alloc_daq_list(&mut self) -> u16 {
        let number = self.daq_lists.len() as u16;
        self.daq_lists.push(DaqList::new(number));
        number
    }

    /// Get a DAQ list by number
    #[must_use]
    pub fn get_daq_list(&self, number: u16) -> Option<&DaqList> {
        self.daq_lists.get(number as usize)
    }

    /// Get a mutable DAQ list by number
    pub fn get_daq_list_mut(&mut self, number: u16) -> Option<&mut DaqList> {
        self.daq_lists.get_mut(number as usize)
    }

    /// Create FREE_DAQ command
    #[must_use]
    pub fn free_daq_cmd(&self) -> XcpCommand {
        XcpCommand::new(XcpCommandCode::FreeDaq, vec![])
    }

    /// Process FREE_DAQ response
    pub fn process_free_daq_response(&mut self, response: &XcpResponse) -> Result<()> {
        if !response.is_positive() {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "FREE_DAQ failed: {:?}",
                    response.error_code().unwrap_or(XcpErrorCode::Generic)
                ),
            });
        }
        self.daq_lists.clear();
        Ok(())
    }

    /// Create ALLOC_DAQ command
    #[must_use]
    pub fn alloc_daq_cmd(&self, daq_count: u16) -> XcpCommand {
        let mut data = vec![0]; // Reserved
        data.extend_from_slice(&daq_count.to_le_bytes());
        XcpCommand::new(XcpCommandCode::AllocDaq, data)
    }

    /// Create START_STOP_DAQ_LIST command
    #[must_use]
    pub fn start_stop_daq_list_cmd(&self, mode: u8, daq_list_number: u16) -> XcpCommand {
        let mut data = vec![mode];
        data.extend_from_slice(&daq_list_number.to_le_bytes());
        XcpCommand::new(XcpCommandCode::StartStopDaqList, data)
    }

    /// Create START_STOP_SYNCH command
    #[must_use]
    pub fn start_stop_synch_cmd(&self, mode: u8) -> XcpCommand {
        XcpCommand::new(XcpCommandCode::StartStopSynch, vec![mode])
    }

    /// Process START_STOP_SYNCH response
    pub fn process_start_stop_synch_response(
        &mut self,
        response: &XcpResponse,
        mode: u8,
    ) -> Result<()> {
        if !response.is_positive() {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "START_STOP_SYNCH failed: {:?}",
                    response.error_code().unwrap_or(XcpErrorCode::Generic)
                ),
            });
        }
        // mode: 0 = stop all, 1 = start selected, 2 = stop selected
        if mode == 0 {
            self.state = XcpClientState::Connected;
        } else if mode == 1 {
            self.state = XcpClientState::DaqRunning;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_code_from_u8() {
        assert_eq!(
            XcpCommandCode::from_u8(0xFF),
            Some(XcpCommandCode::Connect)
        );
        assert_eq!(
            XcpCommandCode::from_u8(0xFE),
            Some(XcpCommandCode::Disconnect)
        );
        assert_eq!(
            XcpCommandCode::from_u8(0xE3),
            Some(XcpCommandCode::ClearDaqList)
        );
        assert_eq!(XcpCommandCode::from_u8(0x00), None);
    }

    #[test]
    fn test_response_pid_from_u8() {
        assert_eq!(
            XcpResponsePid::from_u8(0xFF),
            Some(XcpResponsePid::Response)
        );
        assert_eq!(XcpResponsePid::from_u8(0xFE), Some(XcpResponsePid::Error));
        assert_eq!(XcpResponsePid::from_u8(0xFD), Some(XcpResponsePid::Event));
        assert_eq!(XcpResponsePid::from_u8(0x00), None);
    }

    #[test]
    fn test_error_code_from_u8() {
        assert_eq!(
            XcpErrorCode::from_u8(0x00),
            Some(XcpErrorCode::CmdSynch)
        );
        assert_eq!(
            XcpErrorCode::from_u8(0x20),
            Some(XcpErrorCode::CmdUnknown)
        );
        assert_eq!(
            XcpErrorCode::from_u8(0x25),
            Some(XcpErrorCode::AccessLocked)
        );
        assert_eq!(XcpErrorCode::from_u8(0x99), None);
    }

    #[test]
    fn test_command_connect() {
        let cmd = XcpCommand::connect(0x00);
        assert_eq!(cmd.code, XcpCommandCode::Connect);
        assert_eq!(cmd.data, vec![0x00]);

        let bytes = cmd.to_bytes();
        assert_eq!(bytes, vec![0xFF, 0x00]);
    }

    #[test]
    fn test_command_disconnect() {
        let cmd = XcpCommand::disconnect();
        assert_eq!(cmd.code, XcpCommandCode::Disconnect);
        assert!(cmd.data.is_empty());

        let bytes = cmd.to_bytes();
        assert_eq!(bytes, vec![0xFE]);
    }

    #[test]
    fn test_command_set_mta() {
        let cmd = XcpCommand::set_mta(0x01, 0x12345678);
        assert_eq!(cmd.code, XcpCommandCode::SetMta);

        let bytes = cmd.to_bytes();
        assert_eq!(bytes[0], 0xF6); // SET_MTA
        assert_eq!(bytes[3], 0x01); // address extension
        // Address in little-endian
        assert_eq!(bytes[4], 0x78);
        assert_eq!(bytes[5], 0x56);
        assert_eq!(bytes[6], 0x34);
        assert_eq!(bytes[7], 0x12);
    }

    #[test]
    fn test_command_upload() {
        let cmd = XcpCommand::upload(4);
        assert_eq!(cmd.code, XcpCommandCode::Upload);
        assert_eq!(cmd.data, vec![4]);
    }

    #[test]
    fn test_command_short_upload() {
        let cmd = XcpCommand::short_upload(8, 0x00, 0x1000);
        assert_eq!(cmd.code, XcpCommandCode::ShortUpload);
        assert_eq!(cmd.data[0], 8); // num_elements
    }

    #[test]
    fn test_command_download() {
        let cmd = XcpCommand::download(&[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(cmd.code, XcpCommandCode::Download);
        assert_eq!(cmd.data[0], 4); // length
        assert_eq!(&cmd.data[1..], &[0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn test_command_from_bytes() {
        let bytes = vec![0xFF, 0x00]; // CONNECT normal mode
        let cmd = XcpCommand::from_bytes(&bytes).unwrap();
        assert_eq!(cmd.code, XcpCommandCode::Connect);
        assert_eq!(cmd.data, vec![0x00]);
    }

    #[test]
    fn test_command_from_bytes_empty() {
        let result = XcpCommand::from_bytes(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_response_positive() {
        let resp = XcpResponse::positive(vec![0x01, 0x02, 0x03]);
        assert!(resp.is_positive());
        assert!(!resp.is_error());
        assert_eq!(resp.error_code(), None);
    }

    #[test]
    fn test_response_error() {
        let resp = XcpResponse::error(XcpErrorCode::AccessDenied);
        assert!(!resp.is_positive());
        assert!(resp.is_error());
        assert_eq!(resp.error_code(), Some(XcpErrorCode::AccessDenied));
    }

    #[test]
    fn test_response_roundtrip() {
        let resp = XcpResponse::positive(vec![0xAA, 0xBB]);
        let bytes = resp.to_bytes();
        let parsed = XcpResponse::from_bytes(&bytes).unwrap();
        assert_eq!(resp, parsed);
    }

    #[test]
    fn test_resources_from_byte() {
        let res = XcpResources::from_byte(0x1D); // cal_pag, daq, stim, pgm
        assert!(res.cal_pag);
        assert!(res.daq);
        assert!(res.stim);
        assert!(res.pgm);

        let res2 = XcpResources::from_byte(0x00);
        assert!(!res2.cal_pag);
        assert!(!res2.daq);
        assert!(!res2.stim);
        assert!(!res2.pgm);
    }

    #[test]
    fn test_resources_to_byte() {
        let res = XcpResources {
            cal_pag: true,
            daq: true,
            stim: false,
            pgm: true,
        };
        let byte = res.to_byte();
        assert_eq!(byte, 0x15); // 0x01 | 0x04 | 0x10
    }

    #[test]
    fn test_odt_entry() {
        let entry = OdtEntry::new(0x00, 0x1000, 4);
        assert_eq!(entry.address_extension, 0x00);
        assert_eq!(entry.address, 0x1000);
        assert_eq!(entry.size, 4);
    }

    #[test]
    fn test_odt() {
        let mut odt = Odt::new();
        odt.add_entry(OdtEntry::new(0x00, 0x1000, 4));
        odt.add_entry(OdtEntry::new(0x00, 0x1004, 2));
        assert_eq!(odt.entries.len(), 2);
        assert_eq!(odt.total_size(), 6);
    }

    #[test]
    fn test_daq_list() {
        let mut daq = DaqList::new(0);
        assert_eq!(daq.number, 0);
        assert_eq!(daq.odt_count(), 0);

        let mut odt = Odt::new();
        odt.add_entry(OdtEntry::new(0x00, 0x1000, 4));
        daq.add_odt(odt);
        assert_eq!(daq.odt_count(), 1);
    }

    #[test]
    fn test_daq_event() {
        let event = DaqEvent::new(0, "10ms_Task");
        assert_eq!(event.channel, 0);
        assert_eq!(event.name, "10ms_Task");
    }

    #[test]
    fn test_xcp_client_new() {
        let client = XcpClient::new();
        assert_eq!(client.state(), XcpClientState::Disconnected);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_xcp_client_connect_response() {
        let mut client = XcpClient::new();

        // Simulate CONNECT response
        // [RESOURCE, COMM_MODE_BASIC, MAX_CTO, MAX_DTO_LSB, MAX_DTO_MSB, XCP_PROTOCOL_LAYER_VERSION, XCP_TRANSPORT_LAYER_VERSION]
        let response = XcpResponse::positive(vec![0x1D, 0x00, 0x08, 0x08, 0x00, 0x01, 0x01]);
        client.process_connect_response(&response).unwrap();

        assert!(client.is_connected());
        assert_eq!(client.state(), XcpClientState::Connected);
        assert!(client.resources.cal_pag);
        assert!(client.resources.daq);
        assert!(client.resources.stim);
        assert!(client.resources.pgm);
        assert_eq!(client.max_cto, 8);
        assert_eq!(client.max_dto, 8);
    }

    #[test]
    fn test_xcp_client_disconnect_response() {
        let mut client = XcpClient::new();
        client.state = XcpClientState::Connected;

        let response = XcpResponse::positive(vec![]);
        client.process_disconnect_response(&response).unwrap();

        assert!(!client.is_connected());
        assert_eq!(client.state(), XcpClientState::Disconnected);
    }

    #[test]
    fn test_xcp_client_alloc_daq() {
        let mut client = XcpClient::new();
        let daq0 = client.alloc_daq_list();
        let daq1 = client.alloc_daq_list();

        assert_eq!(daq0, 0);
        assert_eq!(daq1, 1);
        assert_eq!(client.daq_lists.len(), 2);
    }

    #[test]
    fn test_xcp_client_free_daq() {
        let mut client = XcpClient::new();
        client.alloc_daq_list();
        client.alloc_daq_list();
        assert_eq!(client.daq_lists.len(), 2);

        let response = XcpResponse::positive(vec![]);
        client.process_free_daq_response(&response).unwrap();
        assert!(client.daq_lists.is_empty());
    }
}
