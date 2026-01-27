//! ECU Calibration and Measurement Module
//!
//! This module provides comprehensive calibration and measurement functionality
//! similar to Vector CANape and ETAS INCA. It enables:
//!
//! - Real-time ECU measurement via XCP/CCP DAQ
//! - Online calibration (parameter modification at runtime)
//! - Offline calibration (dataset management)
//! - Memory page switching (working/reference pages)
//! - Flash programming
//! - Calibration data management (CDF files)
//! - Seed & Key security
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    CalibrationSession                           │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
//! │  │ MeasurementMgr│  │ CalibrationMgr│  │ FlashProgrammer     │  │
//! │  │  - DAQ Lists  │  │  - Parameters │  │  - Erase/Program    │  │
//! │  │  - Polling    │  │  - Pages      │  │  - Verify           │  │
//! │  │  - Recording  │  │  - Datasets   │  │  - Checksum         │  │
//! │  └──────────────┘  └──────────────┘  └──────────────────────┘  │
//! │                           │                                     │
//! │                    ┌──────┴──────┐                              │
//! │                    │  XcpClient  │                              │
//! │                    └──────┬──────┘                              │
//! │                           │                                     │
//! │                    ┌──────┴──────┐                              │
//! │                    │ Transport   │ (CAN, Ethernet, etc.)        │
//! │                    └─────────────┘                              │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use busmaster_proto::calibration::{CalibrationSession, MeasurementConfig};
//!
//! // Create session with A2L file
//! let mut session = CalibrationSession::new();
//! session.load_a2l("ecu.a2l")?;
//!
//! // Connect to ECU
//! session.connect()?;
//!
//! // Start measurement
//! let config = MeasurementConfig::new()
//!     .add_signal("EngineSpeed", 10) // 10ms rate
//!     .add_signal("EngineTemp", 100); // 100ms rate
//! session.start_measurement(config)?;
//!
//! // Read calibration parameter
//! let fuel_map = session.read_characteristic("FuelMap")?;
//!
//! // Modify parameter online
//! session.write_characteristic("IdleSpeed", 850.0)?;
//!
//! // Save calibration data
//! session.save_dataset("calibration_v1.cdfx")?;
//! ```

use crate::xcp::{DaqList, DaqListMode, Odt, OdtEntry};
use busmaster_core::{BusmasterError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};


// ============================================================================
// Measurement Types
// ============================================================================

/// Measurement signal configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementSignal {
    /// Signal name (from A2L)
    pub name: String,
    /// ECU memory address
    pub address: u64,
    /// Address extension
    pub address_extension: u8,
    /// Data size in bytes
    pub size: u8,
    /// Sampling rate in milliseconds
    pub rate_ms: u16,
    /// Event channel (for DAQ)
    pub event_channel: u16,
    /// Current raw value
    pub raw_value: Vec<u8>,
    /// Current physical value
    pub physical_value: f64,
    /// Timestamp of last update (not serialized)
    #[serde(skip)]
    pub timestamp: Option<Instant>,
    /// Conversion factor
    pub factor: f64,
    /// Conversion offset
    pub offset: f64,
    /// Unit string
    pub unit: String,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

impl MeasurementSignal {
    /// Create a new measurement signal
    #[must_use]
    pub fn new(name: &str, address: u64, size: u8) -> Self {
        Self {
            name: name.to_string(),
            address,
            address_extension: 0,
            size,
            rate_ms: 100,
            event_channel: 0,
            raw_value: vec![0; size as usize],
            physical_value: 0.0,
            timestamp: None,
            factor: 1.0,
            offset: 0.0,
            unit: String::new(),
            min: f64::MIN,
            max: f64::MAX,
        }
    }

    /// Set sampling rate
    #[must_use]
    pub fn with_rate(mut self, rate_ms: u16) -> Self {
        self.rate_ms = rate_ms;
        self
    }

    /// Set conversion parameters
    #[must_use]
    pub fn with_conversion(mut self, factor: f64, offset: f64) -> Self {
        self.factor = factor;
        self.offset = offset;
        self
    }

    /// Set unit
    #[must_use]
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// Update raw value and calculate physical value
    pub fn update(&mut self, raw: &[u8]) {
        self.raw_value = raw.to_vec();
        self.timestamp = Some(Instant::now());

        // Convert raw to physical based on size
        #[allow(clippy::cast_precision_loss)]
        let raw_int: i64 = match self.size {
            1 => raw.first().copied().unwrap_or(0) as i64,
            2 => {
                if raw.len() >= 2 {
                    i16::from_le_bytes([raw[0], raw[1]]) as i64
                } else {
                    0
                }
            }
            4 => {
                if raw.len() >= 4 {
                    i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as i64
                } else {
                    0
                }
            }
            _ => 0,
        };

        #[allow(clippy::cast_precision_loss)]
        {
            self.physical_value = (raw_int as f64) * self.factor + self.offset;
        }
    }
}

