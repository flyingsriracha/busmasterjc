//! UDS (Unified Diagnostic Services) Protocol Implementation
//!
//! UDS is defined in ISO 14229 and provides diagnostic services for automotive ECUs.
//! It's commonly used over CAN (ISO 15765-2) and Ethernet (DoIP).
//!
//! # Key Concepts
//!
//! ## Service Identifiers (SIDs)
//! Each UDS service has a unique identifier (0x10-0x87). Responses use SID + 0x40.
//! Negative responses use SID 0x7F.
//!
//! ## Diagnostic Sessions
//! - Default Session (0x01): Limited services available
//! - Programming Session (0x02): For ECU reprogramming
//! - Extended Session (0x03): Full diagnostic access
//!
//! ## Security Access
//! Many services require security unlock via seed-key authentication.
//!
//! # Example
//!
//! ```
//! use busmaster_proto::uds::{UdsService, DiagnosticSession, UdsRequest};
//!
//! // Create a diagnostic session control request
//! let request = UdsRequest::diagnostic_session_control(DiagnosticSession::Extended);
//! assert_eq!(request.service(), UdsService::DiagnosticSessionControl);
//! ```

use serde::{Deserialize, Serialize};

/// UDS Service Identifiers (SIDs) per ISO 14229-1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum UdsService {
    /// Diagnostic Session Control (0x10)
    DiagnosticSessionControl = 0x10,
    /// ECU Reset (0x11)
    EcuReset = 0x11,
    /// Security Access (0x27)
    SecurityAccess = 0x27,
    /// Communication Control (0x28)
    CommunicationControl = 0x28,
    /// Tester Present (0x3E)
    TesterPresent = 0x3E,
    /// Access Timing Parameters (0x83)
    AccessTimingParameters = 0x83,
    /// Secured Data Transmission (0x84)
    SecuredDataTransmission = 0x84,
    /// Control DTC Setting (0x85)
    ControlDtcSetting = 0x85,
    /// Response On Event (0x86)
    ResponseOnEvent = 0x86,
    /// Link Control (0x87)
    LinkControl = 0x87,
    /// Read Data By Identifier (0x22)
    ReadDataByIdentifier = 0x22,
    /// Read Memory By Address (0x23)
    ReadMemoryByAddress = 0x23,
    /// Read Scaling Data By Identifier (0x24)
    ReadScalingDataByIdentifier = 0x24,
    /// Read Data By Periodic Identifier (0x2A)
    ReadDataByPeriodicIdentifier = 0x2A,
    /// Dynamically Define Data Identifier (0x2C)
    DynamicallyDefineDataIdentifier = 0x2C,
    /// Write Data By Identifier (0x2E)
    WriteDataByIdentifier = 0x2E,
    /// Write Memory By Address (0x3D)
    WriteMemoryByAddress = 0x3D,
    /// Clear Diagnostic Information (0x14)
    ClearDiagnosticInformation = 0x14,
    /// Read DTC Information (0x19)
    ReadDtcInformation = 0x19,
    /// Input Output Control By Identifier (0x2F)
    InputOutputControlByIdentifier = 0x2F,
    /// Routine Control (0x31)
    RoutineControl = 0x31,
    /// Request Download (0x34)
    RequestDownload = 0x34,
    /// Request Upload (0x35)
    RequestUpload = 0x35,
    /// Transfer Data (0x36)
    TransferData = 0x36,
    /// Request Transfer Exit (0x37)
    RequestTransferExit = 0x37,
    /// Request File Transfer (0x38)
    RequestFileTransfer = 0x38,
}

impl UdsService {
    /// Get the positive response SID (service ID + 0x40)
    #[must_use]
    pub fn positive_response_sid(self) -> u8 {
        (self as u8) + 0x40
    }

    /// Try to create a UdsService from a raw SID byte
    #[must_use]
    pub fn from_sid(sid: u8) -> Option<Self> {
        match sid {
            0x10 => Some(Self::DiagnosticSessionControl),
            0x11 => Some(Self::EcuReset),
            0x27 => Some(Self::SecurityAccess),
            0x28 => Some(Self::CommunicationControl),
            0x3E => Some(Self::TesterPresent),
            0x83 => Some(Self::AccessTimingParameters),
            0x84 => Some(Self::SecuredDataTransmission),
            0x85 => Some(Self::ControlDtcSetting),
            0x86 => Some(Self::ResponseOnEvent),
            0x87 => Some(Self::LinkControl),
            0x22 => Some(Self::ReadDataByIdentifier),
            0x23 => Some(Self::ReadMemoryByAddress),
            0x24 => Some(Self::ReadScalingDataByIdentifier),
            0x2A => Some(Self::ReadDataByPeriodicIdentifier),
            0x2C => Some(Self::DynamicallyDefineDataIdentifier),
            0x2E => Some(Self::WriteDataByIdentifier),
            0x3D => Some(Self::WriteMemoryByAddress),
            0x14 => Some(Self::ClearDiagnosticInformation),
            0x19 => Some(Self::ReadDtcInformation),
            0x2F => Some(Self::InputOutputControlByIdentifier),
            0x31 => Some(Self::RoutineControl),
            0x34 => Some(Self::RequestDownload),
            0x35 => Some(Self::RequestUpload),
            0x36 => Some(Self::TransferData),
            0x37 => Some(Self::RequestTransferExit),
            0x38 => Some(Self::RequestFileTransfer),
            _ => None,
        }
    }
}

/// Negative Response Codes (NRCs) per ISO 14229-1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NegativeResponseCode {
    /// General reject (0x10)
    GeneralReject = 0x10,
    /// Service not supported (0x11)
    ServiceNotSupported = 0x11,
    /// Sub-function not supported (0x12)
    SubFunctionNotSupported = 0x12,
    /// Incorrect message length or invalid format (0x13)
    IncorrectMessageLengthOrInvalidFormat = 0x13,
    /// Response too long (0x14)
    ResponseTooLong = 0x14,
    /// Busy repeat request (0x21)
    BusyRepeatRequest = 0x21,
    /// Conditions not correct (0x22)
    ConditionsNotCorrect = 0x22,
    /// Request sequence error (0x24)
    RequestSequenceError = 0x24,
    /// No response from subnet component (0x25)
    NoResponseFromSubnetComponent = 0x25,
    /// Failure prevents execution of requested action (0x26)
    FailurePreventsExecutionOfRequestedAction = 0x26,
    /// Request out of range (0x31)
    RequestOutOfRange = 0x31,
    /// Security access denied (0x33)
    SecurityAccessDenied = 0x33,
    /// Invalid key (0x35)
    InvalidKey = 0x35,
    /// Exceeded number of attempts (0x36)
    ExceededNumberOfAttempts = 0x36,
    /// Required time delay not expired (0x37)
    RequiredTimeDelayNotExpired = 0x37,
    /// Upload/download not accepted (0x70)
    UploadDownloadNotAccepted = 0x70,
    /// Transfer data suspended (0x71)
    TransferDataSuspended = 0x71,
    /// General programming failure (0x72)
    GeneralProgrammingFailure = 0x72,
    /// Wrong block sequence counter (0x73)
    WrongBlockSequenceCounter = 0x73,
    /// Request correctly received - response pending (0x78)
    RequestCorrectlyReceivedResponsePending = 0x78,
    /// Sub-function not supported in active session (0x7E)
    SubFunctionNotSupportedInActiveSession = 0x7E,
    /// Service not supported in active session (0x7F)
    ServiceNotSupportedInActiveSession = 0x7F,
    /// RPM too high (0x81)
    RpmTooHigh = 0x81,
    /// RPM too low (0x82)
    RpmTooLow = 0x82,
    /// Engine is running (0x83)
    EngineIsRunning = 0x83,
    /// Engine is not running (0x84)
    EngineIsNotRunning = 0x84,
    /// Engine run time too low (0x85)
    EngineRunTimeTooLow = 0x85,
    /// Temperature too high (0x86)
    TemperatureTooHigh = 0x86,
    /// Temperature too low (0x87)
    TemperatureTooLow = 0x87,
    /// Vehicle speed too high (0x88)
    VehicleSpeedTooHigh = 0x88,
    /// Vehicle speed too low (0x89)
    VehicleSpeedTooLow = 0x89,
    /// Throttle/pedal too high (0x8A)
    ThrottlePedalTooHigh = 0x8A,
    /// Throttle/pedal too low (0x8B)
    ThrottlePedalTooLow = 0x8B,
    /// Transmission range not in neutral (0x8C)
    TransmissionRangeNotInNeutral = 0x8C,
    /// Transmission range not in gear (0x8D)
    TransmissionRangeNotInGear = 0x8D,
    /// Brake switch not closed (0x8F)
    BrakeSwitchNotClosed = 0x8F,
    /// Shifter lever not in park (0x90)
    ShifterLeverNotInPark = 0x90,
    /// Torque converter clutch locked (0x91)
    TorqueConverterClutchLocked = 0x91,
    /// Voltage too high (0x92)
    VoltageTooHigh = 0x92,
    /// Voltage too low (0x93)
    VoltageTooLow = 0x93,
}

