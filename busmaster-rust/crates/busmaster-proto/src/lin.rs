//! LIN (Local Interconnect Network) Protocol Implementation
//!
//! LIN is a low-cost serial communication protocol used in automotive
//! applications for communication between sensors, actuators, and ECUs.
//!
//! # Protocol Overview
//!
//! LIN uses a single-master, multiple-slave architecture with:
//! - Master node controls bus timing via schedule tables
//! - Slave nodes respond to master requests
//! - Frame IDs 0-59 for signal-carrying frames
//! - Frame IDs 60-61 for diagnostic frames
//! - Frame IDs 62-63 reserved
//!
//! # Example
//!
//! ```
//! use busmaster_proto::lin::{LinFrame, LinId, LinChecksum};
//!
//! // Create a LIN frame
//! let frame = LinFrame::new(LinId::new(0x10).unwrap(), &[0x01, 0x02, 0x03, 0x04]);
//! assert_eq!(frame.id().raw(), 0x10);
//! assert_eq!(frame.data().len(), 4);
//! ```

use busmaster_core::{BusmasterError, Result};
use serde::{Deserialize, Serialize};

/// Maximum LIN frame data length
pub const LIN_MAX_DATA_LEN: usize = 8;

/// LIN diagnostic master request ID
pub const LIN_DIAG_MASTER_REQUEST: u8 = 0x3C;

/// LIN diagnostic slave response ID
pub const LIN_DIAG_SLAVE_RESPONSE: u8 = 0x3D;

/// LIN frame ID (0-63)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinId(u8);

impl LinId {
    /// Create a new LIN ID
    ///
    /// # Arguments
    /// * `id` - Frame ID (0-63)
    ///
    /// # Returns
    /// * `Some(LinId)` if valid
    /// * `None` if ID > 63
    #[must_use]
    pub fn new(id: u8) -> Option<Self> {
        if id <= 63 {
            Some(Self(id))
        } else {
            None
        }
    }

    /// Create a LIN ID without validation (unsafe)
    ///
    /// # Safety
    /// Caller must ensure id <= 63
    #[must_use]
    pub const fn new_unchecked(id: u8) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    #[must_use]
    pub const fn raw(&self) -> u8 {
        self.0
    }

    /// Calculate the protected ID (PID) with parity bits
    #[must_use]
    pub fn protected_id(&self) -> u8 {
        let id = self.0;
        let p0 = (id ^ (id >> 1) ^ (id >> 2) ^ (id >> 4)) & 0x01;
        let p1 = !((id >> 1) ^ (id >> 3) ^ (id >> 4) ^ (id >> 5)) & 0x01;
        id | (p0 << 6) | (p1 << 7)
    }

    /// Create from protected ID (validates parity)
    pub fn from_protected_id(pid: u8) -> Result<Self> {
        let id = pid & 0x3F;
        let lin_id = Self(id);
        if lin_id.protected_id() == pid {
            Ok(lin_id)
        } else {
            Err(BusmasterError::Protocol {
                message: format!("Invalid LIN PID parity: 0x{:02X}", pid),
            })
        }
    }

    /// Check if this is a diagnostic frame ID
    #[must_use]
    pub fn is_diagnostic(&self) -> bool {
        self.0 == LIN_DIAG_MASTER_REQUEST || self.0 == LIN_DIAG_SLAVE_RESPONSE
    }

    /// Check if this is a reserved frame ID (62-63)
    #[must_use]
    pub fn is_reserved(&self) -> bool {
        self.0 >= 62
    }

    /// Check if this is a signal-carrying frame ID (0-59)
    #[must_use]
    pub fn is_signal_frame(&self) -> bool {
        self.0 <= 59
    }
}

/// LIN checksum type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinChecksum {
    /// Classic checksum (LIN 1.x) - data bytes only
    Classic,
    /// Enhanced checksum (LIN 2.x) - PID + data bytes
    Enhanced,
}

impl LinChecksum {
    /// Calculate checksum for given data
    #[must_use]
    pub fn calculate(&self, pid: u8, data: &[u8]) -> u8 {
        let mut sum: u16 = match self {
            Self::Classic => 0,
            Self::Enhanced => pid as u16,
        };

        for &byte in data {
            sum += byte as u16;
            if sum > 255 {
                sum = (sum & 0xFF) + 1;
            }
        }

        !sum as u8
    }

    /// Verify checksum
    #[must_use]
    pub fn verify(&self, pid: u8, data: &[u8], checksum: u8) -> bool {
        self.calculate(pid, data) == checksum
    }
}