/// Measurement configuration for DAQ setup
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasurementConfig {
    /// Signals to measure
    pub signals: Vec<MeasurementSignal>,
    /// Use DAQ (true) or polling (false)
    pub use_daq: bool,
    /// Enable timestamps in DAQ
    pub timestamps_enabled: bool,
    /// Prescaler for DAQ
    pub prescaler: u8,
}

impl MeasurementConfig {
    /// Create new measurement configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
            use_daq: true,
            timestamps_enabled: true,
            prescaler: 1,
        }
    }

    /// Add a signal to measure
    #[must_use]
    pub fn add_signal(mut self, signal: MeasurementSignal) -> Self {
        self.signals.push(signal);
        self
    }

    /// Set DAQ mode
    #[must_use]
    pub fn with_daq(mut self, use_daq: bool) -> Self {
        self.use_daq = use_daq;
        self
    }

    /// Group signals by event channel for DAQ list creation
    #[must_use]
    pub fn group_by_event_channel(&self) -> HashMap<u16, Vec<&MeasurementSignal>> {
        let mut groups: HashMap<u16, Vec<&MeasurementSignal>> = HashMap::new();
        for signal in &self.signals {
            groups.entry(signal.event_channel).or_default().push(signal);
        }
        groups
    }
}

/// Measurement recording session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasurementRecording {
    /// Recording name
    pub name: String,
    /// Start time
    #[serde(skip)]
    pub start_time: Option<Instant>,
    /// Recorded samples: signal_name -> [(timestamp_ms, value)]
    pub samples: HashMap<String, Vec<(u64, f64)>>,
    /// Recording duration limit (None = unlimited)
    pub duration_limit: Option<Duration>,
    /// Sample count limit per signal (None = unlimited)
    pub sample_limit: Option<usize>,
    /// Is recording active
    pub is_recording: bool,
}

impl MeasurementRecording {
    /// Create new recording
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start_time: None,
            samples: HashMap::new(),
            duration_limit: None,
            sample_limit: None,
            is_recording: false,
        }
    }

    /// Start recording
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.is_recording = true;
        self.samples.clear();
    }

    /// Stop recording
    pub fn stop(&mut self) {
        self.is_recording = false;
    }

    /// Add a sample
    pub fn add_sample(&mut self, signal_name: &str, value: f64) {
        if !self.is_recording {
            return;
        }

        let timestamp_ms = self
            .start_time
            .map_or(0, |t| t.elapsed().as_millis() as u64);

        let samples = self.samples.entry(signal_name.to_string()).or_default();

        // Check sample limit
        if let Some(limit) = self.sample_limit {
            if samples.len() >= limit {
                return;
            }
        }

        samples.push((timestamp_ms, value));

        // Check duration limit
        if let Some(limit) = self.duration_limit {
            if let Some(start) = self.start_time {
                if start.elapsed() >= limit {
                    self.stop();
                }
            }
        }
    }

    /// Get sample count for a signal
    #[must_use]
    pub fn sample_count(&self, signal_name: &str) -> usize {
        self.samples.get(signal_name).map_or(0, Vec::len)
    }

    /// Get total sample count
    #[must_use]
    pub fn total_samples(&self) -> usize {
        self.samples.values().map(Vec::len).sum()
    }

    /// Get recording duration
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |t| t.elapsed())
    }
}

// ============================================================================
// Calibration Types
// ============================================================================

/// Calibration parameter (tunable value from A2L CHARACTERISTIC)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationParameter {
    /// Parameter name (from A2L)
    pub name: String,
    /// Description
    pub description: String,
    /// ECU memory address
    pub address: u64,
    /// Address extension
    pub address_extension: u8,
    /// Parameter type
    pub param_type: CalibrationParamType,
    /// Data type
    pub data_type: CalibrationDataType,
    /// Current value(s)
    pub values: Vec<f64>,
    /// Original/reference values
    pub reference_values: Vec<f64>,
    /// Minimum limit
    pub min: f64,
    /// Maximum limit
    pub max: f64,
    /// Conversion factor
    pub factor: f64,
    /// Conversion offset
    pub offset: f64,
    /// Unit string
    pub unit: String,
    /// Is modified (dirty)
    pub is_modified: bool,
    /// X-axis definition (for curves/maps)
    pub x_axis: Option<AxisDefinition>,
    /// Y-axis definition (for maps)
    pub y_axis: Option<AxisDefinition>,
}

