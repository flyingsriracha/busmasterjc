//! DoIP (Diagnostics over IP) Protocol Implementation
//!
//! DoIP is defined in ISO 13400 and provides diagnostic communication over IP networks.
//! It's commonly used in modern vehicles for Ethernet-based diagnostics.
//!
//! # Protocol Overview
//!
//! DoIP uses TCP for reliable diagnostic communication and UDP for vehicle discovery.
//! The protocol supports:
//! - Vehicle identification and discovery
//! - Routing activation
//! - Diagnostic message tunneling (UDS over DoIP)
//! - Alive check mechanism
//!
//! # Example
//!
//! ```
//! use busmaster_proto::doip::{DoipHeader, DoipPayloadType, DoipMessage};
//!
//! // Create a vehicle identification request
//! let header = DoipHeader::new(DoipPayloadType::VehicleIdentificationRequest, 0);
//! let bytes = header.to_bytes();
//! assert_eq!(bytes.len(), 8);
//! ```

use busmaster_core::{BusmasterError, Result};

/// DoIP protocol version
pub const DOIP_PROTOCOL_VERSION: u8 = 0x02;

/// DoIP header size (8 bytes)
pub const DOIP_HEADER_SIZE: usize = 8;

/// DoIP default TCP port
pub const DOIP_TCP_PORT: u16 = 13400;

/// DoIP default UDP port for discovery
pub const DOIP_UDP_PORT: u16 = 13400;

/// DoIP payload types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DoipPayloadType {
    /// Generic DoIP header negative acknowledge
    GenericNegativeAck = 0x0000,
    /// Vehicle identification request
    VehicleIdentificationRequest = 0x0001,
    /// Vehicle identification request with EID
    VehicleIdentificationRequestEid = 0x0002,
    /// Vehicle identification request with VIN
    VehicleIdentificationRequestVin = 0x0003,
    /// Vehicle announcement/identification response
    VehicleIdentificationResponse = 0x0004,
    /// Routing activation request
    RoutingActivationRequest = 0x0005,
    /// Routing activation response
    RoutingActivationResponse = 0x0006,
    /// Alive check request
    AliveCheckRequest = 0x0007,
    /// Alive check response
    AliveCheckResponse = 0x0008,
    /// DoIP entity status request
    EntityStatusRequest = 0x4001,
    /// DoIP entity status response
    EntityStatusResponse = 0x4002,
    /// Diagnostic power mode information request
    DiagnosticPowerModeRequest = 0x4003,
    /// Diagnostic power mode information response
    DiagnosticPowerModeResponse = 0x4004,
    /// Diagnostic message
    DiagnosticMessage = 0x8001,
    /// Diagnostic message positive acknowledge
    DiagnosticMessagePositiveAck = 0x8002,
    /// Diagnostic message negative acknowledge
    DiagnosticMessageNegativeAck = 0x8003,
}

impl DoipPayloadType {
    /// Create from raw u16 value
    #[must_use]
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0000 => Some(Self::GenericNegativeAck),
            0x0001 => Some(Self::VehicleIdentificationRequest),
            0x0002 => Some(Self::VehicleIdentificationRequestEid),
            0x0003 => Some(Self::VehicleIdentificationRequestVin),
            0x0004 => Some(Self::VehicleIdentificationResponse),
            0x0005 => Some(Self::RoutingActivationRequest),
            0x0006 => Some(Self::RoutingActivationResponse),
            0x0007 => Some(Self::AliveCheckRequest),
            0x0008 => Some(Self::AliveCheckResponse),
            0x4001 => Some(Self::EntityStatusRequest),
            0x4002 => Some(Self::EntityStatusResponse),
            0x4003 => Some(Self::DiagnosticPowerModeRequest),
            0x4004 => Some(Self::DiagnosticPowerModeResponse),
            0x8001 => Some(Self::DiagnosticMessage),
            0x8002 => Some(Self::DiagnosticMessagePositiveAck),
            0x8003 => Some(Self::DiagnosticMessageNegativeAck),
            _ => None,
        }
    }

    /// Check if this is a request type
    #[must_use]
    pub fn is_request(&self) -> bool {
        matches!(
            self,
            Self::VehicleIdentificationRequest
                | Self::VehicleIdentificationRequestEid
                | Self::VehicleIdentificationRequestVin
                | Self::RoutingActivationRequest
                | Self::AliveCheckRequest
                | Self::EntityStatusRequest
                | Self::DiagnosticPowerModeRequest
                | Self::DiagnosticMessage
        )
    }

    /// Check if this is a response type
    #[must_use]
    pub fn is_response(&self) -> bool {
        matches!(
            self,
            Self::GenericNegativeAck
                | Self::VehicleIdentificationResponse
                | Self::RoutingActivationResponse
                | Self::AliveCheckResponse
                | Self::EntityStatusResponse
                | Self::DiagnosticPowerModeResponse
                | Self::DiagnosticMessagePositiveAck
                | Self::DiagnosticMessageNegativeAck
        )
    }
}

