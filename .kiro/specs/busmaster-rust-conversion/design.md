# BUSMASTER Rust Conversion - Design Document

**Project Name:** BUSMASTER Rust Conversion (macOS First)  
**Project Owner:** JC  
**Version:** 3.0  
**Date:** January 2026  
**Status:** APPROVED FOR AI DEVELOPMENT  
**Development Model:** AI-ONLY

---

## 1. Design Overview

### 1.1 Architecture Philosophy

1. **MVP First** - Build working software before optimizing
2. **Modular Design** - Independent crates with clear boundaries
3. **Type Safety** - Leverage Rust's type system for correctness
4. **Explicit over Implicit** - Clear code that AI can understand and modify
5. **Test-Driven** - Tests define behavior, implementation follows
6. **Platform Abstraction** - Isolate platform-specific code early
7. **Cloud-Ready** - Design for headless and serverless from start
8. **AI-Friendly** - Structured data for AI analysis integration

### 1.2 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Application Layer                                   │
│  busmaster-cli (MVP) → busmaster-tui → busmaster-gui → busmaster-web    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      API Layer (Phase 3)                                 │
│  busmaster-api: REST API, WebSocket, OpenAPI                            │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      AI Integration Layer (Phase 3)                      │
│  busmaster-ai: OpenAI/Azure OpenAI, Analysis, Anomaly Detection         │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      Business Logic Layer                                │
│  busmaster-engine: orchestration, filtering, logging, diagnostics       │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────┬───────────────┬───────────────┬─────────────────────────┐
│ busmaster-db  │busmaster-proto│ busmaster-log │ busmaster-diag          │
│ DBC/DBF/ARXML │CAN/LIN/Ethernet│ ASC/BLF/MDF4 │ UDS/OBD-II/KWP          │
└───────────────┴───────────────┴───────────────┴─────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      Hardware Abstraction Layer                          │
│  busmaster-dil: traits, busmaster-hardware: implementations             │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      Platform Layer                                      │
│  busmaster-platform: OS-specific USB, timing, networking                │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Crate Structure (Full Vision)

```
busmaster-rust/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── busmaster-core/           # Core types (MVP)
│   ├── busmaster-proto/          # Protocol implementations
│   │   ├── can/                  # CAN, CAN FD (MVP)
│   │   ├── lin/                  # LIN (Phase 3)
│   │   ├── flexray/              # FlexRay (Phase 5)
│   │   ├── j1939/                # J1939 (Phase 2)
│   │   ├── ethernet/             # DoIP, SOME/IP (Phase 2)
│   │   └── xcp/                  # XCP/CCP (Phase 3)
│   ├── busmaster-diag/           # Diagnostic protocols (Phase 2)
│   │   ├── uds/                  # UDS - ISO 14229
│   │   ├── obd/                  # OBD-II
│   │   └── kwp/                  # KWP2000
│   ├── busmaster-db/             # Database parsing
│   │   ├── dbc/                  # DBC parser (MVP)
│   │   ├── dbf/                  # DBF parser (Phase 3)
│   │   ├── ldf/                  # LDF parser (Phase 3)
│   │   ├── arxml/                # ARXML parser (Phase 3)
│   │   ├── odx/                  # ODX parser (Phase 3)
│   │   ├── a2l/                  # A2L parser (Phase 3)
│   │   └── fibex/                # FIBEX parser (Phase 5)
│   ├── busmaster-log/            # Log formats
│   │   ├── asc/                  # ASC format (MVP)
│   │   ├── blf/                  # BLF format (Phase 2)
│   │   ├── mdf4/                 # MDF4 format (Phase 3)
│   │   └── pcap/                 # PCAP for Ethernet (Phase 2)
│   ├── busmaster-dil/            # Driver interface traits (MVP)
│   ├── busmaster-hardware/       # Hardware drivers
│   │   ├── stub/                 # Stub driver (MVP)
│   │   ├── peak/                 # PEAK USB (MVP)
│   │   ├── vector/               # Vector XL (Phase 2)
│   │   ├── kvaser/               # Kvaser (Phase 3)
│   │   ├── etas/                 # ETAS BOA (Phase 3)
│   │   ├── intrepid/             # Intrepid neoVI (Phase 4)
│   │   └── socketcan/            # SocketCAN (Phase 4)
│   ├── busmaster-platform/       # Platform abstraction (MVP)
│   ├── busmaster-engine/         # Business logic (MVP)
│   ├── busmaster-filter/         # Message filtering (MVP)
│   ├── busmaster-api/            # REST API (Phase 3)
│   ├── busmaster-ai/             # AI integration (Phase 3)
│   ├── busmaster-cli/            # CLI application (MVP)
│   ├── busmaster-tui/            # TUI application (MVP)
│   ├── busmaster-gui/            # GUI application (Phase 3)
│   └── busmaster-web/            # Web UI (Phase 3)
├── tests/                        # Integration tests
├── benches/                      # Benchmarks
└── examples/                     # Usage examples
```

---

## 3. Automotive Ethernet Module Design (NEW)

### 3.1 DoIP Implementation