impl CalibrationParameter {
    /// Create a new scalar parameter
    #[must_use]
    pub fn new_scalar(name: &str, address: u64, data_type: CalibrationDataType) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            address,
            address_extension: 0,
            param_type: CalibrationParamType::Scalar,
            data_type,
            values: vec![0.0],
            reference_values: vec![0.0],
            min: f64::MIN,
            max: f64::MAX,
            factor: 1.0,
            offset: 0.0,
            unit: String::new(),
            is_modified: false,
            x_axis: None,
            y_axis: None,
        }
    }

    /// Create a new curve parameter
    #[must_use]
    pub fn new_curve(name: &str, address: u64, x_axis: AxisDefinition) -> Self {
        let size = x_axis.size as usize;
        Self {
            name: name.to_string(),
            description: String::new(),
            address,
            address_extension: 0,
            param_type: CalibrationParamType::Curve,
            data_type: CalibrationDataType::Float32,
            values: vec![0.0; size],
            reference_values: vec![0.0; size],
            min: f64::MIN,
            max: f64::MAX,
            factor: 1.0,
            offset: 0.0,
            unit: String::new(),
            is_modified: false,
            x_axis: Some(x_axis),
            y_axis: None,
        }
    }

    /// Create a new map parameter
    #[must_use]
    pub fn new_map(name: &str, address: u64, x_axis: AxisDefinition, y_axis: AxisDefinition) -> Self {
        let size = (x_axis.size as usize) * (y_axis.size as usize);
        Self {
            name: name.to_string(),
            description: String::new(),
            address,
            address_extension: 0,
            param_type: CalibrationParamType::Map,
            data_type: CalibrationDataType::Float32,
            values: vec![0.0; size],
            reference_values: vec![0.0; size],
            min: f64::MIN,
            max: f64::MAX,
            factor: 1.0,
            offset: 0.0,
            unit: String::new(),
            is_modified: false,
            x_axis: Some(x_axis),
            y_axis: Some(y_axis),
        }
    }

    /// Set value at index
    pub fn set_value(&mut self, index: usize, value: f64) -> Result<()> {
        if index >= self.values.len() {
            return Err(BusmasterError::protocol(format!(
                "Index {} out of bounds for parameter {}",
                index, self.name
            )));
        }
        if value < self.min || value > self.max {
            return Err(BusmasterError::protocol(format!(
                "Value {} out of range [{}, {}]",
                value, self.min, self.max
            )));
        }
        self.values[index] = value;
        self.is_modified = true;
        Ok(())
    }

    /// Check if parameter has been modified
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.is_modified
    }

    /// Reset to reference values
    pub fn reset(&mut self) {
        self.values = self.reference_values.clone();
        self.is_modified = false;
    }
}

/// Calibration parameter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CalibrationParamType {
    /// Scalar value
    #[default]
    Scalar,
    /// 1D curve (lookup table)
    Curve,
    /// 2D map (lookup table)
    Map,
    /// 3D cuboid
    Cuboid,
    /// ASCII string
    Ascii,
    /// Value block (array)
    ValueBlock,
}

/// Calibration data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CalibrationDataType {
    /// Unsigned 8-bit
    UInt8,
    /// Signed 8-bit
    Int8,
    /// Unsigned 16-bit
    UInt16,
    /// Signed 16-bit
    Int16,
    /// Unsigned 32-bit
    UInt32,
    /// Signed 32-bit
    Int32,
    /// 32-bit float
    #[default]
    Float32,
    /// 64-bit float
    Float64,
}

impl CalibrationDataType {
    /// Get size in bytes
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::UInt8 | Self::Int8 => 1,
            Self::UInt16 | Self::Int16 => 2,
            Self::UInt32 | Self::Int32 | Self::Float32 => 4,
            Self::Float64 => 8,
        }
    }
}

/// Axis definition for curves and maps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxisDefinition {
    /// Axis name
    pub name: String,
    /// Number of points
    pub size: u16,
    /// Axis values (breakpoints)
    pub values: Vec<f64>,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Unit string
    pub unit: String,
    /// Input quantity name (measurement reference)
    pub input_quantity: Option<String>,
}

impl AxisDefinition {
    /// Create a new axis definition
    #[must_use]
    pub fn new(name: &str, size: u16) -> Self {
        Self {
            name: name.to_string(),
            size,
            values: vec![0.0; size as usize],
            min: f64::MIN,
            max: f64::MAX,
            unit: String::new(),
            input_quantity: None,
        }
    }

    /// Set axis breakpoints
    #[must_use]
    pub fn with_values(mut self, values: Vec<f64>) -> Self {
        self.values = values;
        self
    }
}

// ============================================================================
// Memory Page Management (Working/Reference Pages)
// ============================================================================

/// Memory page type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MemoryPageType {
    /// Reference page (original calibration data)
    #[default]
    Reference,
    /// Working page (modified calibration data)
    Working,
    /// Flash page
    Flash,
    /// RAM page
    Ram,
}

/// Memory page definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryPage {
    /// Page number
    pub number: u8,
    /// Page type
    pub page_type: MemoryPageType,
    /// Segment number
    pub segment: u8,
    /// Start address
    pub start_address: u64,
    /// Size in bytes
    pub size: u64,
    /// Is page active
    pub is_active: bool,
    /// Page name
    pub name: String,
}

impl MemoryPage {
    /// Create a new memory page
    #[must_use]
    pub fn new(number: u8, page_type: MemoryPageType, segment: u8) -> Self {
        Self {
            number,
            page_type,
            segment,
            start_address: 0,
            size: 0,
            is_active: false,
            name: format!("Page_{}", number),
        }
    }
}

/// Page switching mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PageSwitchMode {
    /// ECU access mode (which page ECU uses)
    #[default]
    EcuAccess,
    /// XCP access mode (which page XCP reads/writes)
    XcpAccess,
    /// Both ECU and XCP access
    All,
}