/// Generic DoIP negative acknowledge codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DoipNackCode {
    /// Incorrect pattern format
    IncorrectPatternFormat = 0x00,
    /// Unknown payload type
    UnknownPayloadType = 0x01,
    /// Message too large
    MessageTooLarge = 0x02,
    /// Out of memory
    OutOfMemory = 0x03,
    /// Invalid payload length
    InvalidPayloadLength = 0x04,
}

impl DoipNackCode {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::IncorrectPatternFormat),
            0x01 => Some(Self::UnknownPayloadType),
            0x02 => Some(Self::MessageTooLarge),
            0x03 => Some(Self::OutOfMemory),
            0x04 => Some(Self::InvalidPayloadLength),
            _ => None,
        }
    }
}

/// Routing activation response codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RoutingActivationResponseCode {
    /// Routing activation denied - unknown source address
    DeniedUnknownSourceAddress = 0x00,
    /// Routing activation denied - all sockets registered and active
    DeniedAllSocketsActive = 0x01,
    /// Routing activation denied - SA different from registered
    DeniedSaDifferent = 0x02,
    /// Routing activation denied - SA already registered
    DeniedSaAlreadyRegistered = 0x03,
    /// Routing activation denied - missing authentication
    DeniedMissingAuthentication = 0x04,
    /// Routing activation denied - rejected confirmation
    DeniedRejectedConfirmation = 0x05,
    /// Routing activation denied - unsupported activation type
    DeniedUnsupportedActivationType = 0x06,
    /// Routing successfully activated
    ActivatedSuccessfully = 0x10,
    /// Routing will be activated - confirmation required
    ActivatedConfirmationRequired = 0x11,
}

impl RoutingActivationResponseCode {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::DeniedUnknownSourceAddress),
            0x01 => Some(Self::DeniedAllSocketsActive),
            0x02 => Some(Self::DeniedSaDifferent),
            0x03 => Some(Self::DeniedSaAlreadyRegistered),
            0x04 => Some(Self::DeniedMissingAuthentication),
            0x05 => Some(Self::DeniedRejectedConfirmation),
            0x06 => Some(Self::DeniedUnsupportedActivationType),
            0x10 => Some(Self::ActivatedSuccessfully),
            0x11 => Some(Self::ActivatedConfirmationRequired),
            _ => None,
        }
    }

    /// Check if activation was successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::ActivatedSuccessfully | Self::ActivatedConfirmationRequired
        )
    }
}

/// Diagnostic message acknowledge codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagnosticMessageAckCode {
    /// Routing confirmation ACK
    RoutingConfirmationAck = 0x00,
}

/// Diagnostic message negative acknowledge codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagnosticMessageNackCode {
    /// Invalid source address
    InvalidSourceAddress = 0x02,
    /// Unknown target address
    UnknownTargetAddress = 0x03,
    /// Diagnostic message too large
    DiagnosticMessageTooLarge = 0x04,
    /// Out of memory
    OutOfMemory = 0x05,
    /// Target unreachable
    TargetUnreachable = 0x06,
    /// Unknown network
    UnknownNetwork = 0x07,
    /// Transport protocol error
    TransportProtocolError = 0x08,
}

impl DiagnosticMessageNackCode {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x02 => Some(Self::InvalidSourceAddress),
            0x03 => Some(Self::UnknownTargetAddress),
            0x04 => Some(Self::DiagnosticMessageTooLarge),
            0x05 => Some(Self::OutOfMemory),
            0x06 => Some(Self::TargetUnreachable),
            0x07 => Some(Self::UnknownNetwork),
            0x08 => Some(Self::TransportProtocolError),
            _ => None,
        }
    }
}

/// DoIP Header (8 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoipHeader {
    /// Protocol version (0x02 for ISO 13400-2:2012)
    pub protocol_version: u8,
    /// Inverse protocol version (for validation)
    pub inverse_version: u8,
    /// Payload type
    pub payload_type: DoipPayloadType,
    /// Payload length (excluding header)
    pub payload_length: u32,
}

impl DoipHeader {
    /// Create a new DoIP header
    #[must_use]
    pub fn new(payload_type: DoipPayloadType, payload_length: u32) -> Self {
        Self {
            protocol_version: DOIP_PROTOCOL_VERSION,
            inverse_version: !DOIP_PROTOCOL_VERSION,
            payload_type,
            payload_length,
        }
    }

