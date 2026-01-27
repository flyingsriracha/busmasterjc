//! SOME/IP (Scalable service-Oriented MiddlewarE over IP) Protocol Implementation
//!
//! SOME/IP is defined in AUTOSAR and provides service-oriented communication
//! over IP networks in automotive systems.
//!
//! # Protocol Overview
//!
//! SOME/IP supports:
//! - Request/Response pattern
//! - Fire & Forget (notification)
//! - Publish/Subscribe via SOME/IP-SD
//! - Segmentation via SOME/IP-TP
//!
//! # Example
//!
//! ```
//! use busmaster_proto::someip::{SomeIpHeader, SomeIpMessageType};
//!
//! // Create a request message
//! let header = SomeIpHeader::new(0x1234, 0x0001, 0x5678, SomeIpMessageType::Request);
//! let bytes = header.to_bytes();
//! assert_eq!(bytes.len(), 16);
//! ```

use busmaster_core::{BusmasterError, Result};

/// SOME/IP protocol version
pub const SOMEIP_PROTOCOL_VERSION: u8 = 0x01;

/// SOME/IP interface version (default)
pub const SOMEIP_INTERFACE_VERSION: u8 = 0x01;

/// SOME/IP header size (16 bytes)
pub const SOMEIP_HEADER_SIZE: usize = 16;

/// SOME/IP-SD service ID
pub const SOMEIP_SD_SERVICE_ID: u16 = 0xFFFF;

/// SOME/IP-SD method ID
pub const SOMEIP_SD_METHOD_ID: u16 = 0x8100;

/// SOME/IP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SomeIpMessageType {
    /// Request expecting response
    Request = 0x00,
    /// Request without response (fire & forget)
    RequestNoReturn = 0x01,
    /// Notification (cyclic or event-based)
    Notification = 0x02,
    /// Response to a request
    Response = 0x80,
    /// Error response
    Error = 0x81,
    /// TP Request (segmented)
    TpRequest = 0x20,
    /// TP Request without response (segmented)
    TpRequestNoReturn = 0x21,
    /// TP Notification (segmented)
    TpNotification = 0x22,
    /// TP Response (segmented)
    TpResponse = 0xA0,
    /// TP Error (segmented)
    TpError = 0xA1,
}

impl SomeIpMessageType {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Request),
            0x01 => Some(Self::RequestNoReturn),
            0x02 => Some(Self::Notification),
            0x80 => Some(Self::Response),
            0x81 => Some(Self::Error),
            0x20 => Some(Self::TpRequest),
            0x21 => Some(Self::TpRequestNoReturn),
            0x22 => Some(Self::TpNotification),
            0xA0 => Some(Self::TpResponse),
            0xA1 => Some(Self::TpError),
            _ => None,
        }
    }

    /// Check if this is a request type
    #[must_use]
    pub fn is_request(&self) -> bool {
        matches!(
            self,
            Self::Request | Self::RequestNoReturn | Self::TpRequest | Self::TpRequestNoReturn
        )
    }

    /// Check if this is a response type
    #[must_use]
    pub fn is_response(&self) -> bool {
        matches!(self, Self::Response | Self::Error | Self::TpResponse | Self::TpError)
    }

    /// Check if this is a TP (segmented) message
    #[must_use]
    pub fn is_tp(&self) -> bool {
        matches!(
            self,
            Self::TpRequest
                | Self::TpRequestNoReturn
                | Self::TpNotification
                | Self::TpResponse
                | Self::TpError
        )
    }
}

/// SOME/IP return codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SomeIpReturnCode {
    /// No error
    Ok = 0x00,
    /// Not OK (unspecified error)
    NotOk = 0x01,
    /// Unknown service
    UnknownService = 0x02,
    /// Unknown method
    UnknownMethod = 0x03,
    /// Not ready
    NotReady = 0x04,
    /// Not reachable
    NotReachable = 0x05,
    /// Timeout
    Timeout = 0x06,
    /// Wrong protocol version
    WrongProtocolVersion = 0x07,
    /// Wrong interface version
    WrongInterfaceVersion = 0x08,
    /// Malformed message
    MalformedMessage = 0x09,
    /// Wrong message type
    WrongMessageType = 0x0A,
    /// E2E repeated
    E2ERepeated = 0x0B,
    /// E2E wrong sequence
    E2EWrongSequence = 0x0C,
    /// E2E not available
    E2ENotAvailable = 0x0D,
    /// E2E no new data
    E2ENoNewData = 0x0E,
}