impl NegativeResponseCode {
    /// Try to create a NRC from a raw byte
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x10 => Some(Self::GeneralReject),
            0x11 => Some(Self::ServiceNotSupported),
            0x12 => Some(Self::SubFunctionNotSupported),
            0x13 => Some(Self::IncorrectMessageLengthOrInvalidFormat),
            0x14 => Some(Self::ResponseTooLong),
            0x21 => Some(Self::BusyRepeatRequest),
            0x22 => Some(Self::ConditionsNotCorrect),
            0x24 => Some(Self::RequestSequenceError),
            0x25 => Some(Self::NoResponseFromSubnetComponent),
            0x26 => Some(Self::FailurePreventsExecutionOfRequestedAction),
            0x31 => Some(Self::RequestOutOfRange),
            0x33 => Some(Self::SecurityAccessDenied),
            0x35 => Some(Self::InvalidKey),
            0x36 => Some(Self::ExceededNumberOfAttempts),
            0x37 => Some(Self::RequiredTimeDelayNotExpired),
            0x70 => Some(Self::UploadDownloadNotAccepted),
            0x71 => Some(Self::TransferDataSuspended),
            0x72 => Some(Self::GeneralProgrammingFailure),
            0x73 => Some(Self::WrongBlockSequenceCounter),
            0x78 => Some(Self::RequestCorrectlyReceivedResponsePending),
            0x7E => Some(Self::SubFunctionNotSupportedInActiveSession),
            0x7F => Some(Self::ServiceNotSupportedInActiveSession),
            0x81 => Some(Self::RpmTooHigh),
            0x82 => Some(Self::RpmTooLow),
            0x83 => Some(Self::EngineIsRunning),
            0x84 => Some(Self::EngineIsNotRunning),
            0x85 => Some(Self::EngineRunTimeTooLow),
            0x86 => Some(Self::TemperatureTooHigh),
            0x87 => Some(Self::TemperatureTooLow),
            0x88 => Some(Self::VehicleSpeedTooHigh),
            0x89 => Some(Self::VehicleSpeedTooLow),
            0x8A => Some(Self::ThrottlePedalTooHigh),
            0x8B => Some(Self::ThrottlePedalTooLow),
            0x8C => Some(Self::TransmissionRangeNotInNeutral),
            0x8D => Some(Self::TransmissionRangeNotInGear),
            0x8F => Some(Self::BrakeSwitchNotClosed),
            0x90 => Some(Self::ShifterLeverNotInPark),
            0x91 => Some(Self::TorqueConverterClutchLocked),
            0x92 => Some(Self::VoltageTooHigh),
            0x93 => Some(Self::VoltageTooLow),
            _ => None,
        }
    }

    /// Check if this NRC indicates a pending response
    #[must_use]
    pub fn is_pending(self) -> bool {
        self == Self::RequestCorrectlyReceivedResponsePending
    }
}

/// Diagnostic Session Types per ISO 14229-1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DiagnosticSession {
    /// Default session (0x01) - limited services
    Default = 0x01,
    /// Programming session (0x02) - for ECU reprogramming
    Programming = 0x02,
    /// Extended diagnostic session (0x03) - full access
    Extended = 0x03,
    /// Safety system diagnostic session (0x04)
    SafetySystem = 0x04,
}

impl DiagnosticSession {
    /// Try to create from a raw byte
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Default),
            0x02 => Some(Self::Programming),
            0x03 => Some(Self::Extended),
            0x04 => Some(Self::SafetySystem),
            _ => None,
        }
    }
}

/// ECU Reset Types per ISO 14229-1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResetType {
    /// Hard reset (0x01)
    HardReset = 0x01,
    /// Key off/on reset (0x02)
    KeyOffOnReset = 0x02,
    /// Soft reset (0x03)
    SoftReset = 0x03,
    /// Enable rapid power shutdown (0x04)
    EnableRapidPowerShutdown = 0x04,
    /// Disable rapid power shutdown (0x05)
    DisableRapidPowerShutdown = 0x05,
}

impl ResetType {
    /// Try to create from a raw byte
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::HardReset),
            0x02 => Some(Self::KeyOffOnReset),
            0x03 => Some(Self::SoftReset),
            0x04 => Some(Self::EnableRapidPowerShutdown),
            0x05 => Some(Self::DisableRapidPowerShutdown),
            _ => None,
        }
    }
}

/// Routine Control Types per ISO 14229-1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoutineControlType {
    /// Start routine (0x01)
    StartRoutine = 0x01,
    /// Stop routine (0x02)
    StopRoutine = 0x02,
    /// Request routine results (0x03)
    RequestRoutineResults = 0x03,
}

impl RoutineControlType {
    /// Try to create from a raw byte
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::StartRoutine),
            0x02 => Some(Self::StopRoutine),
            0x03 => Some(Self::RequestRoutineResults),
            _ => None,
        }
    }
}

/// Security Access State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityAccessState {
    /// Locked - security access not granted
    Locked,
    /// Seed requested - waiting for key
    SeedRequested,
    /// Unlocked - security access granted
    Unlocked,
}

/// Security Access Level (odd = request seed, even = send key)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityLevel(pub u8);

impl SecurityLevel {
    /// Create a new security level
    #[must_use]
    pub fn new(level: u8) -> Self {
        Self(level)
    }

    /// Get the request seed sub-function (odd number)
    #[must_use]
    pub fn request_seed(&self) -> u8 {
        if self.0 % 2 == 0 {
            self.0 - 1
        } else {
            self.0
        }
    }

    /// Get the send key sub-function (even number)
    #[must_use]
    pub fn send_key(&self) -> u8 {
        if self.0 % 2 == 0 {
            self.0
        } else {
            self.0 + 1
        }
    }

    /// Check if this is a request seed sub-function
    #[must_use]
    pub fn is_request_seed(&self) -> bool {
        self.0 % 2 == 1
    }

    /// Check if this is a send key sub-function
    #[must_use]
    pub fn is_send_key(&self) -> bool {
        self.0 % 2 == 0
    }
}

/// DTC (Diagnostic Trouble Code) Status Mask bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DtcStatusMask(pub u8);

impl DtcStatusMask {
    /// Test failed
    pub const TEST_FAILED: u8 = 0x01;
    /// Test failed this operation cycle
    pub const TEST_FAILED_THIS_CYCLE: u8 = 0x02;
    /// Pending DTC
    pub const PENDING_DTC: u8 = 0x04;
    /// Confirmed DTC
    pub const CONFIRMED_DTC: u8 = 0x08;
    /// Test not completed since last clear
    pub const TEST_NOT_COMPLETED_SINCE_CLEAR: u8 = 0x10;
    /// Test failed since last clear
    pub const TEST_FAILED_SINCE_CLEAR: u8 = 0x20;
    /// Test not completed this operation cycle
    pub const TEST_NOT_COMPLETED_THIS_CYCLE: u8 = 0x40;
    /// Warning indicator requested
    pub const WARNING_INDICATOR_REQUESTED: u8 = 0x80;

    /// Create a new status mask
    #[must_use]
    pub fn new(mask: u8) -> Self {
        Self(mask)
    }

    /// Check if test failed
    #[must_use]
    pub fn test_failed(&self) -> bool {
        self.0 & Self::TEST_FAILED != 0
    }

    /// Check if confirmed DTC
    #[must_use]
    pub fn is_confirmed(&self) -> bool {
        self.0 & Self::CONFIRMED_DTC != 0
    }

    /// Check if pending DTC
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.0 & Self::PENDING_DTC != 0
    }
}

/// DTC Format Identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DtcFormatIdentifier {
    /// ISO 15031-6 format (2 bytes)
    Iso15031_6 = 0x00,
    /// ISO 14229-1 format (3 bytes)
    Iso14229_1 = 0x01,
    /// SAE J1939-73 format (4 bytes)
    SaeJ1939_73 = 0x02,
    /// ISO 11992-4 format
    Iso11992_4 = 0x03,
}

/// DTC (Diagnostic Trouble Code)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dtc {
    /// High byte
    pub high: u8,
    /// Middle byte
    pub mid: u8,
    /// Low byte
    pub low: u8,
    /// Status byte
    pub status: DtcStatusMask,
}