// ============================================================================
// Calibration Dataset Management (CDF-like)
// ============================================================================

/// Calibration dataset (similar to CDF/CDFX)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalibrationDataset {
    /// Dataset name
    pub name: String,
    /// Description
    pub description: String,
    /// Version string
    pub version: String,
    /// Creation timestamp
    pub created: String,
    /// Last modified timestamp
    pub modified: String,
    /// Author
    pub author: String,
    /// A2L file reference
    pub a2l_file: String,
    /// Parameter values: name -> values
    pub parameters: HashMap<String, Vec<f64>>,
    /// Axis values: name -> values
    pub axes: HashMap<String, Vec<f64>>,
    /// Comments/annotations
    pub comments: HashMap<String, String>,
}

impl CalibrationDataset {
    /// Create a new empty dataset
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            version: "1.0".to_string(),
            created: chrono_now(),
            modified: chrono_now(),
            author: String::new(),
            a2l_file: String::new(),
            parameters: HashMap::new(),
            axes: HashMap::new(),
            comments: HashMap::new(),
        }
    }

    /// Add parameter values
    pub fn add_parameter(&mut self, name: &str, values: Vec<f64>) {
        self.parameters.insert(name.to_string(), values);
        self.modified = chrono_now();
    }

    /// Get parameter values
    #[must_use]
    pub fn get_parameter(&self, name: &str) -> Option<&Vec<f64>> {
        self.parameters.get(name)
    }

    /// Check if dataset contains parameter
    #[must_use]
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }

    /// Get number of parameters
    #[must_use]
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }
}

/// Get current timestamp as string
fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    "2026-01-26T00:00:00Z".to_string()
}

// ============================================================================
// Flash Programming
// ============================================================================

/// Flash programming state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FlashState {
    /// Idle, not programming
    #[default]
    Idle,
    /// Preparing for programming
    Preparing,
    /// Erasing flash sectors
    Erasing,
    /// Programming data
    Programming,
    /// Verifying programmed data
    Verifying,
    /// Programming complete
    Complete,
    /// Error occurred
    Error,
}

/// Flash sector definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlashSector {
    /// Sector number
    pub number: u16,
    /// Start address
    pub start_address: u64,
    /// Size in bytes
    pub size: u64,
    /// Is sector erased
    pub is_erased: bool,
    /// Is sector programmed
    pub is_programmed: bool,
    /// Sector name
    pub name: String,
}

/// Flash programming progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlashProgress {
    /// Current state
    pub state: FlashState,
    /// Total bytes to program
    pub total_bytes: u64,
    /// Bytes programmed so far
    pub bytes_done: u64,
    /// Current sector being processed
    pub current_sector: u16,
    /// Total sectors
    pub total_sectors: u16,
    /// Error message if any
    pub error_message: Option<String>,
}

impl FlashProgress {
    /// Get progress percentage (0-100)
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn percentage(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.bytes_done as f32 / self.total_bytes as f32) * 100.0
    }

    /// Check if programming is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.state == FlashState::Complete
    }

    /// Check if error occurred
    #[must_use]
    pub fn has_error(&self) -> bool {
        self.state == FlashState::Error
    }
}

// ============================================================================
// Seed & Key Security
// ============================================================================

/// Security access level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// No security (unlocked)
    #[default]
    Unlocked,
    /// Level 1 - Basic access
    Level1,
    /// Level 2 - Calibration access
    Level2,
    /// Level 3 - Programming access
    Level3,
    /// Level 4 - Full access
    Level4,
}

/// Seed & Key handler trait
pub trait SeedKeyHandler: Send + Sync {
    /// Calculate key from seed
    fn calculate_key(&self, seed: &[u8], level: SecurityLevel) -> Result<Vec<u8>>;
    
    /// Get supported security levels
    fn supported_levels(&self) -> Vec<SecurityLevel>;
}

/// Default seed & key handler (XOR-based, for testing only)
#[derive(Debug, Clone, Default)]
pub struct DefaultSeedKeyHandler {
    /// XOR mask for key calculation
    pub xor_mask: u32,
}

impl DefaultSeedKeyHandler {
    /// Create with XOR mask
    #[must_use]
    pub fn new(xor_mask: u32) -> Self {
        Self { xor_mask }
    }
}

impl SeedKeyHandler for DefaultSeedKeyHandler {
    fn calculate_key(&self, seed: &[u8], _level: SecurityLevel) -> Result<Vec<u8>> {
        // Simple XOR-based key calculation (NOT SECURE - for testing only)
        let mut key = seed.to_vec();
        let mask_bytes = self.xor_mask.to_le_bytes();
        for (i, byte) in key.iter_mut().enumerate() {
            *byte ^= mask_bytes[i % 4];
        }
        Ok(key)
    }

    fn supported_levels(&self) -> Vec<SecurityLevel> {
        vec![SecurityLevel::Level1, SecurityLevel::Level2]
    }
}

// ============================================================================
// HEX File Support (Intel HEX, Motorola S-Record)
// ============================================================================