    /// Validate the header
    pub fn validate(&self) -> Result<()> {
        // Check protocol version
        if self.protocol_version != DOIP_PROTOCOL_VERSION {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "Invalid DoIP protocol version: 0x{:02X}",
                    self.protocol_version
                ),
            });
        }

        // Check inverse version
        if self.inverse_version != !self.protocol_version {
            return Err(BusmasterError::Protocol {
                message: "Invalid DoIP inverse version".into(),
            });
        }

        Ok(())
    }

    /// Write to bytes (big-endian as per ISO 13400)
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(DOIP_HEADER_SIZE);
        bytes.push(self.protocol_version);
        bytes.push(self.inverse_version);
        bytes.extend_from_slice(&(self.payload_type as u16).to_be_bytes());
        bytes.extend_from_slice(&self.payload_length.to_be_bytes());
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < DOIP_HEADER_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "DoIP header too short: {} bytes, expected {}",
                    bytes.len(),
                    DOIP_HEADER_SIZE
                ),
            });
        }

        let protocol_version = bytes[0];
        let inverse_version = bytes[1];
        let payload_type_raw = u16::from_be_bytes([bytes[2], bytes[3]]);
        let payload_length = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        let payload_type = DoipPayloadType::from_u16(payload_type_raw).ok_or_else(|| {
            BusmasterError::Parse {
                message: format!("Unknown DoIP payload type: 0x{:04X}", payload_type_raw),
            }
        })?;

        let header = Self {
            protocol_version,
            inverse_version,
            payload_type,
            payload_length,
        };

        header.validate()?;
        Ok(header)
    }
}


/// Vehicle Identification Response (VIN, EID, GID, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleIdentificationResponse {
    /// VIN (17 bytes ASCII)
    pub vin: [u8; 17],
    /// Logical address (2 bytes)
    pub logical_address: u16,
    /// Entity ID (6 bytes, typically MAC address)
    pub eid: [u8; 6],
    /// Group ID (6 bytes)
    pub gid: [u8; 6],
    /// Further action required
    pub further_action: u8,
    /// VIN/GID sync status (optional)
    pub vin_gid_sync: Option<u8>,
}

impl VehicleIdentificationResponse {
    /// Minimum payload size (without optional VIN/GID sync)
    pub const MIN_SIZE: usize = 32;
    /// Maximum payload size (with optional VIN/GID sync)
    pub const MAX_SIZE: usize = 33;

    /// Get VIN as string
    #[must_use]
    pub fn vin_string(&self) -> String {
        String::from_utf8_lossy(&self.vin).trim_end_matches('\0').to_string()
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MAX_SIZE);
        bytes.extend_from_slice(&self.vin);
        bytes.extend_from_slice(&self.logical_address.to_be_bytes());
        bytes.extend_from_slice(&self.eid);
        bytes.extend_from_slice(&self.gid);
        bytes.push(self.further_action);
        if let Some(sync) = self.vin_gid_sync {
            bytes.push(sync);
        }
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::MIN_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Vehicle identification response too short: {} bytes",
                    bytes.len()
                ),
            });
        }

        let mut vin = [0u8; 17];
        vin.copy_from_slice(&bytes[0..17]);

        let logical_address = u16::from_be_bytes([bytes[17], bytes[18]]);

        let mut eid = [0u8; 6];
        eid.copy_from_slice(&bytes[19..25]);

        let mut gid = [0u8; 6];
        gid.copy_from_slice(&bytes[25..31]);

        let further_action = bytes[31];

        let vin_gid_sync = if bytes.len() > Self::MIN_SIZE {
            Some(bytes[32])
        } else {
            None
        };

        Ok(Self {
            vin,
            logical_address,
            eid,
            gid,
            further_action,
            vin_gid_sync,
        })
    }
}

/// Routing Activation Request
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingActivationRequest {
    /// Source address (tester logical address)
    pub source_address: u16,
    /// Activation type
    pub activation_type: u8,
    /// Reserved (ISO) or OEM specific
    pub reserved: u32,
    /// OEM specific data (optional)
    pub oem_specific: Option<u32>,
}

impl RoutingActivationRequest {
    /// Default activation type
    pub const ACTIVATION_TYPE_DEFAULT: u8 = 0x00;
    /// WWH-OBD activation type
    pub const ACTIVATION_TYPE_WWH_OBD: u8 = 0x01;
    /// Central security activation type
    pub const ACTIVATION_TYPE_CENTRAL_SECURITY: u8 = 0xE0;

    /// Minimum payload size
    pub const MIN_SIZE: usize = 7;
    /// Maximum payload size (with OEM specific)
    pub const MAX_SIZE: usize = 11;