```rust
// crates/busmaster-proto/ethernet/src/doip.rs

use std::net::{TcpStream, UdpSocket, SocketAddr};
use busmaster_core::{Result, BusmasterError};

/// DoIP protocol version
pub const DOIP_PROTOCOL_VERSION: u8 = 0x02;

/// DoIP payload types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum DoipPayloadType {
    GenericNegativeAck = 0x0000,
    VehicleIdentificationRequest = 0x0001,
    VehicleIdentificationRequestEid = 0x0002,
    VehicleIdentificationRequestVin = 0x0003,
    VehicleAnnouncementMessage = 0x0004,
    RoutingActivationRequest = 0x0005,
    RoutingActivationResponse = 0x0006,
    AliveCheckRequest = 0x0007,
    AliveCheckResponse = 0x0008,
    EntityStatusRequest = 0x4001,
    EntityStatusResponse = 0x4002,
    DiagnosticPowerModeRequest = 0x4003,
    DiagnosticPowerModeResponse = 0x4004,
    DiagnosticMessage = 0x8001,
    DiagnosticMessagePositiveAck = 0x8002,
    DiagnosticMessageNegativeAck = 0x8003,
}

/// DoIP header structure
#[derive(Debug, Clone)]
pub struct DoipHeader {
    pub protocol_version: u8,
    pub inverse_version: u8,
    pub payload_type: DoipPayloadType,
    pub payload_length: u32,
}

impl DoipHeader {
    pub fn new(payload_type: DoipPayloadType, payload_length: u32) -> Self {
        Self {
            protocol_version: DOIP_PROTOCOL_VERSION,
            inverse_version: !DOIP_PROTOCOL_VERSION,
            payload_type,
            payload_length,
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);
        bytes.push(self.protocol_version);
        bytes.push(self.inverse_version);
        bytes.extend_from_slice(&(self.payload_type as u16).to_be_bytes());
        bytes.extend_from_slice(&self.payload_length.to_be_bytes());
        bytes
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(BusmasterError::Protocol {
                message: "DoIP header too short".into(),
            });
        }
        Ok(Self {
            protocol_version: data[0],
            inverse_version: data[1],
            payload_type: DoipPayloadType::try_from(
                u16::from_be_bytes([data[2], data[3]])
            )?,
            payload_length: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
        })
    }
}

/// DoIP client for diagnostic communication
pub struct DoipClient {
    tcp_stream: Option<TcpStream>,
    udp_socket: UdpSocket,
    source_address: u16,
    target_address: u16,
}

impl DoipClient {
    pub fn new(source_address: u16) -> Result<Self> {
        let udp_socket = UdpSocket::bind("0.0.0.0:13400")?;
        udp_socket.set_broadcast(true)?;
        
        Ok(Self {
            tcp_stream: None,
            udp_socket,
            source_address,
            target_address: 0,
        })
    }
    
    /// Discover DoIP entities on the network
    pub fn discover_entities(&self) -> Result<Vec<DoipEntity>> {
        // Send vehicle identification request
        let header = DoipHeader::new(DoipPayloadType::VehicleIdentificationRequest, 0);
        self.udp_socket.send_to(&header.to_bytes(), "255.255.255.255:13400")?;
        
        // Collect responses
        let mut entities = Vec::new();
        let mut buf = [0u8; 1024];
        
        self.udp_socket.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
        
        while let Ok((len, addr)) = self.udp_socket.recv_from(&mut buf) {
            if let Ok(entity) = DoipEntity::from_announcement(&buf[..len], addr) {
                entities.push(entity);
            }
        }
        
        Ok(entities)
    }
    
    /// Connect to a DoIP entity
    pub fn connect(&mut self, entity: &DoipEntity) -> Result<()> {
        let stream = TcpStream::connect(entity.address)?;
        self.tcp_stream = Some(stream);
        self.target_address = entity.logical_address;
        
        // Send routing activation request
        self.send_routing_activation()?;
        
        Ok(())
    }
    
    /// Send a UDS diagnostic message
    pub fn send_diagnostic(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let stream = self.tcp_stream.as_mut()
            .ok_or(BusmasterError::Protocol { message: "Not connected".into() })?;
        
        // Build diagnostic message
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.source_address.to_be_bytes());
        payload.extend_from_slice(&self.target_address.to_be_bytes());
        payload.extend_from_slice(data);
        
        let header = DoipHeader::new(DoipPayloadType::DiagnosticMessage, payload.len() as u32);
        
        // Send
        use std::io::Write;
        stream.write_all(&header.to_bytes())?;
        stream.write_all(&payload)?;
        
        // Receive response
        self.receive_diagnostic_response()
    }
    
    fn send_routing_activation(&mut self) -> Result<()> {
        // Implementation
        Ok(())
    }
    
    fn receive_diagnostic_response(&mut self) -> Result<Vec<u8>> {
        // Implementation
        Ok(Vec::new())
    }
}

/// Discovered DoIP entity
#[derive(Debug, Clone)]
pub struct DoipEntity {
    pub address: SocketAddr,
    pub logical_address: u16,
    pub vin: String,
    pub eid: [u8; 6],
    pub gid: [u8; 6],
}

impl DoipEntity {
    fn from_announcement(data: &[u8], addr: SocketAddr) -> Result<Self> {
        // Parse vehicle announcement message
        // Implementation
        Ok(Self {
            address: addr,
            logical_address: 0,
            vin: String::new(),
            eid: [0; 6],
            gid: [0; 6],
        })
    }
}
```

### 3.2 SOME/IP Implementation

```rust
// crates/busmaster-proto/ethernet/src/someip.rs

use busmaster_core::{Result, BusmasterError};
use std::collections::HashMap;

/// SOME/IP message types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum SomeIpMessageType {
    Request = 0x00,
    RequestNoReturn = 0x01,
    Notification = 0x02,
    Response = 0x80,
    Error = 0x81,
    TpRequest = 0x20,
    TpRequestNoReturn = 0x21,
    TpNotification = 0x22,
    TpResponse = 0xA0,
    TpError = 0xA1,
}

/// SOME/IP header
#[derive(Debug, Clone)]
pub struct SomeIpHeader {
    pub service_id: u16,
    pub method_id: u16,
    pub length: u32,
    pub client_id: u16,
    pub session_id: u16,
    pub protocol_version: u8,
    pub interface_version: u8,
    pub message_type: SomeIpMessageType,
    pub return_code: u8,
}

impl SomeIpHeader {
    pub const SIZE: usize = 16;
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SIZE);
        bytes.extend_from_slice(&self.service_id.to_be_bytes());
        bytes.extend_from_slice(&self.method_id.to_be_bytes());
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.client_id.to_be_bytes());
        bytes.extend_from_slice(&self.session_id.to_be_bytes());
        bytes.push(self.protocol_version);
        bytes.push(self.interface_version);
        bytes.push(self.message_type as u8);
        bytes.push(self.return_code);
        bytes
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(BusmasterError::Protocol {
                message: "SOME/IP header too short".into(),
            });
        }
        
        Ok(Self {
            service_id: u16::from_be_bytes([data[0], data[1]]),
            method_id: u16::from_be_bytes([data[2], data[3]]),
            length: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            client_id: u16::from_be_bytes([data[8], data[9]]),
            session_id: u16::from_be_bytes([data[10], data[11]]),
            protocol_version: data[12],
            interface_version: data[13],
            message_type: SomeIpMessageType::try_from(data[14])?,
            return_code: data[15],
        })
    }
}

/// SOME/IP Service Discovery entry
#[derive(Debug, Clone)]
pub struct SdEntry {
    pub entry_type: SdEntryType,
    pub service_id: u16,
    pub instance_id: u16,
    pub major_version: u8,
    pub ttl: u32,
    pub minor_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdEntryType {
    FindService,
    OfferService,
    StopOfferService,
    SubscribeEventgroup,
    StopSubscribeEventgroup,
    SubscribeEventgroupAck,
    SubscribeEventgroupNack,
}

/// SOME/IP client for service communication
pub struct SomeIpClient {
    services: HashMap<(u16, u16), ServiceEndpoint>,
    subscriptions: HashMap<(u16, u16, u16), EventSubscription>,
    client_id: u16,
    session_counter: u16,
}

struct ServiceEndpoint {
    address: std::net::SocketAddr,
    protocol: TransportProtocol,
}

struct EventSubscription {
    callback: Box<dyn Fn(&[u8]) + Send>,
}

#[derive(Debug, Clone, Copy)]
pub enum TransportProtocol {
    Udp,
    Tcp,
}

impl SomeIpClient {
    pub fn new(client_id: u16) -> Self {
        Self {
            services: HashMap::new(),
            subscriptions: HashMap::new(),
            client_id,
            session_counter: 1,
        }
    }
    
    /// Find a service on the network
    pub fn find_service(&mut self, service_id: u16, instance_id: u16) -> Result<()> {
        // Send SD FindService entry
        Ok(())
    }
    
    /// Call a service method (request/response)
    pub fn call_method(
        &mut self,
        service_id: u16,
        instance_id: u16,
        method_id: u16,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        let endpoint = self.services.get(&(service_id, instance_id))
            .ok_or(BusmasterError::Protocol {
                message: format!("Service {:04X}:{:04X} not found", service_id, instance_id),
            })?;
        
        let header = SomeIpHeader {
            service_id,
            method_id,
            length: (payload.len() + 8) as u32,
            client_id: self.client_id,
            session_id: self.next_session_id(),
            protocol_version: 1,
            interface_version: 1,
            message_type: SomeIpMessageType::Request,
            return_code: 0,
        };
        
        // Send request and wait for response
        self.send_and_receive(&endpoint.address, &header, payload)
    }
    
    /// Subscribe to an event group
    pub fn subscribe_eventgroup<F>(
        &mut self,
        service_id: u16,
        instance_id: u16,
        eventgroup_id: u16,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(&[u8]) + Send + 'static,
    {
        self.subscriptions.insert(
            (service_id, instance_id, eventgroup_id),
            EventSubscription { callback: Box::new(callback) },
        );
        
        // Send SD SubscribeEventgroup entry
        Ok(())
    }
    
    fn next_session_id(&mut self) -> u16 {
        let id = self.session_counter;
        self.session_counter = self.session_counter.wrapping_add(1);
        if self.session_counter == 0 {
            self.session_counter = 1;
        }
        id
    }
    
    fn send_and_receive(
        &self,
        _addr: &std::net::SocketAddr,
        _header: &SomeIpHeader,
        _payload: &[u8],
    ) -> Result<Vec<u8>> {
        // Implementation
        Ok(Vec::new())
    }
}
```