/// LIN frame direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinDirection {
    /// Master to slave (publish)
    MasterToSlave,
    /// Slave to master (subscribe)
    SlaveToMaster,
}

/// LIN frame
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinFrame {
    /// Frame ID
    id: LinId,
    /// Data bytes (up to 8)
    data: Vec<u8>,
    /// Checksum type
    checksum_type: LinChecksum,
    /// Timestamp in microseconds
    timestamp: u64,
    /// Frame direction
    direction: LinDirection,
}

impl LinFrame {
    /// Create a new LIN frame
    #[must_use]
    pub fn new(id: LinId, data: &[u8]) -> Self {
        let data = if data.len() > LIN_MAX_DATA_LEN {
            data[..LIN_MAX_DATA_LEN].to_vec()
        } else {
            data.to_vec()
        };

        Self {
            id,
            data,
            checksum_type: LinChecksum::Enhanced,
            timestamp: 0,
            direction: LinDirection::MasterToSlave,
        }
    }

    /// Create a diagnostic master request frame
    #[must_use]
    pub fn diagnostic_request(nad: u8, pci: u8, sid: u8, data: &[u8]) -> Self {
        let mut frame_data = vec![nad, pci, sid];
        frame_data.extend_from_slice(data);
        // Pad to 8 bytes
        while frame_data.len() < 8 {
            frame_data.push(0xFF);
        }
        Self::new(LinId::new_unchecked(LIN_DIAG_MASTER_REQUEST), &frame_data)
    }

    /// Create a diagnostic slave response frame
    #[must_use]
    pub fn diagnostic_response(nad: u8, pci: u8, rsid: u8, data: &[u8]) -> Self {
        let mut frame_data = vec![nad, pci, rsid];
        frame_data.extend_from_slice(data);
        // Pad to 8 bytes
        while frame_data.len() < 8 {
            frame_data.push(0xFF);
        }
        let mut frame = Self::new(LinId::new_unchecked(LIN_DIAG_SLAVE_RESPONSE), &frame_data);
        frame.direction = LinDirection::SlaveToMaster;
        frame
    }

    /// Get the frame ID
    #[must_use]
    pub fn id(&self) -> LinId {
        self.id
    }

    /// Get the data bytes
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the data length
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if frame has no data
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the checksum type
    #[must_use]
    pub fn checksum_type(&self) -> LinChecksum {
        self.checksum_type
    }

    /// Set the checksum type
    pub fn set_checksum_type(&mut self, checksum_type: LinChecksum) {
        self.checksum_type = checksum_type;
    }

    /// Get the timestamp
    #[must_use]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Set the timestamp
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    /// Get the direction
    #[must_use]
    pub fn direction(&self) -> LinDirection {
        self.direction
    }

    /// Set the direction
    pub fn set_direction(&mut self, direction: LinDirection) {
        self.direction = direction;
    }

    /// Calculate the checksum for this frame
    #[must_use]
    pub fn calculate_checksum(&self) -> u8 {
        self.checksum_type.calculate(self.id.protected_id(), &self.data)
    }

    /// Encode frame to bytes (PID + data + checksum)
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + self.data.len());
        bytes.push(self.id.protected_id());
        bytes.extend_from_slice(&self.data);
        bytes.push(self.calculate_checksum());
        bytes
    }

    /// Parse frame from bytes (PID + data + checksum)
    pub fn from_bytes(bytes: &[u8], checksum_type: LinChecksum) -> Result<Self> {
        if bytes.len() < 2 {
            return Err(BusmasterError::Parse {
                message: "LIN frame too short".into(),
            });
        }

        let pid = bytes[0];
        let id = LinId::from_protected_id(pid)?;
        let data = &bytes[1..bytes.len() - 1];
        let checksum = bytes[bytes.len() - 1];

        if !checksum_type.verify(pid, data, checksum) {
            return Err(BusmasterError::Protocol {
                message: format!(
                    "LIN checksum mismatch: expected 0x{:02X}, got 0x{:02X}",
                    checksum_type.calculate(pid, data),
                    checksum
                ),
            });
        }

        Ok(Self {
            id,
            data: data.to_vec(),
            checksum_type,
            timestamp: 0,
            direction: LinDirection::MasterToSlave,
        })
    }
}


// ============================================================================
// Schedule Table Support
// ============================================================================