impl Dtc {
    /// Create a new DTC
    #[must_use]
    pub fn new(high: u8, mid: u8, low: u8, status: u8) -> Self {
        Self {
            high,
            mid,
            low,
            status: DtcStatusMask::new(status),
        }
    }

    /// Get the 24-bit DTC number
    #[must_use]
    pub fn number(&self) -> u32 {
        ((self.high as u32) << 16) | ((self.mid as u32) << 8) | (self.low as u32)
    }

    /// Create from a 24-bit number and status
    #[must_use]
    pub fn from_number(number: u32, status: u8) -> Self {
        Self {
            high: ((number >> 16) & 0xFF) as u8,
            mid: ((number >> 8) & 0xFF) as u8,
            low: (number & 0xFF) as u8,
            status: DtcStatusMask::new(status),
        }
    }
}

/// Read DTC Information sub-functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ReadDtcSubFunction {
    /// Report number of DTC by status mask (0x01)
    ReportNumberOfDtcByStatusMask = 0x01,
    /// Report DTC by status mask (0x02)
    ReportDtcByStatusMask = 0x02,
    /// Report DTC snapshot identification (0x03)
    ReportDtcSnapshotIdentification = 0x03,
    /// Report DTC snapshot record by DTC number (0x04)
    ReportDtcSnapshotRecordByDtcNumber = 0x04,
    /// Report DTC stored data by record number (0x05)
    ReportDtcStoredDataByRecordNumber = 0x05,
    /// Report DTC extended data record by DTC number (0x06)
    ReportDtcExtDataRecordByDtcNumber = 0x06,
    /// Report number of DTC by severity mask (0x07)
    ReportNumberOfDtcBySeverityMask = 0x07,
    /// Report DTC by severity mask (0x08)
    ReportDtcBySeverityMask = 0x08,
    /// Report severity information of DTC (0x09)
    ReportSeverityInformationOfDtc = 0x09,
    /// Report supported DTC (0x0A)
    ReportSupportedDtc = 0x0A,
    /// Report first test failed DTC (0x0B)
    ReportFirstTestFailedDtc = 0x0B,
    /// Report first confirmed DTC (0x0C)
    ReportFirstConfirmedDtc = 0x0C,
    /// Report most recent test failed DTC (0x0D)
    ReportMostRecentTestFailedDtc = 0x0D,
    /// Report most recent confirmed DTC (0x0E)
    ReportMostRecentConfirmedDtc = 0x0E,
    /// Report DTC fault detection counter (0x14)
    ReportDtcFaultDetectionCounter = 0x14,
    /// Report DTC with permanent status (0x15)
    ReportDtcWithPermanentStatus = 0x15,
}

/// UDS Request message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UdsRequest {
    /// Service identifier
    service: UdsService,
    /// Sub-function (if applicable)
    sub_function: Option<u8>,
    /// Request data
    data: Vec<u8>,
}

impl UdsRequest {
    /// Create a new UDS request
    #[must_use]
    pub fn new(service: UdsService, sub_function: Option<u8>, data: Vec<u8>) -> Self {
        Self {
            service,
            sub_function,
            data,
        }
    }

    /// Create a Diagnostic Session Control request
    #[must_use]
    pub fn diagnostic_session_control(session: DiagnosticSession) -> Self {
        Self::new(
            UdsService::DiagnosticSessionControl,
            Some(session as u8),
            vec![],
        )
    }

    /// Create an ECU Reset request
    #[must_use]
    pub fn ecu_reset(reset_type: ResetType) -> Self {
        Self::new(UdsService::EcuReset, Some(reset_type as u8), vec![])
    }

    /// Create a Security Access request (request seed)
    #[must_use]
    pub fn security_access_request_seed(level: SecurityLevel) -> Self {
        Self::new(
            UdsService::SecurityAccess,
            Some(level.request_seed()),
            vec![],
        )
    }

    /// Create a Security Access request (send key)
    #[must_use]
    pub fn security_access_send_key(level: SecurityLevel, key: &[u8]) -> Self {
        Self::new(
            UdsService::SecurityAccess,
            Some(level.send_key()),
            key.to_vec(),
        )
    }

    /// Create a Tester Present request
    #[must_use]
    pub fn tester_present(suppress_response: bool) -> Self {
        let sub_function = if suppress_response { 0x80 } else { 0x00 };
        Self::new(UdsService::TesterPresent, Some(sub_function), vec![])
    }

    /// Create a Read Data By Identifier request
    #[must_use]
    pub fn read_data_by_identifier(dids: &[u16]) -> Self {
        let mut data = Vec::with_capacity(dids.len() * 2);
        for did in dids {
            data.push((did >> 8) as u8);
            data.push((did & 0xFF) as u8);
        }
        Self::new(UdsService::ReadDataByIdentifier, None, data)
    }

    /// Create a Write Data By Identifier request
    #[must_use]
    pub fn write_data_by_identifier(did: u16, value: &[u8]) -> Self {
        let mut data = Vec::with_capacity(2 + value.len());
        data.push((did >> 8) as u8);
        data.push((did & 0xFF) as u8);
        data.extend_from_slice(value);
        Self::new(UdsService::WriteDataByIdentifier, None, data)
    }

    /// Create a Clear Diagnostic Information request
    #[must_use]
    pub fn clear_diagnostic_information(group: u32) -> Self {
        let data = vec![
            ((group >> 16) & 0xFF) as u8,
            ((group >> 8) & 0xFF) as u8,
            (group & 0xFF) as u8,
        ];
        Self::new(UdsService::ClearDiagnosticInformation, None, data)
    }

    /// Create a Read DTC Information request
    #[must_use]
    pub fn read_dtc_information(sub_function: ReadDtcSubFunction, status_mask: u8) -> Self {
        Self::new(
            UdsService::ReadDtcInformation,
            Some(sub_function as u8),
            vec![status_mask],
        )
    }

    /// Create a Routine Control request
    #[must_use]
    pub fn routine_control(
        control_type: RoutineControlType,
        routine_id: u16,
        option_record: &[u8],
    ) -> Self {
        let mut data = Vec::with_capacity(2 + option_record.len());
        data.push((routine_id >> 8) as u8);
        data.push((routine_id & 0xFF) as u8);
        data.extend_from_slice(option_record);
        Self::new(UdsService::RoutineControl, Some(control_type as u8), data)
    }

    /// Get the service
    #[must_use]
    pub fn service(&self) -> UdsService {
        self.service
    }

    /// Get the sub-function
    #[must_use]
    pub fn sub_function(&self) -> Option<u8> {
        self.sub_function
    }

    /// Get the data
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Encode the request to bytes
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes =
            Vec::with_capacity(1 + self.sub_function.map_or(0, |_| 1) + self.data.len());
        bytes.push(self.service as u8);
        if let Some(sf) = self.sub_function {
            bytes.push(sf);
        }
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

/// UDS Response types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UdsResponse {
    /// Positive response
    Positive {
        /// Service that was requested
        service: UdsService,
        /// Sub-function echo (if applicable)
        sub_function: Option<u8>,
        /// Response data
        data: Vec<u8>,
    },
    /// Negative response
    Negative {
        /// Service that was rejected
        service: UdsService,
        /// Negative response code
        nrc: NegativeResponseCode,
    },
    /// Negative response with unknown NRC
    NegativeUnknown {
        /// Service that was rejected
        service: UdsService,
        /// Raw NRC byte
        nrc_byte: u8,
    },
}

impl UdsResponse {
    /// Parse a UDS response from bytes
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let sid = bytes[0];

        // Check for negative response (0x7F)
        if sid == 0x7F {
            if bytes.len() < 3 {
                return None;
            }
            let service = UdsService::from_sid(bytes[1])?;
            let nrc_byte = bytes[2];

            if let Some(nrc) = NegativeResponseCode::from_byte(nrc_byte) {
                return Some(Self::Negative { service, nrc });
            } else {
                return Some(Self::NegativeUnknown { service, nrc_byte });
            }
        }

        // Positive response (SID + 0x40)
        if sid >= 0x50 {
            let service = UdsService::from_sid(sid - 0x40)?;
            let (sub_function, data_start) = if bytes.len() > 1 {
                // Check if this service uses sub-functions
                match service {
                    UdsService::DiagnosticSessionControl
                    | UdsService::EcuReset
                    | UdsService::SecurityAccess
                    | UdsService::CommunicationControl
                    | UdsService::TesterPresent
                    | UdsService::ControlDtcSetting
                    | UdsService::ReadDtcInformation
                    | UdsService::RoutineControl => (Some(bytes[1]), 2),
                    _ => (None, 1),
                }
            } else {
                (None, 1)
            };

            let data = if bytes.len() > data_start {
                bytes[data_start..].to_vec()
            } else {
                vec![]
            };

            return Some(Self::Positive {
                service,
                sub_function,
                data,
            });
        }