---

## 4. Diagnostic Protocol Module Design (NEW)

### 4.1 UDS Implementation

```rust
// crates/busmaster-diag/uds/src/lib.rs

use busmaster_core::{Result, BusmasterError};

/// UDS Service Identifiers (SIDs)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum UdsService {
    DiagnosticSessionControl = 0x10,
    EcuReset = 0x11,
    SecurityAccess = 0x27,
    CommunicationControl = 0x28,
    TesterPresent = 0x3E,
    AccessTimingParameter = 0x83,
    SecuredDataTransmission = 0x84,
    ControlDtcSetting = 0x85,
    ResponseOnEvent = 0x86,
    LinkControl = 0x87,
    ReadDataByIdentifier = 0x22,
    ReadMemoryByAddress = 0x23,
    ReadScalingDataByIdentifier = 0x24,
    ReadDataByPeriodicIdentifier = 0x2A,
    DynamicallyDefineDataIdentifier = 0x2C,
    WriteDataByIdentifier = 0x2E,
    WriteMemoryByAddress = 0x3D,
    ClearDiagnosticInformation = 0x14,
    ReadDtcInformation = 0x19,
    InputOutputControlByIdentifier = 0x2F,
    RoutineControl = 0x31,
    RequestDownload = 0x34,
    RequestUpload = 0x35,
    TransferData = 0x36,
    RequestTransferExit = 0x37,
    RequestFileTransfer = 0x38,
}

/// UDS Negative Response Codes
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum UdsNrc {
    GeneralReject = 0x10,
    ServiceNotSupported = 0x11,
    SubFunctionNotSupported = 0x12,
    IncorrectMessageLengthOrInvalidFormat = 0x13,
    ResponseTooLong = 0x14,
    BusyRepeatRequest = 0x21,
    ConditionsNotCorrect = 0x22,
    RequestSequenceError = 0x24,
    NoResponseFromSubnetComponent = 0x25,
    FailurePreventsExecutionOfRequestedAction = 0x26,
    RequestOutOfRange = 0x31,
    SecurityAccessDenied = 0x33,
    InvalidKey = 0x35,
    ExceededNumberOfAttempts = 0x36,
    RequiredTimeDelayNotExpired = 0x37,
    UploadDownloadNotAccepted = 0x70,
    TransferDataSuspended = 0x71,
    GeneralProgrammingFailure = 0x72,
    WrongBlockSequenceCounter = 0x73,
    RequestCorrectlyReceivedResponsePending = 0x78,
    SubFunctionNotSupportedInActiveSession = 0x7E,
    ServiceNotSupportedInActiveSession = 0x7F,
}

/// UDS diagnostic session types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum DiagnosticSession {
    Default = 0x01,
    Programming = 0x02,
    Extended = 0x03,
    SafetySystem = 0x04,
}

/// UDS client for diagnostic communication
pub struct UdsClient<T: UdsTransport> {
    transport: T,
    current_session: DiagnosticSession,
    security_level: u8,
    p2_timeout_ms: u32,
    p2_star_timeout_ms: u32,
}

/// Transport layer trait for UDS
pub trait UdsTransport: Send {
    fn send(&mut self, data: &[u8]) -> Result<()>;
    fn receive(&mut self, timeout_ms: u32) -> Result<Vec<u8>>;
}

impl<T: UdsTransport> UdsClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            current_session: DiagnosticSession::Default,
            security_level: 0,
            p2_timeout_ms: 50,
            p2_star_timeout_ms: 5000,
        }
    }
    
    /// Change diagnostic session
    pub fn diagnostic_session_control(&mut self, session: DiagnosticSession) -> Result<()> {
        let request = vec![UdsService::DiagnosticSessionControl as u8, session as u8];
        let response = self.send_request(&request)?;
        
        if response.len() >= 2 && response[0] == 0x50 {
            self.current_session = session;
            // Update timing parameters from response
            if response.len() >= 6 {
                self.p2_timeout_ms = u16::from_be_bytes([response[2], response[3]]) as u32;
                self.p2_star_timeout_ms = u16::from_be_bytes([response[4], response[5]]) as u32 * 10;
            }
            Ok(())
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    /// Security access - request seed
    pub fn security_access_request_seed(&mut self, level: u8) -> Result<Vec<u8>> {
        let request = vec![UdsService::SecurityAccess as u8, level];
        let response = self.send_request(&request)?;
        
        if response.len() >= 2 && response[0] == 0x67 {
            Ok(response[2..].to_vec())
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    /// Security access - send key
    pub fn security_access_send_key(&mut self, level: u8, key: &[u8]) -> Result<()> {
        let mut request = vec![UdsService::SecurityAccess as u8, level + 1];
        request.extend_from_slice(key);
        let response = self.send_request(&request)?;
        
        if response.len() >= 2 && response[0] == 0x67 {
            self.security_level = level;
            Ok(())
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    /// Read data by identifier
    pub fn read_data_by_identifier(&mut self, did: u16) -> Result<Vec<u8>> {
        let request = vec![
            UdsService::ReadDataByIdentifier as u8,
            (did >> 8) as u8,
            did as u8,
        ];
        let response = self.send_request(&request)?;
        
        if response.len() >= 3 && response[0] == 0x62 {
            Ok(response[3..].to_vec())
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    /// Write data by identifier
    pub fn write_data_by_identifier(&mut self, did: u16, data: &[u8]) -> Result<()> {
        let mut request = vec![
            UdsService::WriteDataByIdentifier as u8,
            (did >> 8) as u8,
            did as u8,
        ];
        request.extend_from_slice(data);
        let response = self.send_request(&request)?;
        
        if response.len() >= 3 && response[0] == 0x6E {
            Ok(())
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    /// Read DTC information
    pub fn read_dtc_information(&mut self, sub_function: u8) -> Result<Vec<DtcInfo>> {
        let request = vec![UdsService::ReadDtcInformation as u8, sub_function];
        let response = self.send_request(&request)?;
        
        if response.len() >= 2 && response[0] == 0x59 {
            self.parse_dtc_response(&response[2..])
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    /// Clear diagnostic information
    pub fn clear_dtc(&mut self, group: u32) -> Result<()> {
        let request = vec![
            UdsService::ClearDiagnosticInformation as u8,
            ((group >> 16) & 0xFF) as u8,
            ((group >> 8) & 0xFF) as u8,
            (group & 0xFF) as u8,
        ];
        let response = self.send_request(&request)?;
        
        if response.len() >= 1 && response[0] == 0x54 {
            Ok(())
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    /// ECU reset
    pub fn ecu_reset(&mut self, reset_type: u8) -> Result<()> {
        let request = vec![UdsService::EcuReset as u8, reset_type];
        let response = self.send_request(&request)?;
        
        if response.len() >= 2 && response[0] == 0x51 {
            Ok(())
        } else {
            Err(self.parse_negative_response(&response))
        }
    }
    
    fn send_request(&mut self, request: &[u8]) -> Result<Vec<u8>> {
        self.transport.send(request)?;
        
        loop {
            let response = self.transport.receive(self.p2_timeout_ms)?;
            
            // Check for response pending
            if response.len() >= 3 && response[0] == 0x7F && response[2] == 0x78 {
                // Wait longer and retry
                continue;
            }
            
            return Ok(response);
        }
    }
    
    fn parse_negative_response(&self, response: &[u8]) -> BusmasterError {
        if response.len() >= 3 && response[0] == 0x7F {
            let nrc = response[2];
            BusmasterError::Protocol {
                message: format!("UDS negative response: NRC 0x{:02X}", nrc),
            }
        } else {
            BusmasterError::Protocol {
                message: "Invalid UDS response".into(),
            }
        }
    }
    
    fn parse_dtc_response(&self, _data: &[u8]) -> Result<Vec<DtcInfo>> {
        // Implementation
        Ok(Vec::new())
    }
}

/// DTC information
#[derive(Debug, Clone)]
pub struct DtcInfo {
    pub dtc: u32,
    pub status: u8,
}
```