/// Schedule table entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleEntryType {
    /// Unconditional frame
    Unconditional,
    /// Event-triggered frame
    EventTriggered,
    /// Sporadic frame
    Sporadic,
    /// Diagnostic master request
    DiagnosticMasterRequest,
    /// Diagnostic slave response
    DiagnosticSlaveResponse,
    /// Assign NAD
    AssignNad,
    /// Conditional change NAD
    ConditionalChangeNad,
    /// Data dump
    DataDump,
    /// Save configuration
    SaveConfiguration,
    /// Assign frame ID range
    AssignFrameIdRange,
    /// Free format
    FreeFormat,
}

/// Schedule table entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleEntry {
    /// Entry type
    pub entry_type: ScheduleEntryType,
    /// Frame ID (for unconditional/event-triggered/sporadic)
    pub frame_id: Option<LinId>,
    /// Frame name
    pub frame_name: String,
    /// Delay in milliseconds after this entry
    pub delay_ms: u16,
}

impl ScheduleEntry {
    /// Create an unconditional frame entry
    #[must_use]
    pub fn unconditional(frame_id: LinId, frame_name: &str, delay_ms: u16) -> Self {
        Self {
            entry_type: ScheduleEntryType::Unconditional,
            frame_id: Some(frame_id),
            frame_name: frame_name.to_string(),
            delay_ms,
        }
    }

    /// Create a diagnostic master request entry
    #[must_use]
    pub fn diagnostic_master_request(delay_ms: u16) -> Self {
        Self {
            entry_type: ScheduleEntryType::DiagnosticMasterRequest,
            frame_id: Some(LinId::new_unchecked(LIN_DIAG_MASTER_REQUEST)),
            frame_name: "MasterReq".to_string(),
            delay_ms,
        }
    }

    /// Create a diagnostic slave response entry
    #[must_use]
    pub fn diagnostic_slave_response(delay_ms: u16) -> Self {
        Self {
            entry_type: ScheduleEntryType::DiagnosticSlaveResponse,
            frame_id: Some(LinId::new_unchecked(LIN_DIAG_SLAVE_RESPONSE)),
            frame_name: "SlaveResp".to_string(),
            delay_ms,
        }
    }
}

/// Schedule table
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleTable {
    /// Table name
    pub name: String,
    /// Entries in the schedule
    pub entries: Vec<ScheduleEntry>,
}

impl ScheduleTable {
    /// Create a new schedule table
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Vec::new(),
        }
    }

    /// Add an entry to the schedule
    pub fn add_entry(&mut self, entry: ScheduleEntry) {
        self.entries.push(entry);
    }

    /// Get the total cycle time in milliseconds
    #[must_use]
    pub fn cycle_time_ms(&self) -> u32 {
        self.entries.iter().map(|e| e.delay_ms as u32).sum()
    }

    /// Get the number of entries
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the schedule is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ============================================================================
// LIN Node Configuration
// ============================================================================

/// LIN node type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinNodeType {
    /// Master node
    Master,
    /// Slave node
    Slave,
}

/// LIN node address (NAD)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinNad(u8);

impl LinNad {
    /// Broadcast NAD
    pub const BROADCAST: Self = Self(0x7F);
    /// Functional NAD (for diagnostics)
    pub const FUNCTIONAL: Self = Self(0x7E);
    /// Sleep NAD
    pub const SLEEP: Self = Self(0x00);

    /// Create a new NAD
    #[must_use]
    pub fn new(nad: u8) -> Self {
        Self(nad)
    }

    /// Get the raw NAD value
    #[must_use]
    pub const fn raw(&self) -> u8 {
        self.0
    }

    /// Check if this is a valid configured NAD (1-125)
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.0 >= 1 && self.0 <= 125
    }
}

/// LIN node configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinNode {
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: LinNodeType,
    /// Node address (NAD) for slaves
    pub nad: Option<LinNad>,
    /// Supplier ID
    pub supplier_id: Option<u16>,
    /// Function ID
    pub function_id: Option<u16>,
    /// Variant ID
    pub variant_id: Option<u8>,
    /// Published frame IDs
    pub publishes: Vec<LinId>,
    /// Subscribed frame IDs
    pub subscribes: Vec<LinId>,
}

impl LinNode {
    /// Create a new master node
    #[must_use]
    pub fn master(name: &str) -> Self {
        Self {
            name: name.to_string(),
            node_type: LinNodeType::Master,
            nad: None,
            supplier_id: None,
            function_id: None,
            variant_id: None,
            publishes: Vec::new(),
            subscribes: Vec::new(),
        }
    }