    /// Create a new routing activation request
    #[must_use]
    pub fn new(source_address: u16, activation_type: u8) -> Self {
        Self {
            source_address,
            activation_type,
            reserved: 0,
            oem_specific: None,
        }
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MAX_SIZE);
        bytes.extend_from_slice(&self.source_address.to_be_bytes());
        bytes.push(self.activation_type);
        bytes.extend_from_slice(&self.reserved.to_be_bytes());
        if let Some(oem) = self.oem_specific {
            bytes.extend_from_slice(&oem.to_be_bytes());
        }
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::MIN_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Routing activation request too short: {} bytes",
                    bytes.len()
                ),
            });
        }

        let source_address = u16::from_be_bytes([bytes[0], bytes[1]]);
        let activation_type = bytes[2];
        let reserved = u32::from_be_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);

        let oem_specific = if bytes.len() >= Self::MAX_SIZE {
            Some(u32::from_be_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]))
        } else {
            None
        };

        Ok(Self {
            source_address,
            activation_type,
            reserved,
            oem_specific,
        })
    }
}

/// Routing Activation Response
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingActivationResponse {
    /// Tester logical address (from request)
    pub tester_address: u16,
    /// DoIP entity logical address
    pub entity_address: u16,
    /// Response code
    pub response_code: RoutingActivationResponseCode,
    /// Reserved (ISO)
    pub reserved: u32,
    /// OEM specific data (optional)
    pub oem_specific: Option<u32>,
}

impl RoutingActivationResponse {
    /// Minimum payload size
    pub const MIN_SIZE: usize = 9;
    /// Maximum payload size (with OEM specific)
    pub const MAX_SIZE: usize = 13;

    /// Check if routing was activated successfully
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.response_code.is_success()
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MAX_SIZE);
        bytes.extend_from_slice(&self.tester_address.to_be_bytes());
        bytes.extend_from_slice(&self.entity_address.to_be_bytes());
        bytes.push(self.response_code as u8);
        bytes.extend_from_slice(&self.reserved.to_be_bytes());
        if let Some(oem) = self.oem_specific {
            bytes.extend_from_slice(&oem.to_be_bytes());
        }
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::MIN_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Routing activation response too short: {} bytes",
                    bytes.len()
                ),
            });
        }

        let tester_address = u16::from_be_bytes([bytes[0], bytes[1]]);
        let entity_address = u16::from_be_bytes([bytes[2], bytes[3]]);
        let response_code =
            RoutingActivationResponseCode::from_u8(bytes[4]).ok_or_else(|| {
                BusmasterError::Parse {
                    message: format!("Unknown routing activation response code: 0x{:02X}", bytes[4]),
                }
            })?;
        let reserved = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);

        let oem_specific = if bytes.len() >= Self::MAX_SIZE {
            Some(u32::from_be_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]))
        } else {
            None
        };

        Ok(Self {
            tester_address,
            entity_address,
            response_code,
            reserved,
            oem_specific,
        })
    }
}

/// Diagnostic Message (UDS over DoIP)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticMessage {
    /// Source address (tester)
    pub source_address: u16,
    /// Target address (ECU)
    pub target_address: u16,
    /// User data (UDS message)
    pub user_data: Vec<u8>,
}

impl DiagnosticMessage {
    /// Header size (source + target addresses)
    pub const HEADER_SIZE: usize = 4;

    /// Create a new diagnostic message
    #[must_use]
    pub fn new(source_address: u16, target_address: u16, user_data: Vec<u8>) -> Self {
        Self {
            source_address,
            target_address,
            user_data,
        }
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::HEADER_SIZE + self.user_data.len());
        bytes.extend_from_slice(&self.source_address.to_be_bytes());
        bytes.extend_from_slice(&self.target_address.to_be_bytes());
        bytes.extend_from_slice(&self.user_data);
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::HEADER_SIZE {
            return Err(BusmasterError::Parse {
                message: format!("Diagnostic message too short: {} bytes", bytes.len()),
            });
        }

        let source_address = u16::from_be_bytes([bytes[0], bytes[1]]);
        let target_address = u16::from_be_bytes([bytes[2], bytes[3]]);
        let user_data = bytes[Self::HEADER_SIZE..].to_vec();

        Ok(Self {
            source_address,
            target_address,
            user_data,
        })
    }
}

/// Diagnostic Message Positive Acknowledge
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticMessagePositiveAck {
    /// Source address
    pub source_address: u16,
    /// Target address
    pub target_address: u16,
    /// ACK code
    pub ack_code: u8,
    /// Previous diagnostic message data (optional)
    pub previous_data: Vec<u8>,
}

impl DiagnosticMessagePositiveAck {
    /// Minimum payload size
    pub const MIN_SIZE: usize = 5;

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::MIN_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Diagnostic message positive ACK too short: {} bytes",
                    bytes.len()
                ),
            });
        }

        let source_address = u16::from_be_bytes([bytes[0], bytes[1]]);
        let target_address = u16::from_be_bytes([bytes[2], bytes[3]]);
        let ack_code = bytes[4];
        let previous_data = bytes[5..].to_vec();

        Ok(Self {
            source_address,
            target_address,
            ack_code,
            previous_data,
        })
    }
}