---

## 5. Cloud-Native Architecture (NEW)

### 5.1 REST API Design

```rust
// crates/busmaster-api/src/lib.rs

use axum::{
    routing::{get, post, delete},
    Router, Json, Extension,
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// API state shared across handlers
pub struct ApiState {
    pub engine: Arc<RwLock<busmaster_engine::Engine>>,
    pub sessions: Arc<RwLock<SessionManager>>,
}

/// Create the API router
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // Hardware management
        .route("/api/v1/drivers", get(list_drivers))
        .route("/api/v1/drivers/:id/connect", post(connect_driver))
        .route("/api/v1/drivers/:id/disconnect", post(disconnect_driver))
        
        // Channel management
        .route("/api/v1/channels", get(list_channels))
        .route("/api/v1/channels/:id/config", post(configure_channel))
        .route("/api/v1/channels/:id/start", post(start_channel))
        .route("/api/v1/channels/:id/stop", post(stop_channel))
        
        // Message operations
        .route("/api/v1/messages", get(get_messages))
        .route("/api/v1/messages/send", post(send_message))
        .route("/api/v1/messages/filter", post(set_filter))
        
        // Database operations
        .route("/api/v1/databases", get(list_databases))
        .route("/api/v1/databases", post(load_database))
        .route("/api/v1/databases/:id", delete(unload_database))
        .route("/api/v1/databases/:id/signals", get(get_signals))
        
        // Logging operations
        .route("/api/v1/logging/start", post(start_logging))
        .route("/api/v1/logging/stop", post(stop_logging))
        .route("/api/v1/logging/status", get(logging_status))
        
        // Diagnostics
        .route("/api/v1/diagnostics/uds", post(send_uds_request))
        .route("/api/v1/diagnostics/dtc", get(read_dtc))
        .route("/api/v1/diagnostics/dtc", delete(clear_dtc))
        
        // WebSocket for real-time streaming
        .route("/api/v1/ws/messages", get(websocket_messages))
        .route("/api/v1/ws/signals", get(websocket_signals))
        
        // AI analysis
        .route("/api/v1/ai/analyze", post(ai_analyze))
        .route("/api/v1/ai/query", post(ai_query))
        
        .with_state(Arc::new(state))
}

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct DriverInfo {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub connected: bool,
    pub channels: Vec<ChannelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: u8,
    pub baudrate: u32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub channel: u8,
    pub id: u32,
    pub extended: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterConfig {
    pub mode: String, // "pass" or "block"
    pub ids: Vec<u32>,
    pub id_mask: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UdsRequest {
    pub service: u8,
    pub sub_function: Option<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAnalysisRequest {
    pub query: String,
    pub context: Option<String>,
    pub include_recent_messages: bool,
}

// Handler implementations (stubs)
async fn list_drivers(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let drivers = engine.list_drivers();
    Json(drivers)
}

async fn connect_driver(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut engine = state.engine.write().await;
    match engine.connect_driver(&id) {
        Ok(_) => Json(serde_json::json!({"status": "connected"})),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn websocket_messages(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ApiState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_message_websocket(socket, state))
}

async fn handle_message_websocket(
    mut socket: axum::extract::ws::WebSocket,
    state: Arc<ApiState>,
) {
    use axum::extract::ws::Message;
    
    // Subscribe to message stream
    let mut rx = {
        let engine = state.engine.read().await;
        engine.subscribe_messages()
    };
    
    while let Some(msg) = rx.recv().await {
        let json = serde_json::to_string(&msg).unwrap();
        if socket.send(Message::Text(json)).await.is_err() {
            break;
        }
    }
}

// Additional handler stubs
async fn disconnect_driver(State(_state): State<Arc<ApiState>>, Path(_id): Path<String>) -> impl IntoResponse { Json(()) }
async fn list_channels(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn configure_channel(State(_state): State<Arc<ApiState>>, Path(_id): Path<u8>) -> impl IntoResponse { Json(()) }
async fn start_channel(State(_state): State<Arc<ApiState>>, Path(_id): Path<u8>) -> impl IntoResponse { Json(()) }
async fn stop_channel(State(_state): State<Arc<ApiState>>, Path(_id): Path<u8>) -> impl IntoResponse { Json(()) }
async fn get_messages(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn send_message(State(_state): State<Arc<ApiState>>, Json(_req): Json<SendMessageRequest>) -> impl IntoResponse { Json(()) }
async fn set_filter(State(_state): State<Arc<ApiState>>, Json(_req): Json<FilterConfig>) -> impl IntoResponse { Json(()) }
async fn list_databases(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn load_database(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn unload_database(State(_state): State<Arc<ApiState>>, Path(_id): Path<String>) -> impl IntoResponse { Json(()) }
async fn get_signals(State(_state): State<Arc<ApiState>>, Path(_id): Path<String>) -> impl IntoResponse { Json(()) }
async fn start_logging(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn stop_logging(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn logging_status(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn send_uds_request(State(_state): State<Arc<ApiState>>, Json(_req): Json<UdsRequest>) -> impl IntoResponse { Json(()) }
async fn read_dtc(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn clear_dtc(State(_state): State<Arc<ApiState>>) -> impl IntoResponse { Json(()) }
async fn websocket_signals(ws: WebSocketUpgrade, State(_state): State<Arc<ApiState>>) -> impl IntoResponse { ws.on_upgrade(|_| async {}) }
async fn ai_analyze(State(_state): State<Arc<ApiState>>, Json(_req): Json<AiAnalysisRequest>) -> impl IntoResponse { Json(()) }
async fn ai_query(State(_state): State<Arc<ApiState>>, Json(_req): Json<AiAnalysisRequest>) -> impl IntoResponse { Json(()) }

pub struct SessionManager;
```