    /// Create a new slave node
    #[must_use]
    pub fn slave(name: &str, nad: LinNad) -> Self {
        Self {
            name: name.to_string(),
            node_type: LinNodeType::Slave,
            nad: Some(nad),
            supplier_id: None,
            function_id: None,
            variant_id: None,
            publishes: Vec::new(),
            subscribes: Vec::new(),
        }
    }

    /// Add a published frame
    pub fn add_publish(&mut self, id: LinId) {
        if !self.publishes.contains(&id) {
            self.publishes.push(id);
        }
    }

    /// Add a subscribed frame
    pub fn add_subscribe(&mut self, id: LinId) {
        if !self.subscribes.contains(&id) {
            self.subscribes.push(id);
        }
    }
}

// ============================================================================
// LIN Signal Definition
// ============================================================================

/// LIN signal definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinSignal {
    /// Signal name
    pub name: String,
    /// Size in bits
    pub size: u8,
    /// Initial value
    pub init_value: u64,
    /// Publisher node name
    pub publisher: String,
    /// Subscriber node names
    pub subscribers: Vec<String>,
}

impl LinSignal {
    /// Create a new LIN signal
    #[must_use]
    pub fn new(name: &str, size: u8, publisher: &str) -> Self {
        Self {
            name: name.to_string(),
            size,
            init_value: 0,
            publisher: publisher.to_string(),
            subscribers: Vec::new(),
        }
    }

    /// Set the initial value
    pub fn with_init_value(mut self, value: u64) -> Self {
        self.init_value = value;
        self
    }

    /// Add a subscriber
    pub fn add_subscriber(&mut self, subscriber: &str) {
        if !self.subscribers.contains(&subscriber.to_string()) {
            self.subscribers.push(subscriber.to_string());
        }
    }
}

// ============================================================================
// LIN Frame Definition (for LDF)
// ============================================================================

/// LIN frame definition (from LDF)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinFrameDef {
    /// Frame name
    pub name: String,
    /// Frame ID
    pub id: LinId,
    /// Publisher node name
    pub publisher: String,
    /// Frame length in bytes
    pub length: u8,
    /// Signals in this frame (signal name, start bit)
    pub signals: Vec<(String, u8)>,
}

impl LinFrameDef {
    /// Create a new frame definition
    #[must_use]
    pub fn new(name: &str, id: LinId, publisher: &str, length: u8) -> Self {
        Self {
            name: name.to_string(),
            id,
            publisher: publisher.to_string(),
            length,
            signals: Vec::new(),
        }
    }