/// Diagnostic Message Negative Acknowledge
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticMessageNegativeAck {
    /// Source address
    pub source_address: u16,
    /// Target address
    pub target_address: u16,
    /// NACK code
    pub nack_code: DiagnosticMessageNackCode,
    /// Previous diagnostic message data (optional)
    pub previous_data: Vec<u8>,
}

impl DiagnosticMessageNegativeAck {
    /// Minimum payload size
    pub const MIN_SIZE: usize = 5;

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::MIN_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "Diagnostic message negative ACK too short: {} bytes",
                    bytes.len()
                ),
            });
        }

        let source_address = u16::from_be_bytes([bytes[0], bytes[1]]);
        let target_address = u16::from_be_bytes([bytes[2], bytes[3]]);
        let nack_code =
            DiagnosticMessageNackCode::from_u8(bytes[4]).ok_or_else(|| BusmasterError::Parse {
                message: format!("Unknown diagnostic message NACK code: 0x{:02X}", bytes[4]),
            })?;
        let previous_data = bytes[5..].to_vec();

        Ok(Self {
            source_address,
            target_address,
            nack_code,
            previous_data,
        })
    }
}

/// Alive Check Request (empty payload)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AliveCheckRequest;

/// Alive Check Response
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliveCheckResponse {
    /// Source address
    pub source_address: u16,
}

impl AliveCheckResponse {
    /// Payload size
    pub const SIZE: usize = 2;

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(BusmasterError::Parse {
                message: format!("Alive check response too short: {} bytes", bytes.len()),
            });
        }

        let source_address = u16::from_be_bytes([bytes[0], bytes[1]]);
        Ok(Self { source_address })
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.source_address.to_be_bytes().to_vec()
    }
}


/// DoIP Message (header + payload)
#[derive(Debug, Clone)]
pub enum DoipMessage {
    /// Vehicle identification request
    VehicleIdentificationRequest,
    /// Vehicle identification request with EID
    VehicleIdentificationRequestEid([u8; 6]),
    /// Vehicle identification request with VIN
    VehicleIdentificationRequestVin([u8; 17]),
    /// Vehicle identification response
    VehicleIdentificationResponse(VehicleIdentificationResponse),
    /// Routing activation request
    RoutingActivationRequest(RoutingActivationRequest),
    /// Routing activation response
    RoutingActivationResponse(RoutingActivationResponse),
    /// Alive check request
    AliveCheckRequest,
    /// Alive check response
    AliveCheckResponse(AliveCheckResponse),
    /// Diagnostic message
    DiagnosticMessage(DiagnosticMessage),
    /// Diagnostic message positive ACK
    DiagnosticMessagePositiveAck(DiagnosticMessagePositiveAck),
    /// Diagnostic message negative ACK
    DiagnosticMessageNegativeAck(DiagnosticMessageNegativeAck),
    /// Generic negative acknowledge
    GenericNegativeAck(DoipNackCode),
}

impl DoipMessage {
    /// Get the payload type for this message
    #[must_use]
    pub fn payload_type(&self) -> DoipPayloadType {
        match self {
            Self::VehicleIdentificationRequest => DoipPayloadType::VehicleIdentificationRequest,
            Self::VehicleIdentificationRequestEid(_) => {
                DoipPayloadType::VehicleIdentificationRequestEid
            },
            Self::VehicleIdentificationRequestVin(_) => {
                DoipPayloadType::VehicleIdentificationRequestVin
            },
            Self::VehicleIdentificationResponse(_) => DoipPayloadType::VehicleIdentificationResponse,
            Self::RoutingActivationRequest(_) => DoipPayloadType::RoutingActivationRequest,
            Self::RoutingActivationResponse(_) => DoipPayloadType::RoutingActivationResponse,
            Self::AliveCheckRequest => DoipPayloadType::AliveCheckRequest,
            Self::AliveCheckResponse(_) => DoipPayloadType::AliveCheckResponse,
            Self::DiagnosticMessage(_) => DoipPayloadType::DiagnosticMessage,
            Self::DiagnosticMessagePositiveAck(_) => DoipPayloadType::DiagnosticMessagePositiveAck,
            Self::DiagnosticMessageNegativeAck(_) => DoipPayloadType::DiagnosticMessageNegativeAck,
            Self::GenericNegativeAck(_) => DoipPayloadType::GenericNegativeAck,
        }
    }