        None
    }

    /// Check if this is a positive response
    #[must_use]
    pub fn is_positive(&self) -> bool {
        matches!(self, Self::Positive { .. })
    }

    /// Check if this is a negative response
    #[must_use]
    pub fn is_negative(&self) -> bool {
        matches!(self, Self::Negative { .. } | Self::NegativeUnknown { .. })
    }

    /// Check if this is a "response pending" negative response
    #[must_use]
    pub fn is_pending(&self) -> bool {
        matches!(
            self,
            Self::Negative {
                nrc: NegativeResponseCode::RequestCorrectlyReceivedResponsePending,
                ..
            }
        )
    }
}

/// Data Transfer Direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferDirection {
    /// Download (tester to ECU)
    Download,
    /// Upload (ECU to tester)
    Upload,
}

/// Memory Address and Length Format Identifier
///
/// Encodes the number of bytes used for memory address and memory size fields.
/// High nibble = address length (1-5 bytes), Low nibble = size length (1-4 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressAndLengthFormatId {
    /// Number of bytes for memory address (1-5)
    pub address_bytes: u8,
    /// Number of bytes for memory size (1-4)
    pub size_bytes: u8,
}

impl AddressAndLengthFormatId {
    /// Create a new format identifier
    ///
    /// # Panics
    /// Panics if address_bytes is not 1-5 or size_bytes is not 1-4
    #[must_use]
    pub fn new(address_bytes: u8, size_bytes: u8) -> Self {
        assert!(
            (1..=5).contains(&address_bytes),
            "address_bytes must be 1-5"
        );
        assert!((1..=4).contains(&size_bytes), "size_bytes must be 1-4");
        Self {
            address_bytes,
            size_bytes,
        }
    }

    /// Encode to a single byte
    #[must_use]
    pub fn encode(&self) -> u8 {
        (self.address_bytes << 4) | self.size_bytes
    }

    /// Decode from a single byte
    #[must_use]
    pub fn decode(byte: u8) -> Self {
        Self {
            address_bytes: (byte >> 4) & 0x0F,
            size_bytes: byte & 0x0F,
        }
    }
}

/// Data Format Identifier for compression and encryption
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DataFormatId {
    /// Compression method (0 = no compression)
    pub compression_method: u8,
    /// Encryption method (0 = no encryption)
    pub encrypting_method: u8,
}

impl DataFormatId {
    /// No compression or encryption
    pub const NONE: Self = Self {
        compression_method: 0,
        encrypting_method: 0,
    };

    /// Create a new data format identifier
    #[must_use]
    pub fn new(compression_method: u8, encrypting_method: u8) -> Self {
        Self {
            compression_method,
            encrypting_method,
        }
    }

    /// Encode to a single byte
    #[must_use]
    pub fn encode(&self) -> u8 {
        (self.compression_method << 4) | self.encrypting_method
    }

    /// Decode from a single byte
    #[must_use]
    pub fn decode(byte: u8) -> Self {
        Self {
            compression_method: (byte >> 4) & 0x0F,
            encrypting_method: byte & 0x0F,
        }
    }
}

/// Transfer session state for managing download/upload operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferSession {
    /// Transfer direction
    pub direction: TransferDirection,
    /// Memory address being transferred
    pub memory_address: u64,
    /// Total memory size to transfer
    pub memory_size: u32,
    /// Maximum number of block length (from ECU response)
    pub max_block_length: u32,
    /// Current block sequence counter (1-255, wraps to 0)
    pub block_sequence_counter: u8,
    /// Total bytes transferred so far
    pub bytes_transferred: u32,
    /// Whether the transfer is complete
    pub is_complete: bool,
}

impl TransferSession {
    /// Create a new transfer session
    #[must_use]
    pub fn new(
        direction: TransferDirection,
        memory_address: u64,
        memory_size: u32,
        max_block_length: u32,
    ) -> Self {
        Self {
            direction,
            memory_address,
            memory_size,
            max_block_length,
            block_sequence_counter: 1,
            bytes_transferred: 0,
            is_complete: false,
        }
    }

    /// Get the next block sequence counter and increment
    pub fn next_block_counter(&mut self) -> u8 {
        let counter = self.block_sequence_counter;
        self.block_sequence_counter = self.block_sequence_counter.wrapping_add(1);
        if self.block_sequence_counter == 0 {
            self.block_sequence_counter = 1; // Skip 0 per ISO 14229
        }
        counter
    }

    /// Record bytes transferred
    pub fn record_transfer(&mut self, bytes: u32) {
        self.bytes_transferred += bytes;
        if self.bytes_transferred >= self.memory_size {
            self.is_complete = true;
        }
    }

    /// Get remaining bytes to transfer
    #[must_use]
    pub fn remaining_bytes(&self) -> u32 {
        self.memory_size.saturating_sub(self.bytes_transferred)
    }

    /// Get progress as a percentage (0-100)
    #[must_use]
    pub fn progress_percent(&self) -> u8 {
        if self.memory_size == 0 {
            return 100;
        }
        let percent = (self.bytes_transferred as u64 * 100) / self.memory_size as u64;
        percent.min(100) as u8
    }
}

impl UdsRequest {
    /// Create a Request Download request (0x34)
    ///
    /// Used to initiate a download (tester to ECU) transfer.
    #[must_use]
    pub fn request_download(
        memory_address: u64,
        memory_size: u32,
        format: AddressAndLengthFormatId,
        data_format: DataFormatId,
    ) -> Self {
        let mut data =
            Vec::with_capacity(1 + format.address_bytes as usize + format.size_bytes as usize);
        data.push(data_format.encode());
        data.push(format.encode());

        // Add memory address (big-endian, variable length)
        for i in (0..format.address_bytes).rev() {
            data.push(((memory_address >> (i * 8)) & 0xFF) as u8);
        }

        // Add memory size (big-endian, variable length)
        for i in (0..format.size_bytes).rev() {
            data.push(((memory_size >> (i * 8)) & 0xFF) as u8);
        }

        Self::new(UdsService::RequestDownload, None, data)
    }

    /// Create a Request Upload request (0x35)
    ///
    /// Used to initiate an upload (ECU to tester) transfer.
    #[must_use]
    pub fn request_upload(
        memory_address: u64,
        memory_size: u32,
        format: AddressAndLengthFormatId,
        data_format: DataFormatId,
    ) -> Self {
        let mut data =
            Vec::with_capacity(1 + format.address_bytes as usize + format.size_bytes as usize);
        data.push(data_format.encode());
        data.push(format.encode());

        // Add memory address (big-endian, variable length)
        for i in (0..format.address_bytes).rev() {
            data.push(((memory_address >> (i * 8)) & 0xFF) as u8);
        }

        // Add memory size (big-endian, variable length)
        for i in (0..format.size_bytes).rev() {
            data.push(((memory_size >> (i * 8)) & 0xFF) as u8);
        }

        Self::new(UdsService::RequestUpload, None, data)
    }

    /// Create a Transfer Data request (0x36)
    ///
    /// Used to transfer data blocks during download/upload.
    #[must_use]
    pub fn transfer_data(block_sequence_counter: u8, data: &[u8]) -> Self {
        let mut request_data = Vec::with_capacity(1 + data.len());
        request_data.push(block_sequence_counter);
        request_data.extend_from_slice(data);
        Self::new(UdsService::TransferData, None, request_data)
    }

    /// Create a Request Transfer Exit request (0x37)
    ///
    /// Used to terminate a download/upload transfer.
    #[must_use]
    pub fn request_transfer_exit(transfer_request_parameter: Option<&[u8]>) -> Self {
        let data = transfer_request_parameter.map_or_else(Vec::new, <[u8]>::to_vec);
        Self::new(UdsService::RequestTransferExit, None, data)
    }

    /// Create a Request File Transfer request (0x38)
    ///
    /// Used for file-based transfers (ISO 14229-1:2020).
    #[must_use]
    pub fn request_file_transfer(
        mode_of_operation: FileTransferMode,
        file_path: &str,
        data_format: DataFormatId,
        file_size: Option<u64>,
    ) -> Self {
        let path_bytes = file_path.as_bytes();
        let path_len = path_bytes.len().min(255) as u8;

        let mut data = Vec::with_capacity(4 + path_bytes.len() + 8);
        data.push(mode_of_operation as u8);
        data.push(path_len);
        data.extend_from_slice(&path_bytes[..path_len as usize]);
        data.push(data_format.encode());

        // Add file size if provided (for add/replace operations)
        if let Some(size) = file_size {
            // Use 4 bytes for file size length indicator
            data.push(0x04);
            data.push(((size >> 24) & 0xFF) as u8);
            data.push(((size >> 16) & 0xFF) as u8);
            data.push(((size >> 8) & 0xFF) as u8);
            data.push((size & 0xFF) as u8);
        }

        Self::new(UdsService::RequestFileTransfer, None, data)
    }
}