    /// Add a signal to the frame
    pub fn add_signal(&mut self, signal_name: &str, start_bit: u8) {
        self.signals.push((signal_name.to_string(), start_bit));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lin_id_creation() {
        assert!(LinId::new(0).is_some());
        assert!(LinId::new(63).is_some());
        assert!(LinId::new(64).is_none());
    }

    #[test]
    fn test_lin_id_protected_id() {
        // Test known PID values
        let id0 = LinId::new(0x00).unwrap();
        assert_eq!(id0.protected_id(), 0x80);

        let id1 = LinId::new(0x01).unwrap();
        assert_eq!(id1.protected_id(), 0xC1);

        let id3c = LinId::new(0x3C).unwrap();
        assert_eq!(id3c.protected_id(), 0x3C); // Diagnostic master request
    }

    #[test]
    fn test_lin_id_from_protected_id() {
        let pid = 0xC1; // ID 0x01 with parity
        let id = LinId::from_protected_id(pid).unwrap();
        assert_eq!(id.raw(), 0x01);

        // Invalid parity should fail
        assert!(LinId::from_protected_id(0x41).is_err());
    }

    #[test]
    fn test_lin_id_classification() {
        let signal_id = LinId::new(0x10).unwrap();
        assert!(signal_id.is_signal_frame());
        assert!(!signal_id.is_diagnostic());
        assert!(!signal_id.is_reserved());

        let diag_master = LinId::new(LIN_DIAG_MASTER_REQUEST).unwrap();
        assert!(diag_master.is_diagnostic());
        assert!(!diag_master.is_signal_frame());

        let reserved = LinId::new(62).unwrap();
        assert!(reserved.is_reserved());
    }

    #[test]
    fn test_lin_checksum_classic() {
        let checksum = LinChecksum::Classic;
        let data = [0x01, 0x02, 0x03, 0x04];
        let cs = checksum.calculate(0x00, &data);
        assert!(checksum.verify(0x00, &data, cs));
    }

    #[test]
    fn test_lin_checksum_enhanced() {
        let checksum = LinChecksum::Enhanced;
        let pid = 0xC1; // ID 0x01 with parity
        let data = [0x01, 0x02, 0x03, 0x04];
        let cs = checksum.calculate(pid, &data);
        assert!(checksum.verify(pid, &data, cs));
    }

    #[test]
    fn test_lin_frame_creation() {
        let id = LinId::new(0x10).unwrap();
        let frame = LinFrame::new(id, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(frame.id().raw(), 0x10);
        assert_eq!(frame.data(), &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(frame.len(), 4);
    }

    #[test]
    fn test_lin_frame_truncation() {
        let id = LinId::new(0x10).unwrap();
        let long_data = [0u8; 16];
        let frame = LinFrame::new(id, &long_data);
        assert_eq!(frame.len(), LIN_MAX_DATA_LEN);
    }

    #[test]
    fn test_lin_frame_roundtrip() {
        let id = LinId::new(0x10).unwrap();
        let frame = LinFrame::new(id, &[0x01, 0x02, 0x03, 0x04]);
        let bytes = frame.to_bytes();

        let parsed = LinFrame::from_bytes(&bytes, LinChecksum::Enhanced).unwrap();
        assert_eq!(parsed.id().raw(), frame.id().raw());
        assert_eq!(parsed.data(), frame.data());
    }

    #[test]
    fn test_lin_diagnostic_frames() {
        let request = LinFrame::diagnostic_request(0x01, 0x06, 0x22, &[0xF1, 0x90]);
        assert_eq!(request.id().raw(), LIN_DIAG_MASTER_REQUEST);
        assert_eq!(request.len(), 8);
        assert_eq!(request.data()[0], 0x01); // NAD
        assert_eq!(request.data()[1], 0x06); // PCI
        assert_eq!(request.data()[2], 0x22); // SID

        let response = LinFrame::diagnostic_response(0x01, 0x06, 0x62, &[0xF1, 0x90]);
        assert_eq!(response.id().raw(), LIN_DIAG_SLAVE_RESPONSE);
        assert_eq!(response.direction(), LinDirection::SlaveToMaster);
    }

    #[test]
    fn test_schedule_table() {
        let mut schedule = ScheduleTable::new("MainSchedule");
        schedule.add_entry(ScheduleEntry::unconditional(
            LinId::new(0x10).unwrap(),
            "MotorControl",
            10,
        ));
        schedule.add_entry(ScheduleEntry::unconditional(
            LinId::new(0x11).unwrap(),
            "SensorData",
            15,
        ));
        schedule.add_entry(ScheduleEntry::diagnostic_master_request(5));

        assert_eq!(schedule.len(), 3);
        assert_eq!(schedule.cycle_time_ms(), 30);
    }

    #[test]
    fn test_lin_node() {
        let mut master = LinNode::master("Master");
        master.add_publish(LinId::new(0x10).unwrap());
        assert_eq!(master.node_type, LinNodeType::Master);
        assert!(master.nad.is_none());

        let mut slave = LinNode::slave("Slave1", LinNad::new(0x01));
        slave.add_subscribe(LinId::new(0x10).unwrap());
        slave.add_publish(LinId::new(0x11).unwrap());
        assert_eq!(slave.node_type, LinNodeType::Slave);
        assert_eq!(slave.nad.unwrap().raw(), 0x01);
    }

    #[test]
    fn test_lin_nad() {
        assert!(LinNad::new(1).is_configured());
        assert!(LinNad::new(125).is_configured());
        assert!(!LinNad::new(0).is_configured());
        assert!(!LinNad::new(126).is_configured());
        assert!(!LinNad::BROADCAST.is_configured());
        assert!(!LinNad::FUNCTIONAL.is_configured());
    }

    #[test]
    fn test_lin_signal() {
        let mut signal = LinSignal::new("MotorSpeed", 16, "Motor")
            .with_init_value(0);
        signal.add_subscriber("ECU1");
        signal.add_subscriber("ECU2");

        assert_eq!(signal.name, "MotorSpeed");
        assert_eq!(signal.size, 16);
        assert_eq!(signal.subscribers.len(), 2);
    }

    #[test]
    fn test_lin_frame_def() {
        let mut frame_def = LinFrameDef::new(
            "MotorControl",
            LinId::new(0x10).unwrap(),
            "Master",
            4,
        );
        frame_def.add_signal("MotorSpeed", 0);
        frame_def.add_signal("MotorDirection", 16);

        assert_eq!(frame_def.signals.len(), 2);
        assert_eq!(frame_def.signals[0], ("MotorSpeed".to_string(), 0));
    }
}