    /// Encode the message to bytes (header + payload)
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let payload = self.payload_bytes();
        let header = DoipHeader::new(self.payload_type(), payload.len() as u32);
        let mut bytes = header.to_bytes();
        bytes.extend_from_slice(&payload);
        bytes
    }

    /// Get the payload bytes
    #[must_use]
    fn payload_bytes(&self) -> Vec<u8> {
        match self {
            Self::VehicleIdentificationRequest | Self::AliveCheckRequest => Vec::new(),
            Self::VehicleIdentificationRequestEid(eid) => eid.to_vec(),
            Self::VehicleIdentificationRequestVin(vin) => vin.to_vec(),
            Self::VehicleIdentificationResponse(resp) => resp.to_bytes(),
            Self::RoutingActivationRequest(req) => req.to_bytes(),
            Self::RoutingActivationResponse(resp) => resp.to_bytes(),
            Self::AliveCheckResponse(resp) => resp.to_bytes(),
            Self::DiagnosticMessage(msg) => msg.to_bytes(),
            Self::DiagnosticMessagePositiveAck(ack) => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&ack.source_address.to_be_bytes());
                bytes.extend_from_slice(&ack.target_address.to_be_bytes());
                bytes.push(ack.ack_code);
                bytes.extend_from_slice(&ack.previous_data);
                bytes
            },
            Self::DiagnosticMessageNegativeAck(nack) => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&nack.source_address.to_be_bytes());
                bytes.extend_from_slice(&nack.target_address.to_be_bytes());
                bytes.push(nack.nack_code as u8);
                bytes.extend_from_slice(&nack.previous_data);
                bytes
            },
            Self::GenericNegativeAck(code) => vec![*code as u8],
        }
    }

    /// Parse a DoIP message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = DoipHeader::from_bytes(bytes)?;
        let payload = &bytes[DOIP_HEADER_SIZE..];

        if payload.len() < header.payload_length as usize {
            return Err(BusmasterError::Parse {
                message: format!(
                    "DoIP payload too short: {} bytes, expected {}",
                    payload.len(),
                    header.payload_length
                ),
            });
        }

        let payload = &payload[..header.payload_length as usize];

        match header.payload_type {
            DoipPayloadType::VehicleIdentificationRequest => Ok(Self::VehicleIdentificationRequest),
            DoipPayloadType::VehicleIdentificationRequestEid => {
                if payload.len() < 6 {
                    return Err(BusmasterError::Parse {
                        message: "EID too short".into(),
                    });
                }
                let mut eid = [0u8; 6];
                eid.copy_from_slice(&payload[..6]);
                Ok(Self::VehicleIdentificationRequestEid(eid))
            },
            DoipPayloadType::VehicleIdentificationRequestVin => {
                if payload.len() < 17 {
                    return Err(BusmasterError::Parse {
                        message: "VIN too short".into(),
                    });
                }
                let mut vin = [0u8; 17];
                vin.copy_from_slice(&payload[..17]);
                Ok(Self::VehicleIdentificationRequestVin(vin))
            },
            DoipPayloadType::VehicleIdentificationResponse => {
                Ok(Self::VehicleIdentificationResponse(
                    VehicleIdentificationResponse::from_bytes(payload)?,
                ))
            },
            DoipPayloadType::RoutingActivationRequest => Ok(Self::RoutingActivationRequest(
                RoutingActivationRequest::from_bytes(payload)?,
            )),
            DoipPayloadType::RoutingActivationResponse => Ok(Self::RoutingActivationResponse(
                RoutingActivationResponse::from_bytes(payload)?,
            )),
            DoipPayloadType::AliveCheckRequest => Ok(Self::AliveCheckRequest),
            DoipPayloadType::AliveCheckResponse => Ok(Self::AliveCheckResponse(
                AliveCheckResponse::from_bytes(payload)?,
            )),
            DoipPayloadType::DiagnosticMessage => {
                Ok(Self::DiagnosticMessage(DiagnosticMessage::from_bytes(payload)?))
            },
            DoipPayloadType::DiagnosticMessagePositiveAck => Ok(Self::DiagnosticMessagePositiveAck(
                DiagnosticMessagePositiveAck::from_bytes(payload)?,
            )),
            DoipPayloadType::DiagnosticMessageNegativeAck => Ok(Self::DiagnosticMessageNegativeAck(
                DiagnosticMessageNegativeAck::from_bytes(payload)?,
            )),
            DoipPayloadType::GenericNegativeAck => {
                if payload.is_empty() {
                    return Err(BusmasterError::Parse {
                        message: "Generic NACK payload empty".into(),
                    });
                }
                let code = DoipNackCode::from_u8(payload[0]).ok_or_else(|| {
                    BusmasterError::Parse {
                        message: format!("Unknown NACK code: 0x{:02X}", payload[0]),
                    }
                })?;
                Ok(Self::GenericNegativeAck(code))
            },
            _ => Err(BusmasterError::Parse {
                message: format!("Unsupported payload type: {:?}", header.payload_type),
            }),
        }
    }
}

/// DoIP Client for managing diagnostic sessions
#[derive(Debug)]
pub struct DoipClient {
    /// Tester logical address
    pub tester_address: u16,
    /// Target ECU logical address
    pub target_address: u16,
    /// Connection state
    pub state: DoipClientState,
}