/// HEX file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HexFormat {
    /// Intel HEX format
    #[default]
    IntelHex,
    /// Motorola S-Record format
    SRecord,
}

/// HEX file data record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HexRecord {
    /// Start address
    pub address: u64,
    /// Data bytes
    pub data: Vec<u8>,
}

/// Parsed HEX file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HexFile {
    /// File format
    pub format: HexFormat,
    /// Data records
    pub records: Vec<HexRecord>,
    /// Start address (entry point)
    pub start_address: Option<u64>,
}

impl HexFile {
    /// Create a new empty HEX file
    #[must_use]
    pub fn new(format: HexFormat) -> Self {
        Self {
            format,
            records: Vec::new(),
            start_address: None,
        }
    }

    /// Parse Intel HEX format
    pub fn parse_intel_hex(content: &str) -> Result<Self> {
        let mut file = Self::new(HexFormat::IntelHex);
        let mut extended_address: u64 = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || !line.starts_with(':') {
                continue;
            }

            let bytes = Self::parse_hex_line(&line[1..]);
            if bytes.len() < 5 {
                continue;
            }

            let byte_count = bytes[0] as usize;
            let address = u16::from_be_bytes([bytes[1], bytes[2]]) as u64;
            let record_type = bytes[3];

            match record_type {
                0x00 => {
                    // Data record
                    if bytes.len() >= 4 + byte_count {
                        let data = bytes[4..4 + byte_count].to_vec();
                        file.records.push(HexRecord {
                            address: extended_address + address,
                            data,
                        });
                    }
                }
                0x01 => break, // EOF
                0x02 => {
                    // Extended segment address
                    if bytes.len() >= 6 {
                        extended_address =
                            (u16::from_be_bytes([bytes[4], bytes[5]]) as u64) << 4;
                    }
                }
                0x04 => {
                    // Extended linear address
                    if bytes.len() >= 6 {
                        extended_address =
                            (u16::from_be_bytes([bytes[4], bytes[5]]) as u64) << 16;
                    }
                }
                0x05 => {
                    // Start linear address
                    if bytes.len() >= 8 {
                        file.start_address = Some(u32::from_be_bytes([
                            bytes[4], bytes[5], bytes[6], bytes[7],
                        ]) as u64);
                    }
                }
                _ => {}
            }
        }

        Ok(file)
    }

    /// Parse Motorola S-Record format
    pub fn parse_srecord(content: &str) -> Result<Self> {
        let mut file = Self::new(HexFormat::SRecord);

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('S') {
                continue;
            }

            if line.len() < 4 {
                continue;
            }

            let record_type = line.chars().nth(1).unwrap_or('0');
            let bytes = Self::parse_hex_line(&line[2..]);

            if bytes.is_empty() {
                continue;
            }

            let byte_count = bytes[0] as usize;
            if bytes.len() < byte_count {
                continue;
            }

            match record_type {
                '1' => {
                    // 16-bit address data
                    if bytes.len() >= 3 {
                        let address = u16::from_be_bytes([bytes[1], bytes[2]]) as u64;
                        let data = bytes[3..byte_count].to_vec();
                        file.records.push(HexRecord { address, data });
                    }
                }
                '2' => {
                    // 24-bit address data
                    if bytes.len() >= 4 {
                        let address = u32::from_be_bytes([0, bytes[1], bytes[2], bytes[3]]) as u64;
                        let data = bytes[4..byte_count].to_vec();
                        file.records.push(HexRecord { address, data });
                    }
                }
                '3' => {
                    // 32-bit address data
                    if bytes.len() >= 5 {
                        let address =
                            u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
                        let data = bytes[5..byte_count].to_vec();
                        file.records.push(HexRecord { address, data });
                    }
                }
                '7' | '8' | '9' => {
                    // Start address records
                    if bytes.len() >= 5 {
                        file.start_address = Some(
                            u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64,
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(file)
    }

    fn parse_hex_line(hex: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut chars = hex.chars().peekable();
        while chars.peek().is_some() {
            let high = chars.next().and_then(|c| c.to_digit(16));
            let low = chars.next().and_then(|c| c.to_digit(16));
            match (high, low) {
                (Some(h), Some(l)) => bytes.push((h * 16 + l) as u8),
                _ => break,
            }
        }
        bytes
    }

    /// Get total data size
    #[must_use]
    pub fn total_size(&self) -> usize {
        self.records.iter().map(|r| r.data.len()).sum()
    }

    /// Get address range
    #[must_use]
    pub fn address_range(&self) -> Option<(u64, u64)> {
        if self.records.is_empty() {
            return None;
        }
        let min = self.records.iter().map(|r| r.address).min()?;
        let max = self
            .records
            .iter()
            .map(|r| r.address + r.data.len() as u64)
            .max()?;
        Some((min, max))
    }
}

// ============================================================================
// Calibration Session (Main Orchestrator)
// ============================================================================

/// Calibration session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SessionState {
    /// Disconnected from ECU
    #[default]
    Disconnected,
    /// Connecting to ECU
    Connecting,
    /// Connected and idle
    Connected,
    /// Measuring (DAQ active)
    Measuring,
    /// Calibrating (writing parameters)
    Calibrating,
    /// Programming flash
    Programming,
    /// Error state
    Error,
}

/// Calibration session configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionConfig {
    /// A2L file path
    pub a2l_file: Option<String>,
    /// Transport type (CAN, Ethernet, etc.)
    pub transport: TransportType,
    /// CAN ID for XCP master
    pub master_id: u32,
    /// CAN ID for XCP slave
    pub slave_id: u32,
    /// Connection timeout in milliseconds
    pub timeout_ms: u32,
    /// Auto-connect on session start
    pub auto_connect: bool,
    /// Enable DAQ timestamps
    pub daq_timestamps: bool,
}

/// Transport type for XCP communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransportType {
    /// CAN transport
    #[default]
    Can,
    /// CAN FD transport
    CanFd,
    /// Ethernet (UDP) transport
    EthernetUdp,
    /// Ethernet (TCP) transport
    EthernetTcp,
    /// USB transport
    Usb,
}