---

## 6. AI Integration Design (NEW)

### 6.1 AI Assistant Module

```rust
// crates/busmaster-ai/src/lib.rs

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use busmaster_core::{CanFrame, Result, BusmasterError};
use serde::{Deserialize, Serialize};

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiProvider {
    OpenAI,
    AzureOpenAI { endpoint: String, deployment: String },
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::OpenAI,
            api_key: String::new(),
            model: "gpt-4".into(),
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: None,
        }
    }
}

/// AI assistant for automotive analysis
pub struct AiAssistant {
    config: AiConfig,
    client: Client<OpenAIConfig>,
    conversation_history: Vec<ChatCompletionRequestMessage>,
}

impl AiAssistant {
    pub fn new(config: AiConfig) -> Result<Self> {
        let openai_config = match &config.provider {
            AiProvider::OpenAI => {
                OpenAIConfig::new().with_api_key(&config.api_key)
            }
            AiProvider::AzureOpenAI { endpoint, .. } => {
                OpenAIConfig::new()
                    .with_api_key(&config.api_key)
                    .with_api_base(endpoint)
            }
        };
        
        let client = Client::with_config(openai_config);
        
        let system_prompt = config.system_prompt.clone().unwrap_or_else(|| {
            AUTOMOTIVE_SYSTEM_PROMPT.to_string()
        });
        
        let conversation_history = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .unwrap()
                .into(),
        ];
        
        Ok(Self {
            config,
            client,
            conversation_history,
        })
    }
    
    /// Analyze CAN traffic for anomalies
    pub async fn analyze_traffic(&mut self, frames: &[CanFrame]) -> Result<TrafficAnalysis> {
        let traffic_summary = self.summarize_traffic(frames);
        
        let prompt = format!(
            "Analyze this CAN bus traffic for anomalies, patterns, or issues:\n\n{}",
            traffic_summary
        );
        
        let response = self.chat(&prompt).await?;
        
        Ok(TrafficAnalysis {
            summary: response.clone(),
            anomalies: self.extract_anomalies(&response),
            recommendations: self.extract_recommendations(&response),
        })
    }
    
    /// Natural language query about the data
    pub async fn query(&mut self, question: &str, context: Option<&str>) -> Result<String> {
        let prompt = if let Some(ctx) = context {
            format!("Context:\n{}\n\nQuestion: {}", ctx, question)
        } else {
            question.to_string()
        };
        
        self.chat(&prompt).await
    }
    
    /// Suggest DBC signal interpretations
    pub async fn suggest_signals(&mut self, frame_id: u32, data_samples: &[Vec<u8>]) -> Result<Vec<SignalSuggestion>> {
        let samples_str = data_samples.iter()
            .take(10)
            .map(|d| format!("{:02X?}", d))
            .collect::<Vec<_>>()
            .join("\n");
        
        let prompt = format!(
            "Analyze these CAN frame data samples for ID 0x{:X} and suggest possible signal definitions:\n\n{}",
            frame_id, samples_str
        );
        
        let response = self.chat(&prompt).await?;
        self.parse_signal_suggestions(&response)
    }
    
    /// Generate test case recommendations
    pub async fn recommend_tests(&mut self, dbc_summary: &str) -> Result<Vec<TestRecommendation>> {
        let prompt = format!(
            "Based on this DBC database summary, recommend test cases for validation:\n\n{}",
            dbc_summary
        );
        
        let response = self.chat(&prompt).await?;
        self.parse_test_recommendations(&response)
    }
    
    /// Diagnose errors and suggest fixes
    pub async fn diagnose_error(&mut self, error_context: &str) -> Result<DiagnosisResult> {
        let prompt = format!(
            "Diagnose this automotive communication error and suggest fixes:\n\n{}",
            error_context
        );
        
        let response = self.chat(&prompt).await?;
        
        Ok(DiagnosisResult {
            diagnosis: response.clone(),
            possible_causes: self.extract_causes(&response),
            suggested_fixes: self.extract_fixes(&response),
        })
    }
    
    /// Core chat function
    async fn chat(&mut self, message: &str) -> Result<String> {
        self.conversation_history.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(message)
                .build()
                .unwrap()
                .into(),
        );
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .messages(self.conversation_history.clone())
            .temperature(self.config.temperature)
            .max_tokens(self.config.max_tokens as u16)
            .build()
            .map_err(|e| BusmasterError::Config {
                message: format!("Failed to build AI request: {}", e),
            })?;
        
        let response = self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| BusmasterError::Config {
                message: format!("AI API error: {}", e),
            })?;
        
        let content = response.choices.first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        
        Ok(content)
    }
    
    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.conversation_history.truncate(1); // Keep system prompt
    }
    
    fn summarize_traffic(&self, frames: &[CanFrame]) -> String {
        use std::collections::HashMap;
        
        let mut id_counts: HashMap<u32, usize> = HashMap::new();
        let mut id_data: HashMap<u32, Vec<&[u8]>> = HashMap::new();
        
        for frame in frames {
            *id_counts.entry(frame.id).or_insert(0) += 1;
            id_data.entry(frame.id).or_default().push(&frame.data);
        }
        
        let mut summary = format!("Total frames: {}\n\n", frames.len());
        
        for (id, count) in id_counts.iter() {
            summary.push_str(&format!("ID 0x{:X}: {} messages\n", id, count));
            if let Some(samples) = id_data.get(id) {
                if let Some(first) = samples.first() {
                    summary.push_str(&format!("  Sample data: {:02X?}\n", first));
                }
            }
        }
        
        summary
    }
    
    fn extract_anomalies(&self, _response: &str) -> Vec<String> { Vec::new() }
    fn extract_recommendations(&self, _response: &str) -> Vec<String> { Vec::new() }
    fn parse_signal_suggestions(&self, _response: &str) -> Result<Vec<SignalSuggestion>> { Ok(Vec::new()) }
    fn parse_test_recommendations(&self, _response: &str) -> Result<Vec<TestRecommendation>> { Ok(Vec::new()) }
    fn extract_causes(&self, _response: &str) -> Vec<String> { Vec::new() }
    fn extract_fixes(&self, _response: &str) -> Vec<String> { Vec::new() }
}

/// System prompt for automotive expertise
const AUTOMOTIVE_SYSTEM_PROMPT: &str = r#"
You are an expert automotive engineer specializing in:
- CAN, LIN, FlexRay, and Automotive Ethernet protocols
- ECU diagnostics (UDS, OBD-II, KWP2000)
- Signal analysis and DBC database interpretation
- J1939 for commercial vehicles
- DoIP and SOME/IP for modern vehicles

When analyzing data:
1. Look for timing anomalies and bus errors
2. Identify potential signal patterns
3. Suggest diagnostic approaches
4. Recommend test cases

Be concise and technical. Use automotive terminology appropriately.
"#;

#[derive(Debug, Clone, Serialize)]
pub struct TrafficAnalysis {
    pub summary: String,
    pub anomalies: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalSuggestion {
    pub name: String,
    pub start_bit: u32,
    pub length: u32,
    pub factor: f64,
    pub offset: f64,
    pub unit: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestRecommendation {
    pub name: String,
    pub description: String,
    pub priority: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosisResult {
    pub diagnosis: String,
    pub possible_causes: Vec<String>,
    pub suggested_fixes: Vec<String>,
}
```