/// DoIP Client connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoipClientState {
    /// Not connected
    Disconnected,
    /// TCP connected, routing not activated
    Connected,
    /// Routing activated, ready for diagnostics
    RoutingActivated,
}

impl DoipClient {
    /// Create a new DoIP client
    #[must_use]
    pub fn new(tester_address: u16, target_address: u16) -> Self {
        Self {
            tester_address,
            target_address,
            state: DoipClientState::Disconnected,
        }
    }

    /// Create a vehicle identification request
    #[must_use]
    pub fn create_vehicle_identification_request(&self) -> DoipMessage {
        DoipMessage::VehicleIdentificationRequest
    }

    /// Create a routing activation request
    #[must_use]
    pub fn create_routing_activation_request(&self) -> DoipMessage {
        DoipMessage::RoutingActivationRequest(RoutingActivationRequest::new(
            self.tester_address,
            RoutingActivationRequest::ACTIVATION_TYPE_DEFAULT,
        ))
    }

    /// Create an alive check response
    #[must_use]
    pub fn create_alive_check_response(&self) -> DoipMessage {
        DoipMessage::AliveCheckResponse(AliveCheckResponse {
            source_address: self.tester_address,
        })
    }

    /// Create a diagnostic message (wraps UDS data)
    #[must_use]
    pub fn create_diagnostic_message(&self, uds_data: Vec<u8>) -> DoipMessage {
        DoipMessage::DiagnosticMessage(DiagnosticMessage::new(
            self.tester_address,
            self.target_address,
            uds_data,
        ))
    }

    /// Process a received message and update state
    pub fn process_response(&mut self, message: &DoipMessage) -> Result<()> {
        match message {
            DoipMessage::RoutingActivationResponse(resp) => {
                if resp.is_success() {
                    self.state = DoipClientState::RoutingActivated;
                } else {
                    return Err(BusmasterError::Protocol {
                        message: format!(
                            "Routing activation failed: {:?}",
                            resp.response_code
                        ),
                    });
                }
            },
            DoipMessage::AliveCheckRequest => {
                // Should respond with alive check response
            },
            DoipMessage::GenericNegativeAck(code) => {
                return Err(BusmasterError::Protocol {
                    message: format!("DoIP negative acknowledge: {:?}", code),
                });
            },
            _ => {},
        }
        Ok(())
    }

    /// Check if routing is activated
    #[must_use]
    pub fn is_routing_activated(&self) -> bool {
        self.state == DoipClientState::RoutingActivated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doip_header_creation() {
        let header = DoipHeader::new(DoipPayloadType::VehicleIdentificationRequest, 0);
        assert_eq!(header.protocol_version, DOIP_PROTOCOL_VERSION);
        assert_eq!(header.inverse_version, !DOIP_PROTOCOL_VERSION);
        assert_eq!(header.payload_type, DoipPayloadType::VehicleIdentificationRequest);
        assert_eq!(header.payload_length, 0);
    }

    #[test]
    fn test_doip_header_roundtrip() {
        let header = DoipHeader::new(DoipPayloadType::DiagnosticMessage, 100);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), DOIP_HEADER_SIZE);