/// Calibration session - main interface for measurement and calibration
#[derive(Debug, Default)]
pub struct CalibrationSession {
    /// Session state
    pub state: SessionState,
    /// Configuration
    pub config: SessionConfig,
    /// Current security level
    pub security_level: SecurityLevel,
    /// Loaded measurements (from A2L)
    pub measurements: HashMap<String, MeasurementSignal>,
    /// Loaded parameters (from A2L)
    pub parameters: HashMap<String, CalibrationParameter>,
    /// Active measurement configuration
    pub measurement_config: Option<MeasurementConfig>,
    /// Current recording
    pub recording: Option<MeasurementRecording>,
    /// Memory pages
    pub pages: Vec<MemoryPage>,
    /// Active page number
    pub active_page: u8,
    /// Flash progress
    pub flash_progress: FlashProgress,
    /// Current dataset
    pub dataset: Option<CalibrationDataset>,
    /// Error message
    pub error_message: Option<String>,
}

impl CalibrationSession {
    /// Create a new calibration session
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with configuration
    #[must_use]
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Add a measurement signal
    pub fn add_measurement(&mut self, signal: MeasurementSignal) {
        self.measurements.insert(signal.name.clone(), signal);
    }

    /// Add a calibration parameter
    pub fn add_parameter(&mut self, param: CalibrationParameter) {
        self.parameters.insert(param.name.clone(), param);
    }

    /// Get measurement by name
    #[must_use]
    pub fn get_measurement(&self, name: &str) -> Option<&MeasurementSignal> {
        self.measurements.get(name)
    }

    /// Get mutable measurement by name
    pub fn get_measurement_mut(&mut self, name: &str) -> Option<&mut MeasurementSignal> {
        self.measurements.get_mut(name)
    }

    /// Get parameter by name
    #[must_use]
    pub fn get_parameter(&self, name: &str) -> Option<&CalibrationParameter> {
        self.parameters.get(name)
    }

    /// Get mutable parameter by name
    pub fn get_parameter_mut(&mut self, name: &str) -> Option<&mut CalibrationParameter> {
        self.parameters.get_mut(name)
    }

    /// Start measurement with configuration
    pub fn start_measurement(&mut self, config: MeasurementConfig) -> Result<()> {
        if self.state != SessionState::Connected {
            return Err(BusmasterError::protocol(
                "Must be connected to start measurement",
            ));
        }
        self.measurement_config = Some(config);
        self.state = SessionState::Measuring;
        Ok(())
    }

    /// Stop measurement
    pub fn stop_measurement(&mut self) -> Result<()> {
        if self.state != SessionState::Measuring {
            return Err(BusmasterError::protocol("Not currently measuring"));
        }
        self.measurement_config = None;
        self.state = SessionState::Connected;
        Ok(())
    }

    /// Start recording
    pub fn start_recording(&mut self, name: &str) -> Result<()> {
        let mut recording = MeasurementRecording::new(name);
        recording.start();
        self.recording = Some(recording);
        Ok(())
    }

    /// Stop recording
    pub fn stop_recording(&mut self) -> Option<MeasurementRecording> {
        if let Some(mut recording) = self.recording.take() {
            recording.stop();
            Some(recording)
        } else {
            None
        }
    }

    /// Get all modified parameters
    #[must_use]
    pub fn get_modified_parameters(&self) -> Vec<&CalibrationParameter> {
        self.parameters.values().filter(|p| p.is_dirty()).collect()
    }

    /// Reset all parameters to reference values
    pub fn reset_all_parameters(&mut self) {
        for param in self.parameters.values_mut() {
            param.reset();
        }
    }

    /// Save current parameters to dataset
    pub fn save_to_dataset(&mut self, name: &str) -> CalibrationDataset {
        let mut dataset = CalibrationDataset::new(name);
        for (param_name, param) in &self.parameters {
            dataset.add_parameter(param_name, param.values.clone());
        }
        dataset
    }

