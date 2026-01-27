//! BUSMASTER Protocol Implementations
//!
//! This crate provides protocol parsing and encoding for various
//! automotive bus protocols including CAN, CAN FD, LIN, J1939, UDS, OBD-II, DoIP, SOME/IP, and XCP.
//!
//! # Supported Protocols
//!
//! ## CAN (MVP)
//! - CAN 2.0A (11-bit standard ID)
//! - CAN 2.0B (29-bit extended ID)
//! - CAN FD (up to 64 bytes)
//!
//! ## J1939 (Phase 2)
//! - PGN parsing and encoding
//! - Transport Protocol (BAM, CMDT)
//! - ECU Name handling
//! - Address claiming support
//!
//! ## UDS (Phase 2)
//! - Diagnostic session control
//! - Security access
//! - Read/write data by identifier
//! - DTC management
//! - Routine control
//! - Transfer data
//!
//! ## OBD-II (Phase 2)
//! - Standard PIDs
//! - DTC reading
//! - VIN reading
//!
//! ## DoIP (Phase 2)
//! - Vehicle identification
//! - Routing activation
//! - Diagnostic message tunneling (UDS over DoIP)
//! - Alive check mechanism
//!
//! ## SOME/IP (Phase 2)
//! - Request/Response pattern
//! - Fire & Forget (notification)
//! - Service Discovery (SOME/IP-SD)
//! - Transport Protocol (SOME/IP-TP)
//!
//! ## LIN (Phase 3)
//! - LIN frame encoding/decoding
//! - Protected ID (PID) calculation
//! - Classic and Enhanced checksums
//! - Schedule table support
//! - Node configuration
//!
//! ## XCP (Phase 3)
//! - Command/Response communication
//! - DAQ (Data Acquisition) for measurement
//! - STIM (Stimulation) for bypassing
//! - Memory read/write operations
//! - Calibration support
//!
//! ## Future Protocols
//! - FlexRay (Phase 5)
//!
//! # Example
//!
//! ```
//! use busmaster_core::CanFrame;
//! use busmaster_proto::{CanParser, CanEncoder};
//!
//! // Create and encode a frame
//! let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03]).unwrap();
//! let bytes = CanEncoder::encode(&frame);
//!
//! // Parse it back
//! let parsed = CanParser::parse(&bytes).unwrap();
//! assert_eq!(frame.id(), parsed.id());
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::redundant_else)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::inherent_to_string)]

pub mod calibration;
pub mod can;
pub mod doip;
pub mod j1939;
pub mod lin;
pub mod obd2;
pub mod someip;
pub mod uds;
pub mod xcp;

pub use can::{
    dlc_to_len, len_to_dlc, CanEncoder, CanParser, MIN_CANFD_FRAME_SIZE, MIN_CAN_FRAME_SIZE,
};

pub use j1939::{
    tp_control, AddressClaimManager, AddressClaimState, EcuName, J1939Frame, J1939Id, J1939MsgType,
    PgnType, TpSession, TpState, ADDRESS_ALL, ADDRESS_NULL, DEFAULT_PRIORITY, MAX_J1939_DATA_LEN,
    PGN_ACKNOWLEDGMENT, PGN_ADDRESS_CLAIMED, PGN_REQUEST, PGN_TP_CM, PGN_TP_DT,
};

pub use obd2::{
    decode_coolant_temp, decode_engine_load, decode_engine_rpm, decode_fuel_tank_level,
    decode_intake_air_temp, decode_maf_flow_rate, decode_throttle_position, decode_vehicle_speed,
    decode_vin, Obd2Dtc, Obd2Mode, Obd2Pid, Obd2Request, Obd2Response, VehicleInfoPid,
    OBD2_REQUEST_BROADCAST, OBD2_REQUEST_ECU_START, OBD2_RESPONSE_ECU_START,
};

pub use uds::{
    common_dids, AddressAndLengthFormatId, DataFormatId, DiagnosticSession, Dtc,
    DtcFormatIdentifier, DtcStatusMask, FileTransferMode, NegativeResponseCode, ReadDtcSubFunction,
    ResetType, RoutineControlType, SecurityAccessState, SecurityLevel, TransferDirection,
    TransferRequestResponse, TransferSession, UdsClient, UdsClientConfig, UdsClientError,
    UdsRequest, UdsResponse, UdsService,
};

pub use doip::{
    AliveCheckRequest, AliveCheckResponse, DiagnosticMessage, DiagnosticMessageAckCode,
    DiagnosticMessageNackCode, DiagnosticMessageNegativeAck, DiagnosticMessagePositiveAck,
    DoipClient, DoipClientState, DoipHeader, DoipMessage, DoipNackCode, DoipPayloadType,
    RoutingActivationRequest, RoutingActivationResponse, RoutingActivationResponseCode,
    VehicleIdentificationResponse, DOIP_HEADER_SIZE, DOIP_PROTOCOL_VERSION, DOIP_TCP_PORT,
    DOIP_UDP_PORT,
};

pub use someip::{
    Ipv4EndpointOption, SdEntryType, SdEventgroupEntry, SdOptionType, SdServiceEntry,
    SomeIpClient, SomeIpHeader, SomeIpMessage, SomeIpMessageType, SomeIpReturnCode,
    SomeIpTpHeader, TpSession as SomeIpTpSession, SOMEIP_HEADER_SIZE, SOMEIP_INTERFACE_VERSION,
    SOMEIP_PROTOCOL_VERSION, SOMEIP_SD_METHOD_ID, SOMEIP_SD_SERVICE_ID,
};

pub use lin::{
    LinChecksum, LinDirection, LinFrame, LinFrameDef, LinId, LinNad, LinNode, LinNodeType,
    LinSignal, ScheduleEntry, ScheduleEntryType, ScheduleTable, LIN_DIAG_MASTER_REQUEST,
    LIN_DIAG_SLAVE_RESPONSE, LIN_MAX_DATA_LEN,
};

pub use xcp::{
    DaqEvent, DaqList, DaqListMode, Odt, OdtEntry, XcpClient, XcpClientState, XcpCommand,
    XcpCommandCode, XcpConnectionMode, XcpErrorCode, XcpResources, XcpResponse, XcpResponsePid,
};

pub use calibration::{
    AxisDefinition, CalibrationDataType, CalibrationDataset, CalibrationParamType,
    CalibrationParameter, CalibrationSession, DaqEntry, DaqListBuilder, DefaultSeedKeyHandler,
    FlashProgress, FlashSector, FlashState, HexFile, HexFormat, HexRecord, MeasurementConfig,
    MeasurementRecording, MeasurementSignal, MemoryPage, MemoryPageType, PageSwitchMode,
    SecurityLevel as CalibrationSecurityLevel, SeedKeyHandler, SessionConfig, SessionState,
    SessionStatistics, TransportType,
};