/// File Transfer Mode for Request File Transfer (0x38)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FileTransferMode {
    /// Add file (0x01)
    AddFile = 0x01,
    /// Delete file (0x02)
    DeleteFile = 0x02,
    /// Replace file (0x03)
    ReplaceFile = 0x03,
    /// Read file (0x04)
    ReadFile = 0x04,
    /// Read directory (0x05)
    ReadDir = 0x05,
}

impl FileTransferMode {
    /// Try to create from a raw byte
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::AddFile),
            0x02 => Some(Self::DeleteFile),
            0x03 => Some(Self::ReplaceFile),
            0x04 => Some(Self::ReadFile),
            0x05 => Some(Self::ReadDir),
            _ => None,
        }
    }
}

/// Response data from Request Download/Upload
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequestResponse {
    /// Length format identifier (number of bytes for max block length)
    pub length_format_id: u8,
    /// Maximum number of bytes per TransferData request
    pub max_number_of_block_length: u32,
}

impl TransferRequestResponse {
    /// Parse from response data bytes
    #[must_use]
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        let length_format_id = data[0] >> 4;
        let num_bytes = (length_format_id & 0x0F) as usize;

        if data.len() < 1 + num_bytes {
            return None;
        }

        let mut max_block_length: u32 = 0;
        for i in 0..num_bytes {
            max_block_length = (max_block_length << 8) | data[1 + i] as u32;
        }

        Some(Self {
            length_format_id,
            max_number_of_block_length: max_block_length,
        })
    }
}

/// UDS Client for managing diagnostic sessions
///
/// The UDS client provides a high-level interface for UDS communication,
/// handling session management, security access, and response pending.
///
/// # Example
///
/// ```
/// use busmaster_proto::uds::{UdsClient, UdsClientConfig, DiagnosticSession};
///
/// let config = UdsClientConfig::default();
/// let mut client = UdsClient::new(config);
///
/// // Start an extended diagnostic session
/// let request = client.start_session(DiagnosticSession::Extended);
/// // Send request and process response...
/// ```
#[derive(Debug, Clone)]
pub struct UdsClient {
    /// Client configuration
    config: UdsClientConfig,
    /// Current diagnostic session
    current_session: DiagnosticSession,
    /// Security access state
    security_state: SecurityAccessState,
    /// Current security level (if unlocked)
    security_level: Option<SecurityLevel>,
    /// Active transfer session (if any)
    transfer_session: Option<TransferSession>,
    /// Pending response count (for response pending handling)
    pending_count: u32,
}

/// Configuration for UDS client
#[derive(Debug, Clone)]
pub struct UdsClientConfig {
    /// P2 timeout in milliseconds (default response timeout)
    pub p2_timeout_ms: u32,
    /// P2* timeout in milliseconds (extended response timeout after 0x78)
    pub p2_star_timeout_ms: u32,
    /// Maximum number of response pending (0x78) responses to wait for
    pub max_pending_responses: u32,
    /// Whether to suppress positive responses where possible
    pub suppress_positive_response: bool,
    /// Tester present interval in milliseconds (0 = disabled)
    pub tester_present_interval_ms: u32,
}

impl Default for UdsClientConfig {
    fn default() -> Self {
        Self {
            p2_timeout_ms: 50,
            p2_star_timeout_ms: 5000,
            max_pending_responses: 100,
            suppress_positive_response: false,
            tester_present_interval_ms: 2000,
        }
    }
}

impl UdsClient {
    /// Create a new UDS client with the given configuration
    #[must_use]
    pub fn new(config: UdsClientConfig) -> Self {
        Self {
            config,
            current_session: DiagnosticSession::Default,
            security_state: SecurityAccessState::Locked,
            security_level: None,
            transfer_session: None,
            pending_count: 0,
        }
    }

    /// Get the current configuration
    #[must_use]
    pub fn config(&self) -> &UdsClientConfig {
        &self.config
    }

    /// Get the current diagnostic session
    #[must_use]
    pub fn current_session(&self) -> DiagnosticSession {
        self.current_session
    }

    /// Get the security access state
    #[must_use]
    pub fn security_state(&self) -> SecurityAccessState {
        self.security_state
    }

    /// Get the current security level (if unlocked)
    #[must_use]
    pub fn security_level(&self) -> Option<SecurityLevel> {
        self.security_level
    }

    /// Get the active transfer session (if any)
    #[must_use]
    pub fn transfer_session(&self) -> Option<&TransferSession> {
        self.transfer_session.as_ref()
    }

    /// Create a request to start a diagnostic session
    #[must_use]
    pub fn start_session(&self, session: DiagnosticSession) -> UdsRequest {
        UdsRequest::diagnostic_session_control(session)
    }

    /// Process a diagnostic session control response
    pub fn process_session_response(
        &mut self,
        response: &UdsResponse,
    ) -> Result<(), UdsClientError> {
        match response {
            UdsResponse::Positive {
                service,
                sub_function,
                ..
            } => {
                if *service != UdsService::DiagnosticSessionControl {
                    return Err(UdsClientError::UnexpectedService(*service));
                }
                if let Some(sf) = sub_function {
                    if let Some(session) = DiagnosticSession::from_byte(*sf & 0x7F) {
                        self.current_session = session;
                        // Reset security state on session change
                        self.security_state = SecurityAccessState::Locked;
                        self.security_level = None;
                        Ok(())
                    } else {
                        Err(UdsClientError::InvalidSubFunction(*sf))
                    }
                } else {
                    Err(UdsClientError::MissingSubFunction)
                }
            },
            UdsResponse::Negative { nrc, .. } => Err(UdsClientError::NegativeResponse(*nrc)),
            UdsResponse::NegativeUnknown { nrc_byte, .. } => {
                Err(UdsClientError::UnknownNrc(*nrc_byte))
            },
        }
    }

    /// Create a request to request a security seed
    #[must_use]
    pub fn request_seed(&self, level: SecurityLevel) -> UdsRequest {
        UdsRequest::security_access_request_seed(level)
    }

    /// Process a security seed response and return the seed
    pub fn process_seed_response(
        &mut self,
        response: &UdsResponse,
        level: SecurityLevel,
    ) -> Result<Vec<u8>, UdsClientError> {
        match response {
            UdsResponse::Positive {
                service,
                sub_function,
                data,
            } => {
                if *service != UdsService::SecurityAccess {
                    return Err(UdsClientError::UnexpectedService(*service));
                }
                if let Some(sf) = sub_function {
                    if *sf != level.request_seed() {
                        return Err(UdsClientError::InvalidSubFunction(*sf));
                    }
                    self.security_state = SecurityAccessState::SeedRequested;
                    Ok(data.clone())
                } else {
                    Err(UdsClientError::MissingSubFunction)
                }
            },
            UdsResponse::Negative { nrc, .. } => Err(UdsClientError::NegativeResponse(*nrc)),
            UdsResponse::NegativeUnknown { nrc_byte, .. } => {
                Err(UdsClientError::UnknownNrc(*nrc_byte))
            },
        }
    }

    /// Create a request to send a security key
    #[must_use]
    pub fn send_key(&self, level: SecurityLevel, key: &[u8]) -> UdsRequest {
        UdsRequest::security_access_send_key(level, key)
    }

    /// Process a security key response
    pub fn process_key_response(
        &mut self,
        response: &UdsResponse,
        level: SecurityLevel,
    ) -> Result<(), UdsClientError> {
        match response {
            UdsResponse::Positive {
                service,
                sub_function,
                ..
            } => {
                if *service != UdsService::SecurityAccess {
                    return Err(UdsClientError::UnexpectedService(*service));
                }
                if let Some(sf) = sub_function {
                    if *sf != level.send_key() {
                        return Err(UdsClientError::InvalidSubFunction(*sf));
                    }
                    self.security_state = SecurityAccessState::Unlocked;
                    self.security_level = Some(level);
                    Ok(())
                } else {
                    Err(UdsClientError::MissingSubFunction)
                }
            },
            UdsResponse::Negative { nrc, .. } => {
                self.security_state = SecurityAccessState::Locked;
                Err(UdsClientError::NegativeResponse(*nrc))
            },
            UdsResponse::NegativeUnknown { nrc_byte, .. } => {
                self.security_state = SecurityAccessState::Locked;
                Err(UdsClientError::UnknownNrc(*nrc_byte))
            },
        }
    }