---

## 7. Core Module Design (MVP - From Original)

### 7.1 CAN Frame Types

```rust
// crates/busmaster-core/src/can.rs
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Standard CAN frame (CAN 2.0A/B)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanFrame {
    pub id: u32,
    pub is_extended: bool,
    pub is_rtr: bool,
    pub dlc: u8,
    pub data: Vec<u8>,
    pub timestamp_us: u64,
    pub channel: u8,
    pub is_tx: bool,
}

impl CanFrame {
    pub fn new_standard(id: u16, data: &[u8]) -> Self {
        assert!(id <= 0x7FF, "Standard ID must be 11 bits");
        assert!(data.len() <= 8, "CAN 2.0 data max 8 bytes");
        Self {
            id: id as u32,
            is_extended: false,
            is_rtr: false,
            dlc: data.len() as u8,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
            is_tx: false,
        }
    }
    
    pub fn new_extended(id: u32, data: &[u8]) -> Self {
        assert!(id <= 0x1FFFFFFF, "Extended ID must be 29 bits");
        assert!(data.len() <= 8, "CAN 2.0 data max 8 bytes");
        Self {
            id,
            is_extended: true,
            is_rtr: false,
            dlc: data.len() as u8,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
            is_tx: false,
        }
    }
}

/// CAN FD frame (extended data length)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanFdFrame {
    pub id: u32,
    pub is_extended: bool,
    pub is_brs: bool,  // Bit Rate Switch
    pub is_esi: bool,  // Error State Indicator
    pub dlc: u8,
    pub data: Vec<u8>,
    pub timestamp_us: u64,
    pub channel: u8,
    pub is_tx: bool,
}

impl CanFdFrame {
    /// DLC to actual data length mapping for CAN FD
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
}
```

### 7.2 Error Types

```rust
// crates/busmaster-core/src/error.rs
#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BusmasterError {
    #[error("Hardware error: {message}")]
    Hardware { message: String, vendor: String },
    
    #[error("Protocol error: {message}")]
    Protocol { message: String },
    
    #[error("Database parse error at line {line}: {message}")]
    DatabaseParse { message: String, line: usize },
    
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Channel {channel} not found")]
    ChannelNotFound { channel: u8 },
    
    #[error("Timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u32 },
    
    #[error("Buffer full (capacity: {capacity})")]
    BufferFull { capacity: usize },
    
    #[error("AI error: {message}")]
    Ai { message: String },
    
    #[error("Network error: {message}")]
    Network { message: String },
}

pub type Result<T> = std::result::Result<T, BusmasterError>;
```

---

## 8. Agent Role Responsibilities (Design Perspective)

### 8.1 Architect Agent Design Outputs

The Architect Agent is responsible for:

1. **Crate Dependency Graph**
```
busmaster-cli
    └── busmaster-engine
        ├── busmaster-proto
        │   └── busmaster-core
        ├── busmaster-db
        │   └── busmaster-core
        ├── busmaster-log
        │   └── busmaster-core
        ├── busmaster-dil
        │   └── busmaster-core
        └── busmaster-hardware
            ├── busmaster-dil
            └── busmaster-platform
```

2. **API Design Principles**
- All public APIs must be documented
- Use builder pattern for complex configurations
- Prefer `&str` over `String` in function parameters
- Return `Result<T>` for fallible operations
- Use `#[must_use]` for important return values

3. **Error Handling Strategy**
- Use `thiserror` for library errors
- Use `anyhow` for application errors
- Provide context with `.context()` or custom error types
- Never panic in library code

### 8.2 Protocol Agent Design Outputs

For each protocol, the Protocol Agent produces:

1. **Frame Types** - Rust structs with serialization
2. **State Machines** - For protocol handling
3. **Encoding/Decoding** - Byte-level operations
4. **Validation** - Input validation functions
5. **Tests** - Unit and property-based tests

### 8.3 Hardware Agent Design Outputs

For each hardware vendor:

1. **FFI Bindings** - Safe wrappers around C APIs
2. **Driver Implementation** - `CanDriver` trait impl
3. **Platform Variants** - macOS/Windows/Linux versions
4. **Error Mapping** - Vendor errors to `BusmasterError`
5. **Documentation** - Setup and usage guides

---

## 9. Testing Strategy

### 9.1 Test Categories

| Category | Coverage Target | Tools |
|----------|-----------------|-------|
| Unit tests | 90% | `cargo test` |
| Integration tests | 70% | `cargo test --test '*'` |
| Property tests | Key algorithms | `proptest` |
| Benchmarks | Performance-critical | `criterion` |
| Fuzz tests | Parsers | `cargo-fuzz` |
| API tests | REST endpoints | `reqwest` + `tokio-test` |