impl SomeIpReturnCode {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Ok),
            0x01 => Some(Self::NotOk),
            0x02 => Some(Self::UnknownService),
            0x03 => Some(Self::UnknownMethod),
            0x04 => Some(Self::NotReady),
            0x05 => Some(Self::NotReachable),
            0x06 => Some(Self::Timeout),
            0x07 => Some(Self::WrongProtocolVersion),
            0x08 => Some(Self::WrongInterfaceVersion),
            0x09 => Some(Self::MalformedMessage),
            0x0A => Some(Self::WrongMessageType),
            0x0B => Some(Self::E2ERepeated),
            0x0C => Some(Self::E2EWrongSequence),
            0x0D => Some(Self::E2ENotAvailable),
            0x0E => Some(Self::E2ENoNewData),
            _ => None,
        }
    }

    /// Check if this is an error code
    #[must_use]
    pub fn is_error(&self) -> bool {
        !matches!(self, Self::Ok)
    }
}

/// SOME/IP Header (16 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SomeIpHeader {
    /// Service ID (16 bits)
    pub service_id: u16,
    /// Method ID (16 bits) - includes event ID for notifications
    pub method_id: u16,
    /// Length (32 bits) - payload length + 8 bytes (from request ID onwards)
    pub length: u32,
    /// Client ID (16 bits)
    pub client_id: u16,
    /// Session ID (16 bits)
    pub session_id: u16,
    /// Protocol version (8 bits)
    pub protocol_version: u8,
    /// Interface version (8 bits)
    pub interface_version: u8,
    /// Message type (8 bits)
    pub message_type: SomeIpMessageType,
    /// Return code (8 bits)
    pub return_code: SomeIpReturnCode,
}

impl SomeIpHeader {
    /// Create a new SOME/IP header
    #[must_use]
    pub fn new(service_id: u16, method_id: u16, client_id: u16, message_type: SomeIpMessageType) -> Self {
        Self {
            service_id,
            method_id,
            length: 8, // Minimum length (no payload)
            client_id,
            session_id: 1,
            protocol_version: SOMEIP_PROTOCOL_VERSION,
            interface_version: SOMEIP_INTERFACE_VERSION,
            message_type,
            return_code: SomeIpReturnCode::Ok,
        }
    }

    /// Create a response header from a request
    #[must_use]
    pub fn create_response(&self, return_code: SomeIpReturnCode) -> Self {
        Self {
            service_id: self.service_id,
            method_id: self.method_id,
            length: 8,
            client_id: self.client_id,
            session_id: self.session_id,
            protocol_version: self.protocol_version,
            interface_version: self.interface_version,
            message_type: if return_code.is_error() {
                SomeIpMessageType::Error
            } else {
                SomeIpMessageType::Response
            },
            return_code,
        }
    }

    /// Get the message ID (service_id << 16 | method_id)
    #[must_use]
    pub fn message_id(&self) -> u32 {
        ((self.service_id as u32) << 16) | (self.method_id as u32)
    }

    /// Get the request ID (client_id << 16 | session_id)
    #[must_use]
    pub fn request_id(&self) -> u32 {
        ((self.client_id as u32) << 16) | (self.session_id as u32)
    }

    /// Get payload length (length - 8)
    #[must_use]
    pub fn payload_length(&self) -> u32 {
        self.length.saturating_sub(8)
    }

    /// Set payload length
    pub fn set_payload_length(&mut self, payload_len: u32) {
        self.length = payload_len + 8;
    }

    /// Write to bytes (big-endian as per AUTOSAR)
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SOMEIP_HEADER_SIZE);
        bytes.extend_from_slice(&self.service_id.to_be_bytes());
        bytes.extend_from_slice(&self.method_id.to_be_bytes());
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.client_id.to_be_bytes());
        bytes.extend_from_slice(&self.session_id.to_be_bytes());
        bytes.push(self.protocol_version);
        bytes.push(self.interface_version);
        bytes.push(self.message_type as u8);
        bytes.push(self.return_code as u8);
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < SOMEIP_HEADER_SIZE {
            return Err(BusmasterError::Parse {
                message: format!(
                    "SOME/IP header too short: {} bytes, expected {}",
                    bytes.len(),
                    SOMEIP_HEADER_SIZE
                ),
            });
        }

        let service_id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let method_id = u16::from_be_bytes([bytes[2], bytes[3]]);
        let length = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let client_id = u16::from_be_bytes([bytes[8], bytes[9]]);
        let session_id = u16::from_be_bytes([bytes[10], bytes[11]]);
        let protocol_version = bytes[12];
        let interface_version = bytes[13];

        let message_type = SomeIpMessageType::from_u8(bytes[14]).ok_or_else(|| {
            BusmasterError::Parse {
                message: format!("Unknown SOME/IP message type: 0x{:02X}", bytes[14]),
            }
        })?;

        let return_code = SomeIpReturnCode::from_u8(bytes[15]).ok_or_else(|| {
            BusmasterError::Parse {
                message: format!("Unknown SOME/IP return code: 0x{:02X}", bytes[15]),
            }
        })?;

        Ok(Self {
            service_id,
            method_id,
            length,
            client_id,
            session_id,
            protocol_version,
            interface_version,
            message_type,
            return_code,
        })
    }
}