    /// Create a tester present request
    #[must_use]
    pub fn tester_present(&self) -> UdsRequest {
        UdsRequest::tester_present(self.config.suppress_positive_response)
    }

    /// Start a download transfer session
    pub fn start_download(
        &mut self,
        memory_address: u64,
        memory_size: u32,
        format: AddressAndLengthFormatId,
        data_format: DataFormatId,
    ) -> UdsRequest {
        self.transfer_session = None; // Clear any existing session
        UdsRequest::request_download(memory_address, memory_size, format, data_format)
    }

    /// Start an upload transfer session
    pub fn start_upload(
        &mut self,
        memory_address: u64,
        memory_size: u32,
        format: AddressAndLengthFormatId,
        data_format: DataFormatId,
    ) -> UdsRequest {
        self.transfer_session = None; // Clear any existing session
        UdsRequest::request_upload(memory_address, memory_size, format, data_format)
    }

    /// Process a download/upload request response
    pub fn process_transfer_request_response(
        &mut self,
        response: &UdsResponse,
        direction: TransferDirection,
        memory_address: u64,
        memory_size: u32,
    ) -> Result<u32, UdsClientError> {
        match response {
            UdsResponse::Positive { service, data, .. } => {
                let expected_service = match direction {
                    TransferDirection::Download => UdsService::RequestDownload,
                    TransferDirection::Upload => UdsService::RequestUpload,
                };
                if *service != expected_service {
                    return Err(UdsClientError::UnexpectedService(*service));
                }

                let transfer_response = TransferRequestResponse::parse(data)
                    .ok_or(UdsClientError::InvalidResponseData)?;

                self.transfer_session = Some(TransferSession::new(
                    direction,
                    memory_address,
                    memory_size,
                    transfer_response.max_number_of_block_length,
                ));

                Ok(transfer_response.max_number_of_block_length)
            },
            UdsResponse::Negative { nrc, .. } => Err(UdsClientError::NegativeResponse(*nrc)),
            UdsResponse::NegativeUnknown { nrc_byte, .. } => {
                Err(UdsClientError::UnknownNrc(*nrc_byte))
            },
        }
    }

    /// Create the next transfer data request for a download
    ///
    /// Returns None if there's no active transfer session or if the transfer is complete.
    pub fn next_transfer_data(&mut self, data: &[u8]) -> Option<UdsRequest> {
        let session = self.transfer_session.as_mut()?;
        if session.is_complete {
            return None;
        }

        let counter = session.next_block_counter();
        Some(UdsRequest::transfer_data(counter, data))
    }

    /// Process a transfer data response
    pub fn process_transfer_data_response(
        &mut self,
        response: &UdsResponse,
        bytes_transferred: u32,
    ) -> Result<(), UdsClientError> {
        match response {
            UdsResponse::Positive { service, .. } => {
                if *service != UdsService::TransferData {
                    return Err(UdsClientError::UnexpectedService(*service));
                }

                if let Some(session) = &mut self.transfer_session {
                    session.record_transfer(bytes_transferred);
                }
                Ok(())
            },
            UdsResponse::Negative { nrc, .. } => Err(UdsClientError::NegativeResponse(*nrc)),
            UdsResponse::NegativeUnknown { nrc_byte, .. } => {
                Err(UdsClientError::UnknownNrc(*nrc_byte))
            },
        }
    }

    /// Create a request to exit the transfer
    #[must_use]
    pub fn request_transfer_exit(&self) -> UdsRequest {
        UdsRequest::request_transfer_exit(None)
    }

    /// Process a transfer exit response
    pub fn process_transfer_exit_response(
        &mut self,
        response: &UdsResponse,
    ) -> Result<(), UdsClientError> {
        match response {
            UdsResponse::Positive { service, .. } => {
                if *service != UdsService::RequestTransferExit {
                    return Err(UdsClientError::UnexpectedService(*service));
                }
                self.transfer_session = None;
                Ok(())
            },
            UdsResponse::Negative { nrc, .. } => Err(UdsClientError::NegativeResponse(*nrc)),
            UdsResponse::NegativeUnknown { nrc_byte, .. } => {
                Err(UdsClientError::UnknownNrc(*nrc_byte))
            },
        }
    }

    /// Check if a response is "response pending" and should wait for more
    pub fn handle_response_pending(&mut self, response: &UdsResponse) -> bool {
        if response.is_pending() {
            self.pending_count += 1;
            self.pending_count <= self.config.max_pending_responses
        } else {
            self.pending_count = 0;
            false
        }
    }

    /// Get the appropriate timeout for the current state
    #[must_use]
    pub fn current_timeout_ms(&self) -> u32 {
        if self.pending_count > 0 {
            self.config.p2_star_timeout_ms
        } else {
            self.config.p2_timeout_ms
        }
    }

    /// Reset the client state (e.g., after communication error)
    pub fn reset(&mut self) {
        self.current_session = DiagnosticSession::Default;
        self.security_state = SecurityAccessState::Locked;
        self.security_level = None;
        self.transfer_session = None;
        self.pending_count = 0;
    }
}

/// Errors that can occur during UDS client operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdsClientError {
    /// Received a negative response
    NegativeResponse(NegativeResponseCode),
    /// Received an unknown negative response code
    UnknownNrc(u8),
    /// Received response for unexpected service
    UnexpectedService(UdsService),
    /// Missing sub-function in response
    MissingSubFunction,
    /// Invalid sub-function value
    InvalidSubFunction(u8),
    /// Invalid response data
    InvalidResponseData,
    /// Transfer session not active
    NoActiveTransfer,
    /// Maximum pending responses exceeded
    MaxPendingExceeded,
}

impl std::fmt::Display for UdsClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeResponse(nrc) => write!(f, "Negative response: {:?}", nrc),
            Self::UnknownNrc(byte) => write!(f, "Unknown NRC: 0x{:02X}", byte),
            Self::UnexpectedService(service) => write!(f, "Unexpected service: {:?}", service),
            Self::MissingSubFunction => write!(f, "Missing sub-function in response"),
            Self::InvalidSubFunction(sf) => write!(f, "Invalid sub-function: 0x{:02X}", sf),
            Self::InvalidResponseData => write!(f, "Invalid response data"),
            Self::NoActiveTransfer => write!(f, "No active transfer session"),
            Self::MaxPendingExceeded => write!(f, "Maximum pending responses exceeded"),
        }
    }
}

impl std::error::Error for UdsClientError {}

/// Common Data Identifiers (DIDs)
pub mod common_dids {
    /// VIN (Vehicle Identification Number)
    pub const VIN: u16 = 0xF190;
    /// ECU Serial Number
    pub const ECU_SERIAL_NUMBER: u16 = 0xF18C;
    /// ECU Hardware Number
    pub const ECU_HARDWARE_NUMBER: u16 = 0xF191;
    /// ECU Hardware Version
    pub const ECU_HARDWARE_VERSION: u16 = 0xF193;
    /// ECU Software Number
    pub const ECU_SOFTWARE_NUMBER: u16 = 0xF194;
    /// ECU Software Version
    pub const ECU_SOFTWARE_VERSION: u16 = 0xF195;
    /// System Supplier ECU Hardware Number
    pub const SYSTEM_SUPPLIER_ECU_HW_NUMBER: u16 = 0xF192;
    /// System Supplier ECU Software Number
    pub const SYSTEM_SUPPLIER_ECU_SW_NUMBER: u16 = 0xF194;
    /// Boot Software Identification
    pub const BOOT_SOFTWARE_ID: u16 = 0xF180;
    /// Application Software Identification
    pub const APPLICATION_SOFTWARE_ID: u16 = 0xF181;
    /// Application Data Identification
    pub const APPLICATION_DATA_ID: u16 = 0xF182;
    /// Active Diagnostic Session
    pub const ACTIVE_DIAGNOSTIC_SESSION: u16 = 0xF186;
    /// Vehicle Manufacturer Spare Part Number
    pub const SPARE_PART_NUMBER: u16 = 0xF187;
    /// System Supplier Identifier
    pub const SYSTEM_SUPPLIER_ID: u16 = 0xF18A;
    /// ECU Manufacturing Date
    pub const ECU_MANUFACTURING_DATE: u16 = 0xF18B;
    /// Programming Date
    pub const PROGRAMMING_DATE: u16 = 0xF199;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uds_service_positive_response() {
        assert_eq!(
            UdsService::DiagnosticSessionControl.positive_response_sid(),
            0x50
        );
        assert_eq!(
            UdsService::ReadDataByIdentifier.positive_response_sid(),
            0x62
        );
        assert_eq!(UdsService::SecurityAccess.positive_response_sid(), 0x67);
    }