        let parsed = DoipHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.protocol_version, header.protocol_version);
        assert_eq!(parsed.payload_type, header.payload_type);
        assert_eq!(parsed.payload_length, header.payload_length);
    }

    #[test]
    fn test_doip_header_validation() {
        let mut header = DoipHeader::new(DoipPayloadType::AliveCheckRequest, 0);
        assert!(header.validate().is_ok());

        header.inverse_version = 0x00; // Invalid
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_payload_type_from_u16() {
        assert_eq!(
            DoipPayloadType::from_u16(0x0001),
            Some(DoipPayloadType::VehicleIdentificationRequest)
        );
        assert_eq!(
            DoipPayloadType::from_u16(0x8001),
            Some(DoipPayloadType::DiagnosticMessage)
        );
        assert_eq!(DoipPayloadType::from_u16(0xFFFF), None);
    }

    #[test]
    fn test_payload_type_is_request() {
        assert!(DoipPayloadType::VehicleIdentificationRequest.is_request());
        assert!(DoipPayloadType::RoutingActivationRequest.is_request());
        assert!(!DoipPayloadType::RoutingActivationResponse.is_request());
    }

    #[test]
    fn test_routing_activation_request() {
        let req = RoutingActivationRequest::new(0x0E80, RoutingActivationRequest::ACTIVATION_TYPE_DEFAULT);
        let bytes = req.to_bytes();
        assert!(bytes.len() >= RoutingActivationRequest::MIN_SIZE);

        let parsed = RoutingActivationRequest::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.source_address, 0x0E80);
        assert_eq!(parsed.activation_type, 0x00);
    }

    #[test]
    fn test_routing_activation_response() {
        let bytes = [
            0x0E, 0x80, // tester address
            0x10, 0x01, // entity address
            0x10,       // success code
            0x00, 0x00, 0x00, 0x00, // reserved
        ];
        let resp = RoutingActivationResponse::from_bytes(&bytes).unwrap();
        assert_eq!(resp.tester_address, 0x0E80);
        assert_eq!(resp.entity_address, 0x1001);
        assert!(resp.is_success());
    }

    #[test]
    fn test_diagnostic_message() {
        let msg = DiagnosticMessage::new(0x0E80, 0x1001, vec![0x10, 0x01]); // UDS: DiagnosticSessionControl
        let bytes = msg.to_bytes();

        let parsed = DiagnosticMessage::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.source_address, 0x0E80);
        assert_eq!(parsed.target_address, 0x1001);
        assert_eq!(parsed.user_data, vec![0x10, 0x01]);
    }

    #[test]
    fn test_vehicle_identification_response() {
        let mut bytes = vec![0u8; 33];
        // VIN
        bytes[0..17].copy_from_slice(b"WVWZZZ3CZWE123456");
        // Logical address
        bytes[17..19].copy_from_slice(&0x1001u16.to_be_bytes());
        // EID (MAC)
        bytes[19..25].copy_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        // GID
        bytes[25..31].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        // Further action
        bytes[31] = 0x00;
        // VIN/GID sync
        bytes[32] = 0x10;

        let resp = VehicleIdentificationResponse::from_bytes(&bytes).unwrap();
        assert_eq!(resp.vin_string(), "WVWZZZ3CZWE123456");
        assert_eq!(resp.logical_address, 0x1001);
        assert_eq!(resp.vin_gid_sync, Some(0x10));
    }

    #[test]
    fn test_alive_check_response() {
        let resp = AliveCheckResponse { source_address: 0x0E80 };
        let bytes = resp.to_bytes();
        assert_eq!(bytes.len(), 2);

        let parsed = AliveCheckResponse::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.source_address, 0x0E80);
    }

    #[test]
    fn test_doip_message_roundtrip() {
        let msg = DoipMessage::DiagnosticMessage(DiagnosticMessage::new(
            0x0E80,
            0x1001,
            vec![0x22, 0xF1, 0x90], // UDS: ReadDataByIdentifier
        ));

        let bytes = msg.to_bytes();
        let parsed = DoipMessage::from_bytes(&bytes).unwrap();

        match parsed {
            DoipMessage::DiagnosticMessage(dm) => {
                assert_eq!(dm.source_address, 0x0E80);
                assert_eq!(dm.target_address, 0x1001);
                assert_eq!(dm.user_data, vec![0x22, 0xF1, 0x90]);
            },
            _ => panic!("Expected DiagnosticMessage"),
        }
    }

    #[test]
    fn test_doip_client() {
        let mut client = DoipClient::new(0x0E80, 0x1001);
        assert_eq!(client.state, DoipClientState::Disconnected);

        // Create routing activation request
        let req = client.create_routing_activation_request();
        assert!(matches!(req, DoipMessage::RoutingActivationRequest(_)));

        // Simulate successful response
        let resp = DoipMessage::RoutingActivationResponse(RoutingActivationResponse {
            tester_address: 0x0E80,
            entity_address: 0x1001,
            response_code: RoutingActivationResponseCode::ActivatedSuccessfully,
            reserved: 0,
            oem_specific: None,
        });

        client.process_response(&resp).unwrap();
        assert!(client.is_routing_activated());
    }

    #[test]
    fn test_doip_client_diagnostic_message() {
        let client = DoipClient::new(0x0E80, 0x1001);
        let msg = client.create_diagnostic_message(vec![0x10, 0x01]);

        match msg {
            DoipMessage::DiagnosticMessage(dm) => {
                assert_eq!(dm.source_address, 0x0E80);
                assert_eq!(dm.target_address, 0x1001);
                assert_eq!(dm.user_data, vec![0x10, 0x01]);
            },
            _ => panic!("Expected DiagnosticMessage"),
        }
    }

    #[test]
    fn test_nack_codes() {
        assert_eq!(DoipNackCode::from_u8(0x00), Some(DoipNackCode::IncorrectPatternFormat));
        assert_eq!(DoipNackCode::from_u8(0x04), Some(DoipNackCode::InvalidPayloadLength));
        assert_eq!(DoipNackCode::from_u8(0xFF), None);
    }

    #[test]
    fn test_diagnostic_message_nack() {
        let bytes = [
            0x0E, 0x80, // source
            0x10, 0x01, // target
            0x03,       // unknown target address
        ];
        let nack = DiagnosticMessageNegativeAck::from_bytes(&bytes).unwrap();
        assert_eq!(nack.nack_code, DiagnosticMessageNackCode::UnknownTargetAddress);
    }
}