/// SOME/IP Message (header + payload)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SomeIpMessage {
    /// Message header
    pub header: SomeIpHeader,
    /// Payload data
    pub payload: Vec<u8>,
}

impl SomeIpMessage {
    /// Create a new SOME/IP message
    #[must_use]
    pub fn new(mut header: SomeIpHeader, payload: Vec<u8>) -> Self {
        header.set_payload_length(payload.len() as u32);
        Self { header, payload }
    }

    /// Create a request message
    #[must_use]
    pub fn request(service_id: u16, method_id: u16, client_id: u16, payload: Vec<u8>) -> Self {
        let header = SomeIpHeader::new(service_id, method_id, client_id, SomeIpMessageType::Request);
        Self::new(header, payload)
    }

    /// Create a notification message
    #[must_use]
    pub fn notification(service_id: u16, event_id: u16, payload: Vec<u8>) -> Self {
        let header = SomeIpHeader::new(service_id, event_id, 0, SomeIpMessageType::Notification);
        Self::new(header, payload)
    }

    /// Create a response from a request
    #[must_use]
    pub fn response(&self, return_code: SomeIpReturnCode, payload: Vec<u8>) -> Self {
        let header = self.header.create_response(return_code);
        Self::new(header, payload)
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = SomeIpHeader::from_bytes(bytes)?;
        let payload_len = header.payload_length() as usize;
        let total_len = SOMEIP_HEADER_SIZE + payload_len;

        if bytes.len() < total_len {
            return Err(BusmasterError::Parse {
                message: format!(
                    "SOME/IP message too short: {} bytes, expected {}",
                    bytes.len(),
                    total_len
                ),
            });
        }

        let payload = bytes[SOMEIP_HEADER_SIZE..total_len].to_vec();
        Ok(Self { header, payload })
    }
}

// ============================================================================
// SOME/IP-SD (Service Discovery)
// ============================================================================

/// SOME/IP-SD entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SdEntryType {
    /// Find service
    FindService = 0x00,
    /// Offer service
    OfferService = 0x01,
    /// Stop offer service
    StopOfferService = 0x81,
    /// Subscribe eventgroup
    SubscribeEventgroup = 0x06,
    /// Stop subscribe eventgroup
    StopSubscribeEventgroup = 0x86,
    /// Subscribe eventgroup ACK
    SubscribeEventgroupAck = 0x07,
    /// Subscribe eventgroup NACK
    SubscribeEventgroupNack = 0x87,
}

impl SdEntryType {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::FindService),
            0x01 => Some(Self::OfferService),
            0x81 => Some(Self::StopOfferService),
            0x06 => Some(Self::SubscribeEventgroup),
            0x86 => Some(Self::StopSubscribeEventgroup),
            0x07 => Some(Self::SubscribeEventgroupAck),
            0x87 => Some(Self::SubscribeEventgroupNack),
            _ => None,
        }
    }

    /// Check if this is a service entry
    #[must_use]
    pub fn is_service_entry(&self) -> bool {
        matches!(self, Self::FindService | Self::OfferService | Self::StopOfferService)
    }

    /// Check if this is an eventgroup entry
    #[must_use]
    pub fn is_eventgroup_entry(&self) -> bool {
        matches!(
            self,
            Self::SubscribeEventgroup
                | Self::StopSubscribeEventgroup
                | Self::SubscribeEventgroupAck
                | Self::SubscribeEventgroupNack
        )
    }
}

/// SOME/IP-SD option types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SdOptionType {
    /// Configuration option
    Configuration = 0x01,
    /// Load balancing option
    LoadBalancing = 0x02,
    /// IPv4 endpoint option
    Ipv4Endpoint = 0x04,
    /// IPv6 endpoint option
    Ipv6Endpoint = 0x06,
    /// IPv4 multicast option
    Ipv4Multicast = 0x14,
    /// IPv6 multicast option
    Ipv6Multicast = 0x16,
    /// IPv4 SD endpoint option
    Ipv4SdEndpoint = 0x24,
    /// IPv6 SD endpoint option
    Ipv6SdEndpoint = 0x26,
}

impl SdOptionType {
    /// Create from raw u8 value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Configuration),
            0x02 => Some(Self::LoadBalancing),
            0x04 => Some(Self::Ipv4Endpoint),
            0x06 => Some(Self::Ipv6Endpoint),
            0x14 => Some(Self::Ipv4Multicast),
            0x16 => Some(Self::Ipv6Multicast),
            0x24 => Some(Self::Ipv4SdEndpoint),
            0x26 => Some(Self::Ipv6SdEndpoint),
            _ => None,
        }
    }
}