### 9.2 Property-Based Testing Examples

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn can_frame_roundtrip(
        id in 0u32..0x800,
        data in prop::collection::vec(any::<u8>(), 0..=8)
    ) {
        let frame = CanFrame::new_standard(id as u16, &data);
        let bytes = frame.to_bytes();
        let decoded = CanFrame::from_bytes(&bytes).unwrap();
        prop_assert_eq!(frame.id, decoded.id);
        prop_assert_eq!(frame.data, decoded.data);
    }
}
```

---

## 10. Design Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Async runtime | tokio (Phase 2+) | Industry standard, required for API |
| HTTP framework | axum | Modern, tower-based, good ergonomics |
| AI client | async-openai | Official-ish, well-maintained |
| UI framework | CLI/TUI first, egui later | Faster iteration |
| Error handling | thiserror | Ergonomic, zero-cost |
| Serialization | serde | Industry standard |
| Logging | tracing | Structured, async-compatible |
| Testing | proptest + criterion | Comprehensive coverage |
| Platform abstraction | Traits + cfg | Compile-time selection |

---

## 11. Known Limitations (MVP)

1. **CAN only** - No LIN, FlexRay, Ethernet, J1939
2. **macOS only** - No Linux/Windows
3. **PEAK + Stub only** - No Vector, Kvaser, ETAS
4. **CLI/TUI only** - No GUI
5. **ASC only** - No BLF logging
6. **DBC only** - No DBF, ARXML, ODX
7. **No plugins** - Deferred to Phase 4
8. **No simulation** - ECU simulation deferred
9. **No AI** - Deferred to Phase 3
10. **No REST API** - Deferred to Phase 3

---

**Document Status:** APPROVED FOR AI DEVELOPMENT  
**Project Owner:** JC  
**Last Updated:** January 2026  
**Version:** 4.0

---

## Appendix: New Module Designs (v4.0 Additions)

### A.1 CAN XL Module Design

```rust
// crates/busmaster-proto/can/src/canxl.rs

use serde::{Deserialize, Serialize};

/// CAN XL frame (up to 2048 bytes payload)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanXlFrame {
    pub priority: u8,           // 3-bit priority
    pub vcid: u8,               // Virtual CAN Network ID
    pub acceptance_field: u32,  // 32-bit acceptance field
    pub sdt: u8,                // Service Data Unit Type
    pub sec: bool,              // Simple Extended Content
    pub dlc: u16,               // Data Length (0-2048)
    pub data: Vec<u8>,
    pub timestamp_us: u64,
    pub channel: u8,
    pub is_tx: bool,
}

impl CanXlFrame {
    pub const MAX_DATA_LEN: usize = 2048;
    
    pub fn new(acceptance_field: u32, data: &[u8]) -> Self {
        assert!(data.len() <= Self::MAX_DATA_LEN);
        Self {
            priority: 0,
            vcid: 0,
            acceptance_field,
            sdt: 0,
            sec: false,
            dlc: data.len() as u16,
            data: data.to_vec(),
            timestamp_us: 0,
            channel: 0,
            is_tx: false,
        }
    }
}
```

### A.2 SecOC Security Module Design

```rust
// crates/busmaster-proto/secoc/src/lib.rs

use aes::Aes128;
use cmac::{Cmac, Mac};
use serde::{Deserialize, Serialize};

/// SecOC PDU structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecOcPdu {
    pub authentic_pdu: Vec<u8>,
    pub freshness_value: u64,
    pub mac: Vec<u8>,
}

/// SecOC configuration
#[derive(Debug, Clone)]
pub struct SecOcConfig {
    pub key: [u8; 16],
    pub freshness_length: u8,
    pub mac_length: u8,
}

/// SecOC processor for message authentication
pub struct SecOcProcessor {
    config: SecOcConfig,
    freshness_counter: u64,
}

impl SecOcProcessor {
    pub fn new(config: SecOcConfig) -> Self {
        Self {
            config,
            freshness_counter: 0,
        }
    }
    
    /// Generate MAC for outgoing message
    pub fn authenticate(&mut self, data: &[u8]) -> SecOcPdu {
        self.freshness_counter += 1;
        let mac = self.compute_mac(data, self.freshness_counter);
        
        SecOcPdu {
            authentic_pdu: data.to_vec(),
            freshness_value: self.freshness_counter,
            mac,
        }
    }
    
    /// Verify MAC for incoming message
    pub fn verify(&self, pdu: &SecOcPdu) -> bool {
        let expected_mac = self.compute_mac(&pdu.authentic_pdu, pdu.freshness_value);
        pdu.mac == expected_mac
    }
    
    fn compute_mac(&self, data: &[u8], freshness: u64) -> Vec<u8> {
        let mut mac = Cmac::<Aes128>::new_from_slice(&self.config.key).unwrap();
        mac.update(data);
        mac.update(&freshness.to_be_bytes());
        mac.finalize().into_bytes()[..self.config.mac_length as usize].to_vec()
    }
}
```

### A.3 Test Automation Framework Design

```rust
// crates/busmaster-test-automation/src/lib.rs

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub preconditions: Vec<String>,
    pub steps: Vec<TestStep>,
    pub expected_results: Vec<ExpectedResult>,
    pub timeout: Duration,
}

/// Test step types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStep {
    SendMessage { channel: u8, id: u32, data: Vec<u8> },
    WaitForMessage { channel: u8, id: u32, timeout_ms: u32 },
    WaitDuration { ms: u32 },
    SetSignal { message: String, signal: String, value: f64 },
    CheckSignal { message: String, signal: String, expected: f64, tolerance: f64 },
    SendUdsRequest { service: u8, data: Vec<u8> },
    Script { lua_code: String },
}

/// Expected result definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedResult {
    pub description: String,
    pub condition: ResultCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultCondition {
    MessageReceived { id: u32, within_ms: u32 },
    SignalValue { signal: String, min: f64, max: f64 },
    NoError,
    UdsPositiveResponse { service: u8 },
}

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub step_results: Vec<StepResult>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed { reason: String },
    Skipped { reason: String },
    Error { message: String },
}

/// Test runner for automated execution
pub struct TestRunner {
    engine: std::sync::Arc<busmaster_engine::Engine>,
    results: Vec<TestResult>,
}

impl TestRunner {
    pub async fn run_test(&mut self, test: &TestCase) -> TestResult {
        // Implementation
        todo!()
    }
    
    pub fn generate_report(&self, format: ReportFormat) -> String {
        // Implementation
        todo!()
    }
}

pub enum ReportFormat {
    Html,
    Xml,
    Json,
}
```

### A.4 Gateway Simulation Design

```rust
// crates/busmaster-gateway/src/lib.rs

use std::collections::HashMap;
use busmaster_core::{CanFrame, Result};

/// Routing rule for gateway
#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub source_channel: u8,
    pub source_id_filter: IdFilter,
    pub destination_channel: u8,
    pub transformation: Option<Transformation>,
}