    #[test]
    fn test_uds_service_from_sid() {
        assert_eq!(
            UdsService::from_sid(0x10),
            Some(UdsService::DiagnosticSessionControl)
        );
        assert_eq!(
            UdsService::from_sid(0x22),
            Some(UdsService::ReadDataByIdentifier)
        );
        assert_eq!(UdsService::from_sid(0xFF), None);
    }

    #[test]
    fn test_diagnostic_session_control_request() {
        let request = UdsRequest::diagnostic_session_control(DiagnosticSession::Extended);
        assert_eq!(request.service(), UdsService::DiagnosticSessionControl);
        assert_eq!(request.sub_function(), Some(0x03));

        let encoded = request.encode();
        assert_eq!(encoded, vec![0x10, 0x03]);
    }

    #[test]
    fn test_ecu_reset_request() {
        let request = UdsRequest::ecu_reset(ResetType::HardReset);
        assert_eq!(request.service(), UdsService::EcuReset);
        assert_eq!(request.sub_function(), Some(0x01));

        let encoded = request.encode();
        assert_eq!(encoded, vec![0x11, 0x01]);
    }

    #[test]
    fn test_security_access_request() {
        let level = SecurityLevel::new(0x01);
        let request = UdsRequest::security_access_request_seed(level);
        assert_eq!(request.service(), UdsService::SecurityAccess);
        assert_eq!(request.sub_function(), Some(0x01));

        let encoded = request.encode();
        assert_eq!(encoded, vec![0x27, 0x01]);
    }