/// SOME/IP-SD Service Entry (16 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdServiceEntry {
    /// Entry type
    pub entry_type: SdEntryType,
    /// Index to first option run
    pub index1: u8,
    /// Index to second option run
    pub index2: u8,
    /// Number of options in first run
    pub num_opt1: u8,
    /// Number of options in second run
    pub num_opt2: u8,
    /// Service ID
    pub service_id: u16,
    /// Instance ID
    pub instance_id: u16,
    /// Major version
    pub major_version: u8,
    /// TTL (time to live in seconds)
    pub ttl: u32,
    /// Minor version
    pub minor_version: u32,
}

impl SdServiceEntry {
    /// Entry size in bytes
    pub const SIZE: usize = 16;

    /// Create a find service entry
    #[must_use]
    pub fn find_service(service_id: u16, instance_id: u16, major_version: u8, minor_version: u32) -> Self {
        Self {
            entry_type: SdEntryType::FindService,
            index1: 0,
            index2: 0,
            num_opt1: 0,
            num_opt2: 0,
            service_id,
            instance_id,
            major_version,
            ttl: 3, // Default TTL for find
            minor_version,
        }
    }

    /// Create an offer service entry
    #[must_use]
    pub fn offer_service(
        service_id: u16,
        instance_id: u16,
        major_version: u8,
        minor_version: u32,
        ttl: u32,
    ) -> Self {
        Self {
            entry_type: SdEntryType::OfferService,
            index1: 0,
            index2: 0,
            num_opt1: 0,
            num_opt2: 0,
            service_id,
            instance_id,
            major_version,
            ttl,
            minor_version,
        }
    }

    /// Create a stop offer service entry
    #[must_use]
    pub fn stop_offer_service(service_id: u16, instance_id: u16, major_version: u8, minor_version: u32) -> Self {
        Self {
            entry_type: SdEntryType::StopOfferService,
            index1: 0,
            index2: 0,
            num_opt1: 0,
            num_opt2: 0,
            service_id,
            instance_id,
            major_version,
            ttl: 0, // TTL 0 means stop
            minor_version,
        }
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        bytes.push(self.entry_type as u8);
        bytes.push(self.index1);
        bytes.push(self.index2);
        bytes.push((self.num_opt1 << 4) | (self.num_opt2 & 0x0F));
        bytes.extend_from_slice(&self.service_id.to_be_bytes());
        bytes.extend_from_slice(&self.instance_id.to_be_bytes());
        bytes.push(self.major_version);
        // TTL is 24 bits
        bytes.push(((self.ttl >> 16) & 0xFF) as u8);
        bytes.push(((self.ttl >> 8) & 0xFF) as u8);
        bytes.push((self.ttl & 0xFF) as u8);
        bytes.extend_from_slice(&self.minor_version.to_be_bytes());
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(BusmasterError::Parse {
                message: format!("SD service entry too short: {} bytes", bytes.len()),
            });
        }

        let entry_type = SdEntryType::from_u8(bytes[0]).ok_or_else(|| BusmasterError::Parse {
            message: format!("Unknown SD entry type: 0x{:02X}", bytes[0]),
        })?;

        let index1 = bytes[1];
        let index2 = bytes[2];
        let num_opt1 = (bytes[3] >> 4) & 0x0F;
        let num_opt2 = bytes[3] & 0x0F;
        let service_id = u16::from_be_bytes([bytes[4], bytes[5]]);
        let instance_id = u16::from_be_bytes([bytes[6], bytes[7]]);
        let major_version = bytes[8];
        let ttl = ((bytes[9] as u32) << 16) | ((bytes[10] as u32) << 8) | (bytes[11] as u32);
        let minor_version = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        Ok(Self {
            entry_type,
            index1,
            index2,
            num_opt1,
            num_opt2,
            service_id,
            instance_id,
            major_version,
            ttl,
            minor_version,
        })
    }
}

/// SOME/IP-SD Eventgroup Entry (16 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdEventgroupEntry {
    /// Entry type
    pub entry_type: SdEntryType,
    /// Index to first option run
    pub index1: u8,
    /// Index to second option run
    pub index2: u8,
    /// Number of options in first run
    pub num_opt1: u8,
    /// Number of options in second run
    pub num_opt2: u8,
    /// Service ID
    pub service_id: u16,
    /// Instance ID
    pub instance_id: u16,
    /// Major version
    pub major_version: u8,
    /// TTL (time to live in seconds)
    pub ttl: u32,
    /// Counter (4 bits)
    pub counter: u8,
    /// Eventgroup ID
    pub eventgroup_id: u16,
}

impl SdEventgroupEntry {
    /// Entry size in bytes
    pub const SIZE: usize = 16;