    /// Load parameters from dataset
    pub fn load_from_dataset(&mut self, dataset: &CalibrationDataset) -> Result<()> {
        for (name, values) in &dataset.parameters {
            if let Some(param) = self.parameters.get_mut(name) {
                if param.values.len() == values.len() {
                    param.values.clone_from(values);
                    param.is_modified = true;
                }
            }
        }
        Ok(())
    }

    /// Check if session is connected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        matches!(
            self.state,
            SessionState::Connected | SessionState::Measuring | SessionState::Calibrating
        )
    }

    /// Check if measurement is active
    #[must_use]
    pub fn is_measuring(&self) -> bool {
        self.state == SessionState::Measuring
    }

    /// Get session statistics
    #[must_use]
    pub fn statistics(&self) -> SessionStatistics {
        SessionStatistics {
            measurement_count: self.measurements.len(),
            parameter_count: self.parameters.len(),
            modified_parameter_count: self.get_modified_parameters().len(),
            recording_samples: self.recording.as_ref().map_or(0, MeasurementRecording::total_samples),
            page_count: self.pages.len(),
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStatistics {
    /// Number of measurements loaded
    pub measurement_count: usize,
    /// Number of parameters loaded
    pub parameter_count: usize,
    /// Number of modified parameters
    pub modified_parameter_count: usize,
    /// Number of recorded samples
    pub recording_samples: usize,
    /// Number of memory pages
    pub page_count: usize,
}

// ============================================================================
// DAQ List Builder (from A2L measurements)
// ============================================================================

/// DAQ list builder for creating XCP DAQ configurations from measurements
#[derive(Debug, Default)]
pub struct DaqListBuilder {
    /// Event channel to DAQ list mapping
    lists: HashMap<u16, Vec<DaqEntry>>,
    /// Maximum ODT entries per ODT
    max_odt_entries: u8,
}

/// DAQ entry for a single measurement
#[derive(Debug, Clone)]
pub struct DaqEntry {
    /// Signal name
    pub name: String,
    /// ECU address
    pub address: u64,
    /// Address extension
    pub address_extension: u8,
    /// Size in bytes
    pub size: u8,
}

impl DaqListBuilder {
    /// Create a new DAQ list builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            lists: HashMap::new(),
            max_odt_entries: 7,
        }
    }

    /// Set maximum ODT entries per ODT
    #[must_use]
    pub fn with_max_odt_entries(mut self, max: u8) -> Self {
        self.max_odt_entries = max;
        self
    }

    /// Add a measurement signal to the builder
    pub fn add_signal(&mut self, signal: &MeasurementSignal) {
        let entry = DaqEntry {
            name: signal.name.clone(),
            address: signal.address,
            address_extension: signal.address_extension,
            size: signal.size,
        };
        self.lists
            .entry(signal.event_channel)
            .or_default()
            .push(entry);
    }

    /// Add multiple signals from measurement config
    pub fn add_config(&mut self, config: &MeasurementConfig) {
        for signal in &config.signals {
            self.add_signal(signal);
        }
    }

    /// Get number of DAQ lists needed
    #[must_use]
    pub fn list_count(&self) -> usize {
        self.lists.len()
    }

    /// Get total number of entries
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.lists.values().map(Vec::len).sum()
    }

    /// Build XCP DAQ lists
    #[must_use]
    pub fn build(&self) -> Vec<DaqList> {
        let mut result = Vec::new();
        for (event_channel, entries) in &self.lists {
            let mut daq_list = DaqList {
                number: result.len() as u16,
                mode: DaqListMode::default(),
                event_channel: *event_channel,
                prescaler: 1,
                priority: 0,
                odts: Vec::new(),
            };

            // Pack entries into ODTs
            let mut current_odt = Odt::new();

            for entry in entries {
                if current_odt.entries.len() >= self.max_odt_entries as usize {
                    daq_list.odts.push(current_odt);
                    current_odt = Odt::new();
                }
                current_odt.entries.push(OdtEntry {
                    address: entry.address as u32,
                    address_extension: entry.address_extension,
                    size: entry.size,
                });
            }

            if !current_odt.entries.is_empty() {
                daq_list.odts.push(current_odt);
            }

            result.push(daq_list);
        }
        result
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_signal_update() {
        let mut signal = MeasurementSignal::new("Speed", 0x1000, 2)
            .with_conversion(0.01, 0.0)
            .with_unit("km/h");

        signal.update(&[0x10, 0x27]); // 10000 in little-endian
        assert!((signal.physical_value - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_measurement_config_grouping() {
        let config = MeasurementConfig::new()
            .add_signal(MeasurementSignal::new("Speed", 0x1000, 2).with_rate(10))
            .add_signal(MeasurementSignal::new("RPM", 0x1002, 2).with_rate(10))
            .add_signal(MeasurementSignal::new("Temp", 0x1004, 1).with_rate(100));

        let groups = config.group_by_event_channel();
        assert_eq!(groups.len(), 1); // All on event channel 0
        assert_eq!(groups.get(&0).unwrap().len(), 3);
    }

    #[test]
    fn test_measurement_recording() {
        let mut recording = MeasurementRecording::new("test");
        recording.start();
        recording.add_sample("Speed", 100.0);
        recording.add_sample("Speed", 101.0);
        recording.add_sample("RPM", 3000.0);
        recording.stop();

        assert_eq!(recording.sample_count("Speed"), 2);
        assert_eq!(recording.sample_count("RPM"), 1);
        assert_eq!(recording.total_samples(), 3);
    }

    #[test]
    fn test_calibration_parameter_scalar() {
        let mut param = CalibrationParameter::new_scalar(
            "IdleSpeed",
            0x2000,
            CalibrationDataType::UInt16,
        );
        param.min = 500.0;
        param.max = 1500.0;

        assert!(param.set_value(0, 850.0).is_ok());
        assert!(param.is_dirty());
        assert_eq!(param.values[0], 850.0);

        // Out of range should fail
        assert!(param.set_value(0, 2000.0).is_err());
    }

    #[test]
    fn test_calibration_parameter_reset() {
        let mut param = CalibrationParameter::new_scalar(
            "Test",
            0x2000,
            CalibrationDataType::Float32,
        );
        param.reference_values = vec![100.0];
        param.values = vec![200.0];
        param.is_modified = true;

        param.reset();
        assert_eq!(param.values[0], 100.0);
        assert!(!param.is_dirty());
    }

    #[test]
    fn test_calibration_dataset() {
        let mut dataset = CalibrationDataset::new("TestDataset");
        dataset.add_parameter("Speed", vec![100.0, 200.0, 300.0]);
        dataset.add_parameter("Temp", vec![25.0]);

        assert_eq!(dataset.parameter_count(), 2);
        assert!(dataset.has_parameter("Speed"));
        assert_eq!(dataset.get_parameter("Speed").unwrap().len(), 3);
    }

    #[test]
    fn test_flash_progress() {
        let mut progress = FlashProgress::default();
        progress.total_bytes = 1000;
        progress.bytes_done = 500;

        assert!((progress.percentage() - 50.0).abs() < 0.1);
        assert!(!progress.is_complete());

        progress.state = FlashState::Complete;
        assert!(progress.is_complete());
    }

    #[test]
    fn test_default_seed_key_handler() {
        let handler = DefaultSeedKeyHandler::new(0x12345678);
        let seed = vec![0x01, 0x02, 0x03, 0x04];
        let key = handler.calculate_key(&seed, SecurityLevel::Level1).unwrap();

        // XOR with mask bytes
        assert_eq!(key.len(), 4);
        assert_eq!(key[0], 0x01 ^ 0x78);
        assert_eq!(key[1], 0x02 ^ 0x56);
    }

    #[test]
    fn test_hex_file_parse_intel() {
        let hex = r#"
:020000040000FA
:10000000214601360121470136007EFE09D2190140
:100010002146017E17C20001FF5F16002148011928
:00000001FF
"#;
        let file = HexFile::parse_intel_hex(hex).unwrap();
        assert_eq!(file.format, HexFormat::IntelHex);
        assert_eq!(file.records.len(), 2);
        assert_eq!(file.records[0].address, 0);
        assert_eq!(file.records[0].data.len(), 16);
    }

    #[test]
    fn test_hex_file_parse_srecord() {
        let srec = r#"
S00600004844521B
S1130000285F245F2212226A000424290008237C2A
S5030001FB
S9030000FC
"#;
        let file = HexFile::parse_srecord(srec).unwrap();
        assert_eq!(file.format, HexFormat::SRecord);
        assert!(!file.records.is_empty());
    }

    #[test]
    fn test_calibration_session() {
        let mut session = CalibrationSession::new();
        session.add_measurement(MeasurementSignal::new("Speed", 0x1000, 2));
        session.add_parameter(CalibrationParameter::new_scalar(
            "IdleSpeed",
            0x2000,
            CalibrationDataType::UInt16,
        ));

        let stats = session.statistics();
        assert_eq!(stats.measurement_count, 1);
        assert_eq!(stats.parameter_count, 1);
    }

    #[test]
    fn test_daq_list_builder() {
        let mut builder = DaqListBuilder::new();
        builder.add_signal(&MeasurementSignal::new("Speed", 0x1000, 2));
        builder.add_signal(&MeasurementSignal::new("RPM", 0x1002, 2));

        assert_eq!(builder.entry_count(), 2);

        let lists = builder.build();
        assert_eq!(lists.len(), 1);
        assert!(!lists[0].odts.is_empty());
    }

    #[test]
    fn test_data_type_sizes() {
        assert_eq!(CalibrationDataType::UInt8.size_bytes(), 1);
        assert_eq!(CalibrationDataType::UInt16.size_bytes(), 2);
        assert_eq!(CalibrationDataType::UInt32.size_bytes(), 4);
        assert_eq!(CalibrationDataType::Float64.size_bytes(), 8);
    }

    #[test]
    fn test_memory_page() {
        let page = MemoryPage::new(0, MemoryPageType::Working, 0);
        assert_eq!(page.number, 0);
        assert_eq!(page.page_type, MemoryPageType::Working);
        assert!(!page.is_active);
    }
}