#[derive(Debug, Clone)]
pub enum IdFilter {
    Single(u32),
    Range { start: u32, end: u32 },
    Mask { id: u32, mask: u32 },
    All,
}

#[derive(Debug, Clone)]
pub enum Transformation {
    IdRemap { new_id: u32 },
    DataModify { offset: usize, value: u8 },
    ProtocolConvert { from: Protocol, to: Protocol },
}

#[derive(Debug, Clone)]
pub enum Protocol {
    Can,
    CanFd,
    CanXl,
    Ethernet,
}

/// Gateway simulator
pub struct GatewaySimulator {
    rules: Vec<RoutingRule>,
    statistics: GatewayStats,
}

#[derive(Debug, Default)]
pub struct GatewayStats {
    pub messages_routed: u64,
    pub messages_dropped: u64,
    pub avg_latency_us: f64,
}

impl GatewaySimulator {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            statistics: GatewayStats::default(),
        }
    }
    
    pub fn add_rule(&mut self, rule: RoutingRule) {
        self.rules.push(rule);
    }
    
    pub fn route_message(&mut self, frame: &CanFrame) -> Vec<(u8, CanFrame)> {
        let mut outputs = Vec::new();
        
        for rule in &self.rules {
            if rule.source_channel == frame.channel && self.matches_filter(&rule.source_id_filter, frame.id) {
                let mut routed_frame = frame.clone();
                routed_frame.channel = rule.destination_channel;
                
                if let Some(transform) = &rule.transformation {
                    self.apply_transformation(&mut routed_frame, transform);
                }
                
                outputs.push((rule.destination_channel, routed_frame));
                self.statistics.messages_routed += 1;
            }
        }
        
        outputs
    }
    
    fn matches_filter(&self, filter: &IdFilter, id: u32) -> bool {
        match filter {
            IdFilter::Single(f) => *f == id,
            IdFilter::Range { start, end } => id >= *start && id <= *end,
            IdFilter::Mask { id: f, mask } => (id & mask) == (*f & mask),
            IdFilter::All => true,
        }
    }
    
    fn apply_transformation(&self, frame: &mut CanFrame, transform: &Transformation) {
        match transform {
            Transformation::IdRemap { new_id } => frame.id = *new_id,
            Transformation::DataModify { offset, value } => {
                if *offset < frame.data.len() {
                    frame.data[*offset] = *value;
                }
            }
            Transformation::ProtocolConvert { .. } => {
                // Protocol conversion logic
            }
        }
    }
}
```

### A.5 Reverse Engineering Module Design

```rust
// crates/busmaster-reverse/src/lib.rs

use std::collections::HashMap;
use busmaster_core::CanFrame;

/// Discovered signal candidate
#[derive(Debug, Clone)]
pub struct SignalCandidate {
    pub start_bit: u32,
    pub length: u32,
    pub is_signed: bool,
    pub byte_order: ByteOrder,
    pub min_value: f64,
    pub max_value: f64,
    pub confidence: f32,
    pub suggested_name: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

/// Signal discovery engine
pub struct SignalDiscovery {
    frame_history: HashMap<u32, Vec<Vec<u8>>>,
    candidates: HashMap<u32, Vec<SignalCandidate>>,
}

impl SignalDiscovery {
    pub fn new() -> Self {
        Self {
            frame_history: HashMap::new(),
            candidates: HashMap::new(),
        }
    }
    
    /// Add frame to analysis
    pub fn add_frame(&mut self, frame: &CanFrame) {
        self.frame_history
            .entry(frame.id)
            .or_default()
            .push(frame.data.clone());
    }
    
    /// Analyze collected frames for signal patterns
    pub fn analyze(&mut self, frame_id: u32) -> Vec<SignalCandidate> {
        let samples = match self.frame_history.get(&frame_id) {
            Some(s) if s.len() >= 10 => s,
            _ => return Vec::new(),
        };
        
        let mut candidates = Vec::new();
        
        // Detect byte boundaries with changing values
        for byte_idx in 0..8 {
            if let Some(candidate) = self.analyze_byte(samples, byte_idx) {
                candidates.push(candidate);
            }
        }
        
        // Detect multi-byte signals
        for start_byte in 0..7 {
            for length in 2..=4 {
                if start_byte + length <= 8 {
                    if let Some(candidate) = self.analyze_multi_byte(samples, start_byte, length) {
                        candidates.push(candidate);
                    }
                }
            }
        }
        
        self.candidates.insert(frame_id, candidates.clone());
        candidates
    }
    
    /// Generate DBC from discovered signals
    pub fn generate_dbc(&self) -> String {
        let mut dbc = String::from("VERSION \"\"\n\nNS_ :\n\nBS_:\n\nBU_:\n\n");
        
        for (id, candidates) in &self.candidates {
            dbc.push_str(&format!("BO_ {} Message_{:X}: 8 Vector__XXX\n", id, id));
            
            for (idx, candidate) in candidates.iter().enumerate() {
                dbc.push_str(&format!(
                    " SG_ Signal_{}_{} : {}|{}@{}+ (1,0) [{}|{}] \"\" Vector__XXX\n",
                    id, idx,
                    candidate.start_bit,
                    candidate.length,
                    if matches!(candidate.byte_order, ByteOrder::LittleEndian) { "1" } else { "0" },
                    candidate.min_value,
                    candidate.max_value
                ));
            }
            dbc.push('\n');
        }
        
        dbc
    }
    
    fn analyze_byte(&self, samples: &[Vec<u8>], byte_idx: usize) -> Option<SignalCandidate> {
        // Analyze single byte for signal patterns
        let values: Vec<u8> = samples.iter()
            .filter_map(|s| s.get(byte_idx).copied())
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        let min = *values.iter().min()? as f64;
        let max = *values.iter().max()? as f64;
        let variance = self.calculate_variance(&values);
        
        // Only consider as signal if there's meaningful variation
        if variance > 0.1 && max > min {
            Some(SignalCandidate {
                start_bit: (byte_idx * 8) as u32,
                length: 8,
                is_signed: false,
                byte_order: ByteOrder::LittleEndian,
                min_value: min,
                max_value: max,
                confidence: (variance / 100.0).min(1.0) as f32,
                suggested_name: format!("Signal_B{}", byte_idx),
            })
        } else {
            None
        }
    }
    
    fn analyze_multi_byte(&self, samples: &[Vec<u8>], start: usize, len: usize) -> Option<SignalCandidate> {
        // Analyze multi-byte signal patterns
        // Implementation would detect 16-bit, 32-bit signals
        None
    }
    
    fn calculate_variance(&self, values: &[u8]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mean = values.iter().map(|&v| v as f64).sum::<f64>() / values.len() as f64;
        values.iter().map(|&v| (v as f64 - mean).powi(2)).sum::<f64>() / values.len() as f64
    }
}
```