    /// Create a subscribe eventgroup entry
    #[must_use]
    pub fn subscribe(
        service_id: u16,
        instance_id: u16,
        major_version: u8,
        eventgroup_id: u16,
        ttl: u32,
    ) -> Self {
        Self {
            entry_type: SdEntryType::SubscribeEventgroup,
            index1: 0,
            index2: 0,
            num_opt1: 0,
            num_opt2: 0,
            service_id,
            instance_id,
            major_version,
            ttl,
            counter: 0,
            eventgroup_id,
        }
    }

    /// Create a subscribe ACK entry
    #[must_use]
    pub fn subscribe_ack(
        service_id: u16,
        instance_id: u16,
        major_version: u8,
        eventgroup_id: u16,
        ttl: u32,
    ) -> Self {
        Self {
            entry_type: SdEntryType::SubscribeEventgroupAck,
            index1: 0,
            index2: 0,
            num_opt1: 0,
            num_opt2: 0,
            service_id,
            instance_id,
            major_version,
            ttl,
            counter: 0,
            eventgroup_id,
        }
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        bytes.push(self.entry_type as u8);
        bytes.push(self.index1);
        bytes.push(self.index2);
        bytes.push((self.num_opt1 << 4) | (self.num_opt2 & 0x0F));
        bytes.extend_from_slice(&self.service_id.to_be_bytes());
        bytes.extend_from_slice(&self.instance_id.to_be_bytes());
        bytes.push(self.major_version);
        // TTL is 24 bits
        bytes.push(((self.ttl >> 16) & 0xFF) as u8);
        bytes.push(((self.ttl >> 8) & 0xFF) as u8);
        bytes.push((self.ttl & 0xFF) as u8);
        // Reserved (4 bits) + counter (4 bits)
        bytes.push(self.counter & 0x0F);
        bytes.push(0); // Reserved
        bytes.extend_from_slice(&self.eventgroup_id.to_be_bytes());
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(BusmasterError::Parse {
                message: format!("SD eventgroup entry too short: {} bytes", bytes.len()),
            });
        }

        let entry_type = SdEntryType::from_u8(bytes[0]).ok_or_else(|| BusmasterError::Parse {
            message: format!("Unknown SD entry type: 0x{:02X}", bytes[0]),
        })?;

        let index1 = bytes[1];
        let index2 = bytes[2];
        let num_opt1 = (bytes[3] >> 4) & 0x0F;
        let num_opt2 = bytes[3] & 0x0F;
        let service_id = u16::from_be_bytes([bytes[4], bytes[5]]);
        let instance_id = u16::from_be_bytes([bytes[6], bytes[7]]);
        let major_version = bytes[8];
        let ttl = ((bytes[9] as u32) << 16) | ((bytes[10] as u32) << 8) | (bytes[11] as u32);
        let counter = bytes[12] & 0x0F;
        let eventgroup_id = u16::from_be_bytes([bytes[14], bytes[15]]);

        Ok(Self {
            entry_type,
            index1,
            index2,
            num_opt1,
            num_opt2,
            service_id,
            instance_id,
            major_version,
            ttl,
            counter,
            eventgroup_id,
        })
    }
}

/// IPv4 Endpoint Option (12 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4EndpointOption {
    /// IPv4 address
    pub address: [u8; 4],
    /// Protocol (UDP = 0x11, TCP = 0x06)
    pub protocol: u8,
    /// Port number
    pub port: u16,
}

impl Ipv4EndpointOption {
    /// Option size in bytes
    pub const SIZE: usize = 12;

    /// UDP protocol
    pub const PROTOCOL_UDP: u8 = 0x11;
    /// TCP protocol
    pub const PROTOCOL_TCP: u8 = 0x06;

    /// Create a new IPv4 endpoint option
    #[must_use]
    pub fn new(address: [u8; 4], protocol: u8, port: u16) -> Self {
        Self { address, protocol, port }
    }

    /// Create a UDP endpoint
    #[must_use]
    pub fn udp(address: [u8; 4], port: u16) -> Self {
        Self::new(address, Self::PROTOCOL_UDP, port)
    }

    /// Create a TCP endpoint
    #[must_use]
    pub fn tcp(address: [u8; 4], port: u16) -> Self {
        Self::new(address, Self::PROTOCOL_TCP, port)
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        // Length (2 bytes) - option length excluding length field
        bytes.extend_from_slice(&9u16.to_be_bytes());
        // Type
        bytes.push(SdOptionType::Ipv4Endpoint as u8);
        // Reserved
        bytes.push(0);
        // IPv4 address
        bytes.extend_from_slice(&self.address);
        // Reserved
        bytes.push(0);
        // Protocol
        bytes.push(self.protocol);
        // Port
        bytes.extend_from_slice(&self.port.to_be_bytes());
        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(BusmasterError::Parse {
                message: format!("IPv4 endpoint option too short: {} bytes", bytes.len()),
            });
        }