    #[test]
    fn test_security_access_send_key() {
        let level = SecurityLevel::new(0x01);
        let key = [0x12, 0x34, 0x56, 0x78];
        let request = UdsRequest::security_access_send_key(level, &key);
        assert_eq!(request.sub_function(), Some(0x02));

        let encoded = request.encode();
        assert_eq!(encoded, vec![0x27, 0x02, 0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_tester_present_request() {
        let request = UdsRequest::tester_present(false);
        let encoded = request.encode();
        assert_eq!(encoded, vec![0x3E, 0x00]);

        let request_suppress = UdsRequest::tester_present(true);
        let encoded_suppress = request_suppress.encode();
        assert_eq!(encoded_suppress, vec![0x3E, 0x80]);
    }

    #[test]
    fn test_read_data_by_identifier_request() {
        let request = UdsRequest::read_data_by_identifier(&[0xF190, 0xF191]);
        let encoded = request.encode();
        assert_eq!(encoded, vec![0x22, 0xF1, 0x90, 0xF1, 0x91]);
    }

    #[test]
    fn test_write_data_by_identifier_request() {
        let request = UdsRequest::write_data_by_identifier(0xF190, &[0x01, 0x02, 0x03]);
        let encoded = request.encode();
        assert_eq!(encoded, vec![0x2E, 0xF1, 0x90, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_clear_diagnostic_information_request() {
        let request = UdsRequest::clear_diagnostic_information(0xFFFFFF);
        let encoded = request.encode();
        assert_eq!(encoded, vec![0x14, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_routine_control_request() {
        let request =
            UdsRequest::routine_control(RoutineControlType::StartRoutine, 0x0203, &[0x01, 0x02]);
        let encoded = request.encode();
        assert_eq!(encoded, vec![0x31, 0x01, 0x02, 0x03, 0x01, 0x02]);
    }

    #[test]
    fn test_parse_positive_response() {
        // Positive response to DiagnosticSessionControl
        let bytes = vec![0x50, 0x03, 0x00, 0x19, 0x01, 0xF4];
        let response = UdsResponse::parse(&bytes).unwrap();

        match response {
            UdsResponse::Positive {
                service,
                sub_function,
                data,
            } => {
                assert_eq!(service, UdsService::DiagnosticSessionControl);
                assert_eq!(sub_function, Some(0x03));
                assert_eq!(data, vec![0x00, 0x19, 0x01, 0xF4]);
            },
            _ => panic!("Expected positive response"),
        }
    }

    #[test]
    fn test_parse_negative_response() {
        // Negative response: service not supported
        let bytes = vec![0x7F, 0x10, 0x11];
        let response = UdsResponse::parse(&bytes).unwrap();

        match response {
            UdsResponse::Negative { service, nrc } => {
                assert_eq!(service, UdsService::DiagnosticSessionControl);
                assert_eq!(nrc, NegativeResponseCode::ServiceNotSupported);
            },
            _ => panic!("Expected negative response"),
        }
    }

    #[test]
    fn test_parse_pending_response() {
        let bytes = vec![0x7F, 0x22, 0x78];
        let response = UdsResponse::parse(&bytes).unwrap();
        assert!(response.is_pending());
    }

    #[test]
    fn test_security_level() {
        let level = SecurityLevel::new(0x01);
        assert_eq!(level.request_seed(), 0x01);
        assert_eq!(level.send_key(), 0x02);
        assert!(level.is_request_seed());
        assert!(!level.is_send_key());

        let level2 = SecurityLevel::new(0x02);
        assert_eq!(level2.request_seed(), 0x01);
        assert_eq!(level2.send_key(), 0x02);
        assert!(!level2.is_request_seed());
        assert!(level2.is_send_key());
    }

    #[test]
    fn test_dtc() {
        let dtc = Dtc::new(0x12, 0x34, 0x56, 0x09);
        assert_eq!(dtc.number(), 0x123456);
        assert!(dtc.status.test_failed());
        assert!(dtc.status.is_confirmed());
        assert!(!dtc.status.is_pending());

        let dtc2 = Dtc::from_number(0xABCDEF, 0x04);
        assert_eq!(dtc2.high, 0xAB);
        assert_eq!(dtc2.mid, 0xCD);
        assert_eq!(dtc2.low, 0xEF);
        assert!(dtc2.status.is_pending());
    }

    #[test]
    fn test_nrc_is_pending() {
        assert!(NegativeResponseCode::RequestCorrectlyReceivedResponsePending.is_pending());
        assert!(!NegativeResponseCode::ServiceNotSupported.is_pending());
    }

    #[test]
    fn test_request_download() {
        let format = AddressAndLengthFormatId::new(4, 4);
        let data_format = DataFormatId::NONE;
        let request = UdsRequest::request_download(0x00010000, 0x1000, format, data_format);

        let encoded = request.encode();
        assert_eq!(encoded[0], 0x34); // Request Download SID
        assert_eq!(encoded[1], 0x00); // Data format (no compression/encryption)
        assert_eq!(encoded[2], 0x44); // Address/length format (4 bytes each)
                                      // Memory address: 0x00010000
        assert_eq!(encoded[3], 0x00);
        assert_eq!(encoded[4], 0x01);
        assert_eq!(encoded[5], 0x00);
        assert_eq!(encoded[6], 0x00);
        // Memory size: 0x1000
        assert_eq!(encoded[7], 0x00);
        assert_eq!(encoded[8], 0x00);
        assert_eq!(encoded[9], 0x10);
        assert_eq!(encoded[10], 0x00);
    }

    #[test]
    fn test_request_upload() {
        let format = AddressAndLengthFormatId::new(2, 2);
        let data_format = DataFormatId::NONE;
        let request = UdsRequest::request_upload(0x1000, 0x0100, format, data_format);

        let encoded = request.encode();
        assert_eq!(encoded[0], 0x35); // Request Upload SID
        assert_eq!(encoded[1], 0x00); // Data format
        assert_eq!(encoded[2], 0x22); // Address/length format (2 bytes each)
        assert_eq!(encoded[3], 0x10); // Address high
        assert_eq!(encoded[4], 0x00); // Address low
        assert_eq!(encoded[5], 0x01); // Size high
        assert_eq!(encoded[6], 0x00); // Size low
    }

    #[test]
    fn test_transfer_data() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let request = UdsRequest::transfer_data(0x01, &data);

        let encoded = request.encode();
        assert_eq!(encoded[0], 0x36); // Transfer Data SID
        assert_eq!(encoded[1], 0x01); // Block sequence counter
        assert_eq!(&encoded[2..], &data);
    }

    #[test]
    fn test_request_transfer_exit() {
        let request = UdsRequest::request_transfer_exit(None);
        let encoded = request.encode();
        assert_eq!(encoded, vec![0x37]);

        let request_with_param = UdsRequest::request_transfer_exit(Some(&[0x01, 0x02]));
        let encoded_with_param = request_with_param.encode();
        assert_eq!(encoded_with_param, vec![0x37, 0x01, 0x02]);
    }

    #[test]
    fn test_transfer_session() {
        let mut session = TransferSession::new(TransferDirection::Download, 0x10000, 1000, 256);

        assert_eq!(session.block_sequence_counter, 1);
        assert_eq!(session.bytes_transferred, 0);
        assert_eq!(session.remaining_bytes(), 1000);
        assert_eq!(session.progress_percent(), 0);
        assert!(!session.is_complete);

        // Transfer some data
        let counter = session.next_block_counter();
        assert_eq!(counter, 1);
        session.record_transfer(256);

        assert_eq!(session.block_sequence_counter, 2);
        assert_eq!(session.bytes_transferred, 256);
        assert_eq!(session.remaining_bytes(), 744);
        assert_eq!(session.progress_percent(), 25);

        // Complete the transfer
        session.record_transfer(744);
        assert!(session.is_complete);
        assert_eq!(session.progress_percent(), 100);
    }

    #[test]
    fn test_address_and_length_format() {
        let format = AddressAndLengthFormatId::new(4, 2);
        assert_eq!(format.encode(), 0x42);

        let decoded = AddressAndLengthFormatId::decode(0x42);
        assert_eq!(decoded.address_bytes, 4);
        assert_eq!(decoded.size_bytes, 2);
    }

    #[test]
    fn test_data_format_id() {
        let format = DataFormatId::new(1, 2);
        assert_eq!(format.encode(), 0x12);

        let decoded = DataFormatId::decode(0x12);
        assert_eq!(decoded.compression_method, 1);
        assert_eq!(decoded.encrypting_method, 2);

        assert_eq!(DataFormatId::NONE.encode(), 0x00);
    }

    #[test]
    fn test_transfer_request_response_parse() {
        // Response with 2-byte max block length
        let data = [0x20, 0x01, 0x00]; // Format: 2 bytes, max block: 256
        let response = TransferRequestResponse::parse(&data).unwrap();
        assert_eq!(response.max_number_of_block_length, 256);
    }

    #[test]
    fn test_block_sequence_counter_wrap() {
        let mut session = TransferSession::new(TransferDirection::Download, 0x0, 10000, 100);

        // Advance to 255
        session.block_sequence_counter = 255;
        let counter = session.next_block_counter();
        assert_eq!(counter, 255);

        // Should wrap to 1, skipping 0
        assert_eq!(session.block_sequence_counter, 1);
    }

    #[test]
    fn test_uds_client_creation() {
        let config = UdsClientConfig::default();
        let client = UdsClient::new(config);

        assert_eq!(client.current_session(), DiagnosticSession::Default);
        assert_eq!(client.security_state(), SecurityAccessState::Locked);
        assert!(client.security_level().is_none());
        assert!(client.transfer_session().is_none());
    }

    #[test]
    fn test_uds_client_session_control() {
        let mut client = UdsClient::new(UdsClientConfig::default());

        // Create session request
        let request = client.start_session(DiagnosticSession::Extended);
        assert_eq!(request.service(), UdsService::DiagnosticSessionControl);

        // Process positive response
        let response = UdsResponse::Positive {
            service: UdsService::DiagnosticSessionControl,
            sub_function: Some(0x03),
            data: vec![0x00, 0x19, 0x01, 0xF4],
        };

        client.process_session_response(&response).unwrap();
        assert_eq!(client.current_session(), DiagnosticSession::Extended);
    }

    #[test]
    fn test_uds_client_security_access() {
        let mut client = UdsClient::new(UdsClientConfig::default());
        let level = SecurityLevel::new(0x01);

        // Request seed
        let request = client.request_seed(level);
        assert_eq!(request.service(), UdsService::SecurityAccess);

        // Process seed response
        let seed_response = UdsResponse::Positive {
            service: UdsService::SecurityAccess,
            sub_function: Some(0x01),
            data: vec![0x12, 0x34, 0x56, 0x78],
        };

        let seed = client.process_seed_response(&seed_response, level).unwrap();
        assert_eq!(seed, vec![0x12, 0x34, 0x56, 0x78]);
        assert_eq!(client.security_state(), SecurityAccessState::SeedRequested);

        // Send key
        let key = [0xAB, 0xCD, 0xEF, 0x01];
        let key_request = client.send_key(level, &key);
        assert_eq!(key_request.service(), UdsService::SecurityAccess);

        // Process key response
        let key_response = UdsResponse::Positive {
            service: UdsService::SecurityAccess,
            sub_function: Some(0x02),
            data: vec![],
        };

        client.process_key_response(&key_response, level).unwrap();
        assert_eq!(client.security_state(), SecurityAccessState::Unlocked);
        assert_eq!(client.security_level(), Some(level));
    }

    #[test]
    fn test_uds_client_negative_response() {
        let mut client = UdsClient::new(UdsClientConfig::default());

        let response = UdsResponse::Negative {
            service: UdsService::DiagnosticSessionControl,
            nrc: NegativeResponseCode::ConditionsNotCorrect,
        };

        let result = client.process_session_response(&response);
        assert!(matches!(
            result,
            Err(UdsClientError::NegativeResponse(
                NegativeResponseCode::ConditionsNotCorrect
            ))
        ));
    }

    #[test]
    fn test_uds_client_response_pending() {
        let mut client = UdsClient::new(UdsClientConfig::default());

        let pending_response = UdsResponse::Negative {
            service: UdsService::ReadDataByIdentifier,
            nrc: NegativeResponseCode::RequestCorrectlyReceivedResponsePending,
        };

        // First pending should return true (continue waiting)
        assert!(client.handle_response_pending(&pending_response));
        assert_eq!(
            client.current_timeout_ms(),
            client.config().p2_star_timeout_ms
        );

        // Non-pending response should reset counter
        let positive_response = UdsResponse::Positive {
            service: UdsService::ReadDataByIdentifier,
            sub_function: None,
            data: vec![0x01, 0x02],
        };
        assert!(!client.handle_response_pending(&positive_response));
        assert_eq!(client.current_timeout_ms(), client.config().p2_timeout_ms);
    }

    #[test]
    fn test_uds_client_transfer_session() {
        let mut client = UdsClient::new(UdsClientConfig::default());

        // Start download
        let format = AddressAndLengthFormatId::new(4, 4);
        let request = client.start_download(0x10000, 1000, format, DataFormatId::NONE);
        assert_eq!(request.service(), UdsService::RequestDownload);

        // Process response
        let response = UdsResponse::Positive {
            service: UdsService::RequestDownload,
            sub_function: None,
            data: vec![0x20, 0x01, 0x00], // Max block length = 256
        };

        let max_block = client
            .process_transfer_request_response(
                &response,
                TransferDirection::Download,
                0x10000,
                1000,
            )
            .unwrap();

        assert_eq!(max_block, 256);
        assert!(client.transfer_session().is_some());

        // Transfer data
        let data = vec![0x01; 100];
        let transfer_request = client.next_transfer_data(&data).unwrap();
        assert_eq!(transfer_request.service(), UdsService::TransferData);

        // Process transfer response
        let transfer_response = UdsResponse::Positive {
            service: UdsService::TransferData,
            sub_function: None,
            data: vec![0x01], // Block counter echo
        };

        client
            .process_transfer_data_response(&transfer_response, 100)
            .unwrap();
        assert_eq!(client.transfer_session().unwrap().bytes_transferred, 100);

        // Exit transfer
        let exit_request = client.request_transfer_exit();
        assert_eq!(exit_request.service(), UdsService::RequestTransferExit);

        let exit_response = UdsResponse::Positive {
            service: UdsService::RequestTransferExit,
            sub_function: None,
            data: vec![],
        };

        client
            .process_transfer_exit_response(&exit_response)
            .unwrap();
        assert!(client.transfer_session().is_none());
    }

    #[test]
    fn test_uds_client_reset() {
        let mut client = UdsClient::new(UdsClientConfig::default());

        // Set up some state
        client.current_session = DiagnosticSession::Extended;
        client.security_state = SecurityAccessState::Unlocked;
        client.security_level = Some(SecurityLevel::new(0x01));

        // Reset
        client.reset();

        assert_eq!(client.current_session(), DiagnosticSession::Default);
        assert_eq!(client.security_state(), SecurityAccessState::Locked);
        assert!(client.security_level().is_none());
        assert!(client.transfer_session().is_none());
    }

    #[test]
    fn test_uds_client_error_display() {
        let error = UdsClientError::NegativeResponse(NegativeResponseCode::SecurityAccessDenied);
        assert!(error.to_string().contains("SecurityAccessDenied"));

        let error2 = UdsClientError::UnknownNrc(0xAB);
        assert!(error2.to_string().contains("0xAB"));
    }
}