        let mut address = [0u8; 4];
        address.copy_from_slice(&bytes[4..8]);
        let protocol = bytes[9];
        let port = u16::from_be_bytes([bytes[10], bytes[11]]);

        Ok(Self { address, protocol, port })
    }
}


// ============================================================================
// SOME/IP-TP (Transport Protocol for segmentation)
// ============================================================================

/// SOME/IP-TP Header extension (4 bytes after standard header)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SomeIpTpHeader {
    /// Offset in 16-byte units
    pub offset: u32,
    /// More segments flag
    pub more_segments: bool,
}

impl SomeIpTpHeader {
    /// TP header size
    pub const SIZE: usize = 4;

    /// Create a new TP header
    #[must_use]
    pub fn new(offset: u32, more_segments: bool) -> Self {
        Self { offset, more_segments }
    }

    /// Get the byte offset
    #[must_use]
    pub fn byte_offset(&self) -> u32 {
        self.offset * 16
    }

    /// Write to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut value = self.offset & 0x0FFFFFFF;
        if self.more_segments {
            value |= 0x10000000;
        }
        value.to_be_bytes().to_vec()
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(BusmasterError::Parse {
                message: format!("SOME/IP-TP header too short: {} bytes", bytes.len()),
            });
        }

        let value = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let more_segments = (value & 0x10000000) != 0;
        let offset = value & 0x0FFFFFFF;

        Ok(Self { offset, more_segments })
    }
}

/// SOME/IP-TP Session for reassembly
#[derive(Debug)]
pub struct TpSession {
    /// Service ID
    pub service_id: u16,
    /// Method ID
    pub method_id: u16,
    /// Client ID
    pub client_id: u16,
    /// Session ID
    pub session_id: u16,
    /// Reassembly buffer
    buffer: Vec<u8>,
    /// Expected next offset
    next_offset: u32,
    /// Complete flag
    complete: bool,
}

impl TpSession {
    /// Create a new TP session
    #[must_use]
    pub fn new(header: &SomeIpHeader) -> Self {
        Self {
            service_id: header.service_id,
            method_id: header.method_id,
            client_id: header.client_id,
            session_id: header.session_id,
            buffer: Vec::new(),
            next_offset: 0,
            complete: false,
        }
    }

    /// Check if this session matches a header
    #[must_use]
    pub fn matches(&self, header: &SomeIpHeader) -> bool {
        self.service_id == header.service_id
            && self.method_id == header.method_id
            && self.client_id == header.client_id
            && self.session_id == header.session_id
    }

    /// Add a segment to the session
    pub fn add_segment(&mut self, tp_header: &SomeIpTpHeader, data: &[u8]) -> Result<()> {
        if tp_header.offset != self.next_offset {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "TP segment out of order: expected offset {}, got {}",
                    self.next_offset, tp_header.offset
                ),
            });
        }

        self.buffer.extend_from_slice(data);
        self.next_offset = tp_header.offset + (data.len() as u32).div_ceil(16);
        self.complete = !tp_header.more_segments;

        Ok(())
    }

    /// Check if reassembly is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Get the reassembled data
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Take the reassembled data
    #[must_use]
    pub fn take_data(self) -> Vec<u8> {
        self.buffer
    }
}

// ============================================================================
// SOME/IP Client
// ============================================================================

/// SOME/IP Client for managing service communication
#[derive(Debug)]
pub struct SomeIpClient {
    /// Client ID
    pub client_id: u16,
    /// Next session ID
    next_session_id: u16,
    /// Known services (service_id -> instance_id)
    known_services: std::collections::HashMap<u16, u16>,
    /// Active subscriptions (service_id, eventgroup_id)
    subscriptions: Vec<(u16, u16)>,
}

impl SomeIpClient {
    /// Create a new SOME/IP client
    #[must_use]
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            next_session_id: 1,
            known_services: std::collections::HashMap::new(),
            subscriptions: Vec::new(),
        }
    }

    /// Get the next session ID
    fn next_session(&mut self) -> u16 {
        let session = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        if self.next_session_id == 0 {
            self.next_session_id = 1;
        }
        session
    }

    /// Create a request message
    #[must_use]
    pub fn create_request(&mut self, service_id: u16, method_id: u16, payload: Vec<u8>) -> SomeIpMessage {
        let mut header = SomeIpHeader::new(service_id, method_id, self.client_id, SomeIpMessageType::Request);
        header.session_id = self.next_session();
        SomeIpMessage::new(header, payload)
    }

    /// Create a fire-and-forget request
    #[must_use]
    pub fn create_request_no_return(&mut self, service_id: u16, method_id: u16, payload: Vec<u8>) -> SomeIpMessage {
        let mut header = SomeIpHeader::new(service_id, method_id, self.client_id, SomeIpMessageType::RequestNoReturn);
        header.session_id = self.next_session();
        SomeIpMessage::new(header, payload)
    }

    /// Create a find service SD entry
    #[must_use]
    pub fn create_find_service(&self, service_id: u16, major_version: u8, minor_version: u32) -> SdServiceEntry {
        SdServiceEntry::find_service(service_id, 0xFFFF, major_version, minor_version)
    }

    /// Create a subscribe eventgroup SD entry
    #[must_use]
    pub fn create_subscribe(&self, service_id: u16, instance_id: u16, eventgroup_id: u16, ttl: u32) -> SdEventgroupEntry {
        SdEventgroupEntry::subscribe(service_id, instance_id, 0xFF, eventgroup_id, ttl)
    }

    /// Register a discovered service
    pub fn register_service(&mut self, service_id: u16, instance_id: u16) {
        self.known_services.insert(service_id, instance_id);
    }

    /// Check if a service is known
    #[must_use]
    pub fn is_service_known(&self, service_id: u16) -> bool {
        self.known_services.contains_key(&service_id)
    }

    /// Get instance ID for a service
    #[must_use]
    pub fn get_instance_id(&self, service_id: u16) -> Option<u16> {
        self.known_services.get(&service_id).copied()
    }

    /// Add a subscription
    pub fn add_subscription(&mut self, service_id: u16, eventgroup_id: u16) {
        if !self.subscriptions.contains(&(service_id, eventgroup_id)) {
            self.subscriptions.push((service_id, eventgroup_id));
        }
    }

    /// Remove a subscription
    pub fn remove_subscription(&mut self, service_id: u16, eventgroup_id: u16) {
        self.subscriptions.retain(|&(s, e)| s != service_id || e != eventgroup_id);
    }

    /// Check if subscribed to an eventgroup
    #[must_use]
    pub fn is_subscribed(&self, service_id: u16, eventgroup_id: u16) -> bool {
        self.subscriptions.contains(&(service_id, eventgroup_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_someip_header_creation() {
        let header = SomeIpHeader::new(0x1234, 0x0001, 0x5678, SomeIpMessageType::Request);
        assert_eq!(header.service_id, 0x1234);
        assert_eq!(header.method_id, 0x0001);
        assert_eq!(header.client_id, 0x5678);
        assert_eq!(header.message_type, SomeIpMessageType::Request);
        assert_eq!(header.length, 8); // Minimum length
    }

    #[test]
    fn test_someip_header_roundtrip() {
        let header = SomeIpHeader::new(0x1234, 0x0001, 0x5678, SomeIpMessageType::Request);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), SOMEIP_HEADER_SIZE);

        let parsed = SomeIpHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.service_id, header.service_id);
        assert_eq!(parsed.method_id, header.method_id);
        assert_eq!(parsed.client_id, header.client_id);
        assert_eq!(parsed.message_type, header.message_type);
    }

    #[test]
    fn test_someip_message_roundtrip() {
        let msg = SomeIpMessage::request(0x1234, 0x0001, 0x5678, vec![0x01, 0x02, 0x03]);
        let bytes = msg.to_bytes();

        let parsed = SomeIpMessage::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.header.service_id, 0x1234);
        assert_eq!(parsed.payload, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_someip_response_creation() {
        let request = SomeIpMessage::request(0x1234, 0x0001, 0x5678, vec![]);
        let response = request.response(SomeIpReturnCode::Ok, vec![0xAA, 0xBB]);

        assert_eq!(response.header.message_type, SomeIpMessageType::Response);
        assert_eq!(response.header.return_code, SomeIpReturnCode::Ok);
        assert_eq!(response.header.client_id, 0x5678);
        assert_eq!(response.payload, vec![0xAA, 0xBB]);
    }

    #[test]
    fn test_someip_error_response() {
        let request = SomeIpMessage::request(0x1234, 0x0001, 0x5678, vec![]);
        let response = request.response(SomeIpReturnCode::UnknownMethod, vec![]);

        assert_eq!(response.header.message_type, SomeIpMessageType::Error);
        assert_eq!(response.header.return_code, SomeIpReturnCode::UnknownMethod);
    }

    #[test]
    fn test_message_type_classification() {
        assert!(SomeIpMessageType::Request.is_request());
        assert!(SomeIpMessageType::RequestNoReturn.is_request());
        assert!(!SomeIpMessageType::Response.is_request());

        assert!(SomeIpMessageType::Response.is_response());
        assert!(SomeIpMessageType::Error.is_response());
        assert!(!SomeIpMessageType::Request.is_response());

        assert!(SomeIpMessageType::TpRequest.is_tp());
        assert!(!SomeIpMessageType::Request.is_tp());
    }

    #[test]
    fn test_return_code_classification() {
        assert!(!SomeIpReturnCode::Ok.is_error());
        assert!(SomeIpReturnCode::NotOk.is_error());
        assert!(SomeIpReturnCode::UnknownService.is_error());
    }

    #[test]
    fn test_sd_service_entry_roundtrip() {
        let entry = SdServiceEntry::offer_service(0x1234, 0x0001, 1, 0x00000001, 3600);
        let bytes = entry.to_bytes();
        assert_eq!(bytes.len(), SdServiceEntry::SIZE);

        let parsed = SdServiceEntry::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.service_id, 0x1234);
        assert_eq!(parsed.instance_id, 0x0001);
        assert_eq!(parsed.major_version, 1);
        assert_eq!(parsed.ttl, 3600);
    }

    #[test]
    fn test_sd_eventgroup_entry_roundtrip() {
        let entry = SdEventgroupEntry::subscribe(0x1234, 0x0001, 1, 0x0001, 3600);
        let bytes = entry.to_bytes();
        assert_eq!(bytes.len(), SdEventgroupEntry::SIZE);

        let parsed = SdEventgroupEntry::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.service_id, 0x1234);
        assert_eq!(parsed.eventgroup_id, 0x0001);
        assert_eq!(parsed.ttl, 3600);
    }

    #[test]
    fn test_ipv4_endpoint_option() {
        let opt = Ipv4EndpointOption::udp([192, 168, 1, 100], 30500);
        let bytes = opt.to_bytes();
        assert_eq!(bytes.len(), Ipv4EndpointOption::SIZE);

        let parsed = Ipv4EndpointOption::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.address, [192, 168, 1, 100]);
        assert_eq!(parsed.protocol, Ipv4EndpointOption::PROTOCOL_UDP);
        assert_eq!(parsed.port, 30500);
    }

    #[test]
    fn test_tp_header_roundtrip() {
        let tp = SomeIpTpHeader::new(10, true);
        let bytes = tp.to_bytes();
        assert_eq!(bytes.len(), SomeIpTpHeader::SIZE);

        let parsed = SomeIpTpHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.offset, 10);
        assert!(parsed.more_segments);
        assert_eq!(parsed.byte_offset(), 160);
    }

    #[test]
    fn test_tp_session() {
        let header = SomeIpHeader::new(0x1234, 0x0001, 0x5678, SomeIpMessageType::TpRequest);
        let mut session = TpSession::new(&header);

        // First segment
        let tp1 = SomeIpTpHeader::new(0, true);
        session.add_segment(&tp1, &[0x01; 16]).unwrap();
        assert!(!session.is_complete());

        // Second segment (final)
        let tp2 = SomeIpTpHeader::new(1, false);
        session.add_segment(&tp2, &[0x02; 8]).unwrap();
        assert!(session.is_complete());

        assert_eq!(session.data().len(), 24);
    }

    #[test]
    fn test_someip_client() {
        let mut client = SomeIpClient::new(0x5678);

        // Create request
        let msg = client.create_request(0x1234, 0x0001, vec![0x01, 0x02]);
        assert_eq!(msg.header.client_id, 0x5678);
        assert_eq!(msg.header.session_id, 1);

        // Second request should have incremented session
        let msg2 = client.create_request(0x1234, 0x0001, vec![]);
        assert_eq!(msg2.header.session_id, 2);
    }

    #[test]
    fn test_someip_client_services() {
        let mut client = SomeIpClient::new(0x5678);

        assert!(!client.is_service_known(0x1234));

        client.register_service(0x1234, 0x0001);
        assert!(client.is_service_known(0x1234));
        assert_eq!(client.get_instance_id(0x1234), Some(0x0001));
    }

    #[test]
    fn test_someip_client_subscriptions() {
        let mut client = SomeIpClient::new(0x5678);

        assert!(!client.is_subscribed(0x1234, 0x0001));

        client.add_subscription(0x1234, 0x0001);
        assert!(client.is_subscribed(0x1234, 0x0001));

        client.remove_subscription(0x1234, 0x0001);
        assert!(!client.is_subscribed(0x1234, 0x0001));
    }

    #[test]
    fn test_message_id_and_request_id() {
        let header = SomeIpHeader::new(0x1234, 0x5678, 0xABCD, SomeIpMessageType::Request);
        assert_eq!(header.message_id(), 0x12345678);
        assert_eq!(header.request_id(), 0xABCD0001); // session_id defaults to 1
    }

    #[test]
    fn test_notification_message() {
        let msg = SomeIpMessage::notification(0x1234, 0x8001, vec![0x01, 0x02, 0x03]);
        assert_eq!(msg.header.message_type, SomeIpMessageType::Notification);
        assert_eq!(msg.header.client_id, 0); // Notifications have no client
    }
}
