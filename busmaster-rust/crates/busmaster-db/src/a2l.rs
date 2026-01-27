//! A2L (ASAP2) File Parser
//!
//! This module provides parsing for ASAM MCD-2 MC (A2L/ASAP2) files, which describe
//! internal ECU data for measurement and calibration purposes.
//!
//! # A2L Format Overview
//!
//! A2L files are structured ASCII text files containing:
//! - PROJECT - Root container for all data
//! - HEADER - Project metadata (version, ECU info)
//! - MODULE - ECU description container
//! - MEASUREMENT - Recordable ECU variables
//! - CHARACTERISTIC - Tunable parameters (scalars, curves, maps)
//! - COMPU_METHOD - Value conversion methods
//! - RECORD_LAYOUT - Memory layout descriptions
//! - AXIS_PTS - Axis point definitions for curves/maps
//! - IF_DATA - Communication interface configuration
//!
//! # Standards Compliance
//!
//! This implementation follows ASAM MCD-2 MC v1.7 specification.
//!
//! # Example
//!
//! ```
//! use busmaster_db::a2l::A2lParser;
//!
//! let a2l = r#"
//! ASAP2_VERSION 1 71
//! /begin PROJECT TestProject ""
//!   /begin MODULE TestModule ""
//!     /begin MEASUREMENT EngineSpeed
//!       "Engine rotational speed"
//!       UWORD NO_COMPU_METHOD 0 0 0 16383
//!     /end MEASUREMENT
//!   /end MODULE
//! /end PROJECT
//! "#;
//!
//! let db = A2lParser::parse(a2l).unwrap();
//! assert_eq!(db.project.name, "TestProject");
//! ```

#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::format_push_string)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::manual_strip)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_possible_truncation)]

use busmaster_core::{BusmasterError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// ASAP2 version information
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Asap2Version {
    /// Major version number
    pub major: u16,
    /// Minor version number (e.g., 71 for v1.71)
    pub minor: u16,
}

impl Asap2Version {
    /// Create a new ASAP2 version
    #[must_use]
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Get version as string (e.g., "1.71")
    #[must_use]
    pub fn to_version_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

/// A2L data types as defined in ASAM MCD-2 MC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum A2lDataType {
    /// Unsigned 8-bit integer
    #[default]
    UByte,
    /// Signed 8-bit integer (two's complement)
    SByte,
    /// Unsigned 16-bit integer
    UWord,
    /// Signed 16-bit integer (two's complement)
    SWord,
    /// Unsigned 32-bit integer
    ULong,
    /// Signed 32-bit integer (two's complement)
    SLong,
    /// Unsigned 64-bit integer
    AUint64,
    /// Signed 64-bit integer (two's complement)
    AInt64,
    /// IEEE 754 single precision float (32-bit)
    Float32Ieee,
    /// IEEE 754 double precision float (64-bit)
    Float64Ieee,
}

impl A2lDataType {
    /// Parse from string
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "UBYTE" => Some(Self::UByte),
            "SBYTE" => Some(Self::SByte),
            "UWORD" => Some(Self::UWord),
            "SWORD" => Some(Self::SWord),
            "ULONG" => Some(Self::ULong),
            "SLONG" => Some(Self::SLong),
            "A_UINT64" => Some(Self::AUint64),
            "A_INT64" => Some(Self::AInt64),
            "FLOAT32_IEEE" => Some(Self::Float32Ieee),
            "FLOAT64_IEEE" => Some(Self::Float64Ieee),
            _ => None,
        }
    }

    /// Get size in bytes
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::UByte | Self::SByte => 1,
            Self::UWord | Self::SWord => 2,
            Self::ULong | Self::SLong | Self::Float32Ieee => 4,
            Self::AUint64 | Self::AInt64 | Self::Float64Ieee => 8,
        }
    }

    /// Check if signed
    #[must_use]
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::SByte
                | Self::SWord
                | Self::SLong
                | Self::AInt64
                | Self::Float32Ieee
                | Self::Float64Ieee
        )
    }
}


/// Computation method type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CompuMethodType {
    /// No conversion (identical)
    #[default]
    Identical,
    /// Linear conversion: y = ax + b
    Linear,
    /// Rational function: 6-coefficient
    RatFunc,
    /// Table with interpolation
    TabIntp,
    /// Table without interpolation
    TabNoIntp,
    /// Verbal table (enumeration)
    TabVerb,
    /// Formula-based conversion
    Form,
}

impl CompuMethodType {
    /// Parse from string
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "IDENTICAL" => Some(Self::Identical),
            "LINEAR" => Some(Self::Linear),
            "RAT_FUNC" => Some(Self::RatFunc),
            "TAB_INTP" => Some(Self::TabIntp),
            "TAB_NOINTP" => Some(Self::TabNoIntp),
            "TAB_VERB" => Some(Self::TabVerb),
            "FORM" => Some(Self::Form),
            _ => None,
        }
    }
}

/// Characteristic type (tunable parameter type)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CharacteristicType {
    /// Scalar value
    #[default]
    Value,
    /// ASCII string
    Ascii,
    /// Value block (1D array, no axes)
    ValBlk,
    /// 1D lookup table (curve)
    Curve,
    /// 2D lookup table (map)
    Map,
    /// 3D lookup table (cuboid)
    Cuboid,
    /// 4D lookup table
    Cube4,
    /// 5D lookup table
    Cube5,
}

impl CharacteristicType {
    /// Parse from string
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "VALUE" => Some(Self::Value),
            "ASCII" => Some(Self::Ascii),
            "VAL_BLK" => Some(Self::ValBlk),
            "CURVE" => Some(Self::Curve),
            "MAP" => Some(Self::Map),
            "CUBOID" => Some(Self::Cuboid),
            "CUBE_4" => Some(Self::Cube4),
            "CUBE_5" => Some(Self::Cube5),
            _ => None,
        }
    }
}

/// Axis type for curves and maps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AxisType {
    /// Standard axis specific to one table
    #[default]
    StdAxis,
    /// Fixed axis with calculated points (not stored in memory)
    FixAxis,
    /// Common axis shared by multiple tables
    ComAxis,
    /// Curve axis (shared and rescaled by a curve)
    CurveAxis,
    /// Rescale axis (shared and rescaled by another axis)
    ResAxis,
}

impl AxisType {
    /// Parse from string
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "STD_AXIS" => Some(Self::StdAxis),
            "FIX_AXIS" => Some(Self::FixAxis),
            "COM_AXIS" => Some(Self::ComAxis),
            "CURVE_AXIS" => Some(Self::CurveAxis),
            "RES_AXIS" => Some(Self::ResAxis),
            _ => None,
        }
    }
}


/// Byte order specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum A2lByteOrder {
    /// Little-endian (LSB first)
    #[default]
    LittleEndian,
    /// Big-endian (MSB first)
    BigEndian,
}

/// Deposit mode for axis points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DepositMode {
    /// Absolute values stored
    #[default]
    Absolute,
    /// Difference values between points stored
    Difference,
}

/// A2L Database - root container for parsed A2L file
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lDatabase {
    /// ASAP2 version
    pub asap2_version: Asap2Version,
    /// A2ML version (optional)
    pub a2ml_version: Option<Asap2Version>,
    /// Project definition
    pub project: A2lProject,
}

impl A2lDatabase {
    /// Create a new empty A2L database
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Find a measurement by name
    #[must_use]
    pub fn find_measurement(&self, name: &str) -> Option<&A2lMeasurement> {
        self.project
            .modules
            .iter()
            .flat_map(|m| m.measurements.iter())
            .find(|meas| meas.name == name)
    }

    /// Find a characteristic by name
    #[must_use]
    pub fn find_characteristic(&self, name: &str) -> Option<&A2lCharacteristic> {
        self.project
            .modules
            .iter()
            .flat_map(|m| m.characteristics.iter())
            .find(|c| c.name == name)
    }

    /// Find a computation method by name
    #[must_use]
    pub fn find_compu_method(&self, name: &str) -> Option<&A2lCompuMethod> {
        self.project
            .modules
            .iter()
            .flat_map(|m| m.compu_methods.iter())
            .find(|cm| cm.name == name)
    }

    /// Get all measurements across all modules
    #[must_use]
    pub fn all_measurements(&self) -> Vec<&A2lMeasurement> {
        self.project
            .modules
            .iter()
            .flat_map(|m| m.measurements.iter())
            .collect()
    }

    /// Get all characteristics across all modules
    #[must_use]
    pub fn all_characteristics(&self) -> Vec<&A2lCharacteristic> {
        self.project
            .modules
            .iter()
            .flat_map(|m| m.characteristics.iter())
            .collect()
    }
}


/// A2L Project - top-level container
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lProject {
    /// Project name
    pub name: String,
    /// Long identifier/description
    pub long_identifier: String,
    /// Header information
    pub header: Option<A2lHeader>,
    /// Modules (ECU descriptions)
    pub modules: Vec<A2lModule>,
}

/// A2L Header - project metadata
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lHeader {
    /// Comment/description
    pub comment: String,
    /// Project number
    pub project_no: Option<String>,
    /// Version string
    pub version: Option<String>,
}

/// A2L Module - ECU description container
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lModule {
    /// Module name
    pub name: String,
    /// Long identifier/description
    pub long_identifier: String,
    /// Common module parameters
    pub mod_common: Option<A2lModCommon>,
    /// Module parameters
    pub mod_par: Option<A2lModPar>,
    /// Measurements (recordable variables)
    pub measurements: Vec<A2lMeasurement>,
    /// Characteristics (tunable parameters)
    pub characteristics: Vec<A2lCharacteristic>,
    /// Computation methods
    pub compu_methods: Vec<A2lCompuMethod>,
    /// Computation tables
    pub compu_tabs: Vec<A2lCompuTab>,
    /// Verbal computation tables
    pub compu_vtabs: Vec<A2lCompuVtab>,
    /// Record layouts
    pub record_layouts: Vec<A2lRecordLayout>,
    /// Axis points definitions
    pub axis_pts: Vec<A2lAxisPts>,
    /// Functions (grouping)
    pub functions: Vec<A2lFunction>,
    /// Groups
    pub groups: Vec<A2lGroup>,
    /// Units
    pub units: Vec<A2lUnit>,
    /// IF_DATA sections (interface configuration)
    pub if_data: Vec<A2lIfData>,
}


/// MOD_COMMON - common module parameters
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lModCommon {
    /// Comment
    pub comment: String,
    /// Byte order
    pub byte_order: Option<A2lByteOrder>,
    /// Data size in bits
    pub data_size: Option<u16>,
    /// Deposit mode
    pub deposit: Option<DepositMode>,
    /// Alignment for byte data
    pub alignment_byte: Option<u16>,
    /// Alignment for word data
    pub alignment_word: Option<u16>,
    /// Alignment for long data
    pub alignment_long: Option<u16>,
    /// Alignment for float32 data
    pub alignment_float32: Option<u16>,
    /// Alignment for float64 data
    pub alignment_float64: Option<u16>,
    /// Alignment for int64 data
    pub alignment_int64: Option<u16>,
}

/// MOD_PAR - module parameters
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lModPar {
    /// Comment
    pub comment: String,
    /// CPU type
    pub cpu_type: Option<String>,
    /// Customer name
    pub customer: Option<String>,
    /// Customer number
    pub customer_no: Option<String>,
    /// ECU description
    pub ecu: Option<String>,
    /// ECU calibration offset
    pub ecu_calibration_offset: Option<i64>,
    /// EPK address
    pub addr_epk: Option<u64>,
    /// EPK string
    pub epk: Option<String>,
    /// Number of interfaces
    pub no_of_interfaces: Option<u16>,
    /// Phone number
    pub phone_no: Option<String>,
    /// Supplier
    pub supplier: Option<String>,
    /// User
    pub user: Option<String>,
    /// Version
    pub version: Option<String>,
    /// Memory segments
    pub memory_segments: Vec<A2lMemorySegment>,
    /// System constants
    pub system_constants: HashMap<String, String>,
}

/// Memory segment definition
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lMemorySegment {
    /// Segment name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Program type (PRG_CODE, PRG_DATA, PRG_RESERVED)
    pub prg_type: String,
    /// Memory type (RAM, EEPROM, EPROM, ROM, FLASH, etc.)
    pub memory_type: String,
    /// Attribute (INTERN, EXTERN)
    pub attribute: String,
    /// Start address
    pub address: u64,
    /// Size in bytes
    pub size: u64,
    /// Offsets
    pub offsets: Vec<i64>,
}


/// MEASUREMENT - recordable ECU variable
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lMeasurement {
    /// Measurement name (identifier)
    pub name: String,
    /// Long identifier/description
    pub long_identifier: String,
    /// Data type
    pub data_type: A2lDataType,
    /// Reference to computation method
    pub compu_method: String,
    /// Resolution (smallest change)
    pub resolution: f64,
    /// Accuracy
    pub accuracy: f64,
    /// Lower limit
    pub lower_limit: f64,
    /// Upper limit
    pub upper_limit: f64,
    /// ECU address
    pub ecu_address: Option<u64>,
    /// ECU address extension
    pub ecu_address_extension: Option<u8>,
    /// Byte order override
    pub byte_order: Option<A2lByteOrder>,
    /// Bit mask
    pub bit_mask: Option<u64>,
    /// Array size (for arrays)
    pub array_size: Option<u32>,
    /// Matrix dimensions
    pub matrix_dim: Option<Vec<u32>>,
    /// Display format (C printf style)
    pub format: Option<String>,
    /// Physical unit override
    pub phys_unit: Option<String>,
    /// Display identifier
    pub display_identifier: Option<String>,
    /// Read-write flag
    pub read_write: bool,
    /// Discrete flag (no interpolation)
    pub discrete: bool,
    /// Symbol link (linker map reference)
    pub symbol_link: Option<(String, i64)>,
    /// Function list references
    pub function_list: Vec<String>,
    /// IF_DATA sections
    pub if_data: Vec<A2lIfData>,
    /// Annotations
    pub annotations: Vec<A2lAnnotation>,
    /// Max refresh rate
    pub max_refresh: Option<(u16, u32)>,
    /// Virtual measurement formula inputs
    pub virtual_inputs: Vec<String>,
}

impl A2lMeasurement {
    /// Create a new measurement
    #[must_use]
    pub fn new(name: &str, description: &str, data_type: A2lDataType) -> Self {
        Self {
            name: name.to_string(),
            long_identifier: description.to_string(),
            data_type,
            compu_method: "NO_COMPU_METHOD".to_string(),
            ..Default::default()
        }
    }

    /// Set ECU address
    #[must_use]
    pub fn with_address(mut self, address: u64) -> Self {
        self.ecu_address = Some(address);
        self
    }

    /// Set limits
    #[must_use]
    pub fn with_limits(mut self, lower: f64, upper: f64) -> Self {
        self.lower_limit = lower;
        self.upper_limit = upper;
        self
    }

    /// Set computation method reference
    #[must_use]
    pub fn with_compu_method(mut self, name: &str) -> Self {
        self.compu_method = name.to_string();
        self
    }
}


/// CHARACTERISTIC - tunable parameter
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lCharacteristic {
    /// Characteristic name
    pub name: String,
    /// Long identifier/description
    pub long_identifier: String,
    /// Characteristic type
    pub char_type: CharacteristicType,
    /// ECU address
    pub address: u64,
    /// Record layout reference
    pub record_layout: String,
    /// Maximum difference (for calibration)
    pub max_diff: f64,
    /// Computation method reference
    pub compu_method: String,
    /// Lower limit
    pub lower_limit: f64,
    /// Upper limit
    pub upper_limit: f64,
    /// Extended limits (lower, upper)
    pub extended_limits: Option<(f64, f64)>,
    /// Byte order override
    pub byte_order: Option<A2lByteOrder>,
    /// Bit mask
    pub bit_mask: Option<u64>,
    /// Display format
    pub format: Option<String>,
    /// Physical unit override
    pub phys_unit: Option<String>,
    /// Display identifier
    pub display_identifier: Option<String>,
    /// Read-only flag
    pub read_only: bool,
    /// Discrete flag
    pub discrete: bool,
    /// Guard rails flag
    pub guard_rails: bool,
    /// Step size for calibration
    pub step_size: Option<f64>,
    /// Number of elements (for arrays/strings)
    pub number: Option<u32>,
    /// Matrix dimensions
    pub matrix_dim: Option<Vec<u32>>,
    /// Axis descriptions (for curves/maps)
    pub axis_descr: Vec<A2lAxisDescr>,
    /// Symbol link
    pub symbol_link: Option<(String, i64)>,
    /// Calibration access type
    pub calibration_access: Option<String>,
    /// ECU address extension
    pub ecu_address_extension: Option<u8>,
    /// Function list references
    pub function_list: Vec<String>,
    /// IF_DATA sections
    pub if_data: Vec<A2lIfData>,
    /// Annotations
    pub annotations: Vec<A2lAnnotation>,
    /// Dependent characteristic formula
    pub dependent_characteristic: Option<(String, Vec<String>)>,
    /// Virtual characteristic formula
    pub virtual_characteristic: Option<(String, Vec<String>)>,
    /// Map list (for cuboids)
    pub map_list: Vec<String>,
    /// Comparison quantity reference
    pub comparison_quantity: Option<String>,
    /// Memory segment reference
    pub ref_memory_segment: Option<String>,
    /// Max refresh rate
    pub max_refresh: Option<(u16, u32)>,
}

impl A2lCharacteristic {
    /// Create a new characteristic
    #[must_use]
    pub fn new(
        name: &str,
        description: &str,
        char_type: CharacteristicType,
        address: u64,
        record_layout: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            long_identifier: description.to_string(),
            char_type,
            address,
            record_layout: record_layout.to_string(),
            compu_method: "NO_COMPU_METHOD".to_string(),
            ..Default::default()
        }
    }

    /// Set limits
    #[must_use]
    pub fn with_limits(mut self, lower: f64, upper: f64) -> Self {
        self.lower_limit = lower;
        self.upper_limit = upper;
        self
    }

    /// Set computation method
    #[must_use]
    pub fn with_compu_method(mut self, name: &str) -> Self {
        self.compu_method = name.to_string();
        self
    }

    /// Add axis description
    pub fn add_axis(&mut self, axis: A2lAxisDescr) {
        self.axis_descr.push(axis);
    }
}


/// AXIS_DESCR - axis description within a characteristic
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lAxisDescr {
    /// Axis type
    pub axis_type: AxisType,
    /// Input quantity (measurement reference)
    pub input_quantity: String,
    /// Computation method reference
    pub compu_method: String,
    /// Maximum number of axis points
    pub max_axis_points: u16,
    /// Lower limit
    pub lower_limit: f64,
    /// Upper limit
    pub upper_limit: f64,
    /// Extended limits
    pub extended_limits: Option<(f64, f64)>,
    /// Byte order override
    pub byte_order: Option<A2lByteOrder>,
    /// Deposit mode
    pub deposit: Option<DepositMode>,
    /// Display format
    pub format: Option<String>,
    /// Physical unit override
    pub phys_unit: Option<String>,
    /// Read-only flag
    pub read_only: bool,
    /// Step size
    pub step_size: Option<f64>,
    /// Maximum gradient
    pub max_grad: Option<f64>,
    /// Monotony constraint
    pub monotony: Option<String>,
    /// Axis points reference (for COM_AXIS, RES_AXIS)
    pub axis_pts_ref: Option<String>,
    /// Curve axis reference (for CURVE_AXIS)
    pub curve_axis_ref: Option<String>,
    /// Fix axis parameters (offset, shift, count)
    pub fix_axis_par: Option<(f64, f64, u16)>,
    /// Fix axis parameter distribution (offset, distance, count)
    pub fix_axis_par_dist: Option<(f64, f64, u16)>,
    /// Fix axis parameter list (explicit values)
    pub fix_axis_par_list: Option<Vec<f64>>,
    /// Annotations
    pub annotations: Vec<A2lAnnotation>,
}

/// AXIS_PTS - standalone axis points definition
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lAxisPts {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// ECU address
    pub address: u64,
    /// Input quantity reference
    pub input_quantity: String,
    /// Record layout reference
    pub record_layout: String,
    /// Maximum difference
    pub max_diff: f64,
    /// Computation method reference
    pub compu_method: String,
    /// Maximum number of axis points
    pub max_axis_points: u16,
    /// Lower limit
    pub lower_limit: f64,
    /// Upper limit
    pub upper_limit: f64,
    /// Extended limits
    pub extended_limits: Option<(f64, f64)>,
    /// Byte order override
    pub byte_order: Option<A2lByteOrder>,
    /// Deposit mode
    pub deposit: Option<DepositMode>,
    /// Display format
    pub format: Option<String>,
    /// Physical unit override
    pub phys_unit: Option<String>,
    /// Display identifier
    pub display_identifier: Option<String>,
    /// Read-only flag
    pub read_only: bool,
    /// Step size
    pub step_size: Option<f64>,
    /// Guard rails flag
    pub guard_rails: bool,
    /// Monotony constraint
    pub monotony: Option<String>,
    /// Symbol link
    pub symbol_link: Option<(String, i64)>,
    /// Calibration access
    pub calibration_access: Option<String>,
    /// ECU address extension
    pub ecu_address_extension: Option<u8>,
    /// Function list references
    pub function_list: Vec<String>,
    /// IF_DATA sections
    pub if_data: Vec<A2lIfData>,
    /// Annotations
    pub annotations: Vec<A2lAnnotation>,
    /// Memory segment reference
    pub ref_memory_segment: Option<String>,
    /// Max refresh rate
    pub max_refresh: Option<(u16, u32)>,
}


/// COMPU_METHOD - computation/conversion method
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lCompuMethod {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Conversion type
    pub conversion_type: CompuMethodType,
    /// Display format (C printf style)
    pub format: String,
    /// Unit string
    pub unit: String,
    /// Coefficients for LINEAR (a, b) where y = ax + b
    pub coeffs_linear: Option<(f64, f64)>,
    /// Coefficients for RAT_FUNC (a, b, c, d, e, f)
    /// Physical = (a*x^2 + b*x + c) / (d*x^2 + e*x + f)
    pub coeffs: Option<(f64, f64, f64, f64, f64, f64)>,
    /// Reference to computation table
    pub compu_tab_ref: Option<String>,
    /// Formula string
    pub formula: Option<String>,
    /// Inverse formula string
    pub formula_inv: Option<String>,
    /// Reference to unit definition
    pub ref_unit: Option<String>,
    /// Status string reference (for split numeric/verbal)
    pub status_string_ref: Option<String>,
}

impl A2lCompuMethod {
    /// Create a new computation method
    #[must_use]
    pub fn new(name: &str, description: &str, conv_type: CompuMethodType) -> Self {
        Self {
            name: name.to_string(),
            long_identifier: description.to_string(),
            conversion_type: conv_type,
            format: "%.3f".to_string(),
            unit: String::new(),
            ..Default::default()
        }
    }

    /// Create an identical (no conversion) method
    #[must_use]
    pub fn identical(name: &str, description: &str) -> Self {
        Self::new(name, description, CompuMethodType::Identical)
    }

    /// Create a linear conversion method
    #[must_use]
    pub fn linear(name: &str, description: &str, slope: f64, offset: f64) -> Self {
        let mut cm = Self::new(name, description, CompuMethodType::Linear);
        cm.coeffs_linear = Some((slope, offset));
        cm
    }

    /// Set unit
    #[must_use]
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// Set format
    #[must_use]
    pub fn with_format(mut self, format: &str) -> Self {
        self.format = format.to_string();
        self
    }

    /// Convert internal value to physical value
    #[must_use]
    pub fn convert_to_physical(&self, internal: f64) -> f64 {
        match self.conversion_type {
            CompuMethodType::Identical => internal,
            CompuMethodType::Linear => {
                if let Some((a, b)) = self.coeffs_linear {
                    a * internal + b
                } else {
                    internal
                }
            }
            CompuMethodType::RatFunc => {
                if let Some((a, b, c, d, e, f)) = self.coeffs {
                    // Note: RAT_FUNC is physical->internal, so we need inverse
                    // For now, return internal (proper inverse requires solving)
                    let denom = d * internal * internal + e * internal + f;
                    if denom.abs() > f64::EPSILON {
                        (a * internal * internal + b * internal + c) / denom
                    } else {
                        internal
                    }
                } else {
                    internal
                }
            }
            _ => internal, // Tables and formulas need lookup/evaluation
        }
    }
}


/// COMPU_TAB - computation table (numeric)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lCompuTab {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Conversion type (TAB_INTP or TAB_NOINTP)
    pub conversion_type: CompuMethodType,
    /// Number of value pairs
    pub number_value_pairs: u16,
    /// Value pairs (input, output)
    pub values: Vec<(f64, f64)>,
    /// Default value (for out-of-range)
    pub default_value: Option<f64>,
    /// Default value numeric
    pub default_value_numeric: Option<f64>,
}

/// COMPU_VTAB - verbal computation table (enumeration)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lCompuVtab {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Conversion type (TAB_VERB)
    pub conversion_type: CompuMethodType,
    /// Number of value pairs
    pub number_value_pairs: u16,
    /// Value pairs (numeric value, string)
    pub values: Vec<(i64, String)>,
    /// Default value string
    pub default_value: Option<String>,
}

/// COMPU_VTAB_RANGE - verbal table with ranges
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lCompuVtabRange {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Number of value triples
    pub number_value_triples: u16,
    /// Value triples (lower, upper, string)
    pub values: Vec<(f64, f64, String)>,
    /// Default value string
    pub default_value: Option<String>,
}

/// RECORD_LAYOUT - memory layout description
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lRecordLayout {
    /// Name
    pub name: String,
    /// Function values specification
    pub fnc_values: Option<A2lFncValues>,
    /// Identification field
    pub identification: Option<(u16, A2lDataType)>,
    /// Axis points X specification
    pub axis_pts_x: Option<A2lAxisPtsLayout>,
    /// Axis points Y specification
    pub axis_pts_y: Option<A2lAxisPtsLayout>,
    /// Axis points Z specification
    pub axis_pts_z: Option<A2lAxisPtsLayout>,
    /// Axis points 4 specification
    pub axis_pts_4: Option<A2lAxisPtsLayout>,
    /// Axis points 5 specification
    pub axis_pts_5: Option<A2lAxisPtsLayout>,
    /// Number of axis points X
    pub no_axis_pts_x: Option<(u16, A2lDataType)>,
    /// Number of axis points Y
    pub no_axis_pts_y: Option<(u16, A2lDataType)>,
    /// Number of axis points Z
    pub no_axis_pts_z: Option<(u16, A2lDataType)>,
    /// Fixed number of axis points X
    pub fix_no_axis_pts_x: Option<u16>,
    /// Fixed number of axis points Y
    pub fix_no_axis_pts_y: Option<u16>,
    /// Fixed number of axis points Z
    pub fix_no_axis_pts_z: Option<u16>,
    /// Source address X
    pub src_addr_x: Option<(u16, A2lDataType)>,
    /// Source address Y
    pub src_addr_y: Option<(u16, A2lDataType)>,
    /// Source address Z
    pub src_addr_z: Option<(u16, A2lDataType)>,
    /// RIP address W (result of interpolation)
    pub rip_addr_w: Option<(u16, A2lDataType)>,
    /// RIP address X
    pub rip_addr_x: Option<(u16, A2lDataType)>,
    /// RIP address Y
    pub rip_addr_y: Option<(u16, A2lDataType)>,
    /// RIP address Z
    pub rip_addr_z: Option<(u16, A2lDataType)>,
    /// Shift operation X
    pub shift_op_x: Option<(u16, A2lDataType)>,
    /// Shift operation Y
    pub shift_op_y: Option<(u16, A2lDataType)>,
    /// Shift operation Z
    pub shift_op_z: Option<(u16, A2lDataType)>,
    /// Offset X
    pub offset_x: Option<(u16, A2lDataType)>,
    /// Offset Y
    pub offset_y: Option<(u16, A2lDataType)>,
    /// Offset Z
    pub offset_z: Option<(u16, A2lDataType)>,
    /// Distance operation X
    pub dist_op_x: Option<(u16, A2lDataType)>,
    /// Distance operation Y
    pub dist_op_y: Option<(u16, A2lDataType)>,
    /// Distance operation Z
    pub dist_op_z: Option<(u16, A2lDataType)>,
    /// Axis rescale X
    pub axis_rescale_x: Option<A2lAxisRescale>,
    /// Reserved positions
    pub reserved: Vec<(u16, A2lDataType)>,
    /// Static record layout flag
    pub static_record_layout: bool,
    /// Alignment for byte data
    pub alignment_byte: Option<u16>,
    /// Alignment for word data
    pub alignment_word: Option<u16>,
    /// Alignment for long data
    pub alignment_long: Option<u16>,
    /// Alignment for float32 data
    pub alignment_float32: Option<u16>,
    /// Alignment for float64 data
    pub alignment_float64: Option<u16>,
    /// Alignment for int64 data
    pub alignment_int64: Option<u16>,
}


/// FNC_VALUES specification in RECORD_LAYOUT
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lFncValues {
    /// Position
    pub position: u16,
    /// Data type
    pub data_type: A2lDataType,
    /// Index mode (COLUMN_DIR, ROW_DIR, ALTERNATE_WITH_X, etc.)
    pub index_mode: String,
    /// Address type (DIRECT, PBYTE, PWORD, PLONG, etc.)
    pub address_type: String,
}

/// Axis points layout specification
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lAxisPtsLayout {
    /// Position
    pub position: u16,
    /// Data type
    pub data_type: A2lDataType,
    /// Index increment
    pub index_incr: String,
    /// Address type
    pub address_type: String,
}

/// Axis rescale specification
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lAxisRescale {
    /// Position
    pub position: u16,
    /// Data type
    pub data_type: A2lDataType,
    /// Max number of rescale pairs
    pub max_number_of_rescale_pairs: u16,
    /// Index increment
    pub index_incr: String,
    /// Address type
    pub address_type: String,
}

/// FUNCTION - grouping of measurements and characteristics
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lFunction {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Version
    pub version: Option<String>,
    /// Input measurements
    pub in_measurement: Vec<String>,
    /// Output measurements
    pub out_measurement: Vec<String>,
    /// Local measurements
    pub loc_measurement: Vec<String>,
    /// Defined characteristics
    pub def_characteristic: Vec<String>,
    /// Referenced characteristics
    pub ref_characteristic: Vec<String>,
    /// Sub-functions
    pub sub_function: Vec<String>,
    /// Annotations
    pub annotations: Vec<A2lAnnotation>,
    /// IF_DATA sections
    pub if_data: Vec<A2lIfData>,
}

/// GROUP - grouping for selection lists
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lGroup {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Root flag
    pub root: bool,
    /// Referenced characteristics
    pub ref_characteristic: Vec<String>,
    /// Referenced measurements
    pub ref_measurement: Vec<String>,
    /// Sub-groups
    pub sub_group: Vec<String>,
    /// Annotations
    pub annotations: Vec<A2lAnnotation>,
    /// IF_DATA sections
    pub if_data: Vec<A2lIfData>,
}

/// UNIT - physical unit definition
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lUnit {
    /// Name
    pub name: String,
    /// Long identifier
    pub long_identifier: String,
    /// Display string
    pub display: String,
    /// Unit type (DERIVED, EXTENDED_SI)
    pub unit_type: String,
    /// Reference unit (for derived units)
    pub ref_unit: Option<String>,
    /// SI exponents (length, mass, time, current, temperature, amount, luminosity)
    pub si_exponents: Option<(i8, i8, i8, i8, i8, i8, i8)>,
    /// Unit conversion (slope, offset)
    pub unit_conversion: Option<(f64, f64)>,
}

/// IF_DATA - interface data section
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lIfData {
    /// Protocol name (e.g., "XCP", "XCPplus", "ASAP1B_CAN")
    pub name: String,
    /// Raw content (unparsed)
    pub content: String,
}

/// Annotation
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct A2lAnnotation {
    /// Label
    pub label: Option<String>,
    /// Origin
    pub origin: Option<String>,
    /// Text lines
    pub text: Vec<String>,
}


// ============================================================================
// Parser Implementation
// ============================================================================

/// A2L file parser
pub struct A2lParser;

impl A2lParser {
    /// Parse an A2L file from a string
    ///
    /// # Arguments
    ///
    /// * `input` - A2L file content as a string
    ///
    /// # Returns
    ///
    /// A parsed `A2lDatabase` or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the A2L file is malformed or contains invalid data.
    pub fn parse(input: &str) -> Result<A2lDatabase> {
        let mut parser = A2lParserState::new(input);
        parser.parse()
    }
}

/// Internal parser state
struct A2lParserState<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> A2lParserState<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
        }
    }

    fn parse(&mut self) -> Result<A2lDatabase> {
        let mut db = A2lDatabase::new();

        self.skip_whitespace_and_comments();

        // Parse ASAP2_VERSION
        if self.peek_keyword("ASAP2_VERSION") {
            db.asap2_version = self.parse_asap2_version()?;
            self.skip_whitespace_and_comments();
        }

        // Parse A2ML_VERSION (optional)
        if self.peek_keyword("A2ML_VERSION") {
            db.a2ml_version = Some(self.parse_a2ml_version()?);
            self.skip_whitespace_and_comments();
        }

        // Parse PROJECT
        if self.peek_keyword("/begin") {
            self.expect_keyword("/begin")?;
            self.skip_whitespace_and_comments();
            self.expect_keyword("PROJECT")?;
            db.project = self.parse_project()?;
        }

        Ok(db)
    }

    fn parse_asap2_version(&mut self) -> Result<Asap2Version> {
        self.expect_keyword("ASAP2_VERSION")?;
        self.skip_whitespace_and_comments();
        let major = self.parse_u16()?;
        self.skip_whitespace_and_comments();
        let minor = self.parse_u16()?;
        Ok(Asap2Version::new(major, minor))
    }

    fn parse_a2ml_version(&mut self) -> Result<Asap2Version> {
        self.expect_keyword("A2ML_VERSION")?;
        self.skip_whitespace_and_comments();
        let major = self.parse_u16()?;
        self.skip_whitespace_and_comments();
        let minor = self.parse_u16()?;
        Ok(Asap2Version::new(major, minor))
    }

    fn parse_project(&mut self) -> Result<A2lProject> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut project = A2lProject {
            name,
            long_identifier,
            header: None,
            modules: Vec::new(),
        };

        // Parse project contents
        loop {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("/end") {
                break;
            }
            if self.at_end() {
                break;
            }
            if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();

                if self.peek_keyword("HEADER") {
                    self.expect_keyword("HEADER")?;
                    project.header = Some(self.parse_header()?);
                } else if self.peek_keyword("MODULE") {
                    self.expect_keyword("MODULE")?;
                    project.modules.push(self.parse_module()?);
                } else {
                    // Skip unknown block
                    self.skip_block()?;
                }
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("PROJECT")?;

        Ok(project)
    }


    fn parse_header(&mut self) -> Result<A2lHeader> {
        self.skip_whitespace_and_comments();
        let comment = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut header = A2lHeader {
            comment,
            project_no: None,
            version: None,
        };

        loop {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("/end") || self.at_end() {
                break;
            }
            if self.peek_keyword("PROJECT_NO") {
                self.expect_keyword("PROJECT_NO")?;
                self.skip_whitespace_and_comments();
                header.project_no = Some(self.parse_identifier()?);
            } else if self.peek_keyword("VERSION") {
                self.expect_keyword("VERSION")?;
                self.skip_whitespace_and_comments();
                header.version = Some(self.parse_string()?);
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("HEADER")?;

        Ok(header)
    }

    fn parse_module(&mut self) -> Result<A2lModule> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut module = A2lModule {
            name,
            long_identifier,
            ..Default::default()
        };

        loop {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("/end") {
                break;
            }
            if self.at_end() {
                break;
            }
            if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();

                if self.peek_keyword("MOD_COMMON") {
                    self.expect_keyword("MOD_COMMON")?;
                    module.mod_common = Some(self.parse_mod_common()?);
                } else if self.peek_keyword("MOD_PAR") {
                    self.expect_keyword("MOD_PAR")?;
                    module.mod_par = Some(self.parse_mod_par()?);
                } else if self.peek_keyword("MEASUREMENT") {
                    self.expect_keyword("MEASUREMENT")?;
                    module.measurements.push(self.parse_measurement()?);
                } else if self.peek_keyword("CHARACTERISTIC") {
                    self.expect_keyword("CHARACTERISTIC")?;
                    module.characteristics.push(self.parse_characteristic()?);
                } else if self.peek_keyword("COMPU_METHOD") {
                    self.expect_keyword("COMPU_METHOD")?;
                    module.compu_methods.push(self.parse_compu_method()?);
                } else if self.peek_keyword("COMPU_TAB") {
                    self.expect_keyword("COMPU_TAB")?;
                    module.compu_tabs.push(self.parse_compu_tab()?);
                } else if self.peek_keyword("COMPU_VTAB") {
                    self.expect_keyword("COMPU_VTAB")?;
                    module.compu_vtabs.push(self.parse_compu_vtab()?);
                } else if self.peek_keyword("RECORD_LAYOUT") {
                    self.expect_keyword("RECORD_LAYOUT")?;
                    module.record_layouts.push(self.parse_record_layout()?);
                } else if self.peek_keyword("AXIS_PTS") {
                    self.expect_keyword("AXIS_PTS")?;
                    module.axis_pts.push(self.parse_axis_pts()?);
                } else if self.peek_keyword("FUNCTION") {
                    self.expect_keyword("FUNCTION")?;
                    module.functions.push(self.parse_function()?);
                } else if self.peek_keyword("GROUP") {
                    self.expect_keyword("GROUP")?;
                    module.groups.push(self.parse_group()?);
                } else if self.peek_keyword("UNIT") {
                    self.expect_keyword("UNIT")?;
                    module.units.push(self.parse_unit()?);
                } else if self.peek_keyword("IF_DATA") {
                    self.expect_keyword("IF_DATA")?;
                    module.if_data.push(self.parse_if_data()?);
                } else if self.peek_keyword("A2ML") {
                    self.expect_keyword("A2ML")?;
                    self.skip_block()?; // Skip A2ML for now
                } else {
                    self.skip_block()?;
                }
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("MODULE")?;

        Ok(module)
    }


    fn parse_mod_common(&mut self) -> Result<A2lModCommon> {
        self.skip_whitespace_and_comments();
        let comment = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut mod_common = A2lModCommon {
            comment,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("BYTE_ORDER") {
                self.expect_keyword("BYTE_ORDER")?;
                self.skip_whitespace_and_comments();
                let order = self.parse_identifier()?;
                mod_common.byte_order = Some(if order == "MSB_FIRST" || order == "MSB_LAST" {
                    A2lByteOrder::BigEndian
                } else {
                    A2lByteOrder::LittleEndian
                });
            } else if self.peek_keyword("DATA_SIZE") {
                self.expect_keyword("DATA_SIZE")?;
                self.skip_whitespace_and_comments();
                mod_common.data_size = Some(self.parse_u16()?);
            } else if self.peek_keyword("DEPOSIT") {
                self.expect_keyword("DEPOSIT")?;
                self.skip_whitespace_and_comments();
                let deposit = self.parse_identifier()?;
                mod_common.deposit = Some(if deposit == "DIFFERENCE" {
                    DepositMode::Difference
                } else {
                    DepositMode::Absolute
                });
            } else if self.peek_keyword("ALIGNMENT_BYTE") {
                self.expect_keyword("ALIGNMENT_BYTE")?;
                self.skip_whitespace_and_comments();
                mod_common.alignment_byte = Some(self.parse_u16()?);
            } else if self.peek_keyword("ALIGNMENT_WORD") {
                self.expect_keyword("ALIGNMENT_WORD")?;
                self.skip_whitespace_and_comments();
                mod_common.alignment_word = Some(self.parse_u16()?);
            } else if self.peek_keyword("ALIGNMENT_LONG") {
                self.expect_keyword("ALIGNMENT_LONG")?;
                self.skip_whitespace_and_comments();
                mod_common.alignment_long = Some(self.parse_u16()?);
            } else if self.peek_keyword("ALIGNMENT_FLOAT32_IEEE") {
                self.expect_keyword("ALIGNMENT_FLOAT32_IEEE")?;
                self.skip_whitespace_and_comments();
                mod_common.alignment_float32 = Some(self.parse_u16()?);
            } else if self.peek_keyword("ALIGNMENT_FLOAT64_IEEE") {
                self.expect_keyword("ALIGNMENT_FLOAT64_IEEE")?;
                self.skip_whitespace_and_comments();
                mod_common.alignment_float64 = Some(self.parse_u16()?);
            } else if self.peek_keyword("ALIGNMENT_INT64") {
                self.expect_keyword("ALIGNMENT_INT64")?;
                self.skip_whitespace_and_comments();
                mod_common.alignment_int64 = Some(self.parse_u16()?);
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("MOD_COMMON")?;

        Ok(mod_common)
    }

    fn parse_mod_par(&mut self) -> Result<A2lModPar> {
        self.skip_whitespace_and_comments();
        let comment = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut mod_par = A2lModPar {
            comment,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("CPU_TYPE") {
                self.expect_keyword("CPU_TYPE")?;
                self.skip_whitespace_and_comments();
                mod_par.cpu_type = Some(self.parse_string()?);
            } else if self.peek_keyword("CUSTOMER") {
                self.expect_keyword("CUSTOMER")?;
                self.skip_whitespace_and_comments();
                mod_par.customer = Some(self.parse_string()?);
            } else if self.peek_keyword("VERSION") {
                self.expect_keyword("VERSION")?;
                self.skip_whitespace_and_comments();
                mod_par.version = Some(self.parse_string()?);
            } else if self.peek_keyword("EPK") {
                self.expect_keyword("EPK")?;
                self.skip_whitespace_and_comments();
                mod_par.epk = Some(self.parse_string()?);
            } else if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();
                if self.peek_keyword("MEMORY_SEGMENT") {
                    self.expect_keyword("MEMORY_SEGMENT")?;
                    mod_par.memory_segments.push(self.parse_memory_segment()?);
                } else {
                    self.skip_block()?;
                }
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("MOD_PAR")?;

        Ok(mod_par)
    }

    fn parse_memory_segment(&mut self) -> Result<A2lMemorySegment> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let prg_type = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let memory_type = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let attribute = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let address = self.parse_hex_or_dec()?;
        self.skip_whitespace_and_comments();
        let size = self.parse_hex_or_dec()?;
        self.skip_whitespace_and_comments();

        // Skip to end
        while !self.peek_keyword("/end") {
            if self.at_end() {
                break;
            }
            self.advance_char();
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("MEMORY_SEGMENT")?;

        Ok(A2lMemorySegment {
            name,
            long_identifier,
            prg_type,
            memory_type,
            attribute,
            address,
            size,
            offsets: Vec::new(),
        })
    }


    fn parse_measurement(&mut self) -> Result<A2lMeasurement> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let data_type_str = self.parse_identifier()?;
        let data_type = A2lDataType::from_str(&data_type_str).unwrap_or_default();
        self.skip_whitespace_and_comments();
        let compu_method = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let resolution = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let accuracy = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let lower_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let upper_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();

        let mut measurement = A2lMeasurement {
            name,
            long_identifier,
            data_type,
            compu_method,
            resolution,
            accuracy,
            lower_limit,
            upper_limit,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("ECU_ADDRESS") {
                self.expect_keyword("ECU_ADDRESS")?;
                self.skip_whitespace_and_comments();
                measurement.ecu_address = Some(self.parse_hex_or_dec()?);
            } else if self.peek_keyword("ECU_ADDRESS_EXTENSION") {
                self.expect_keyword("ECU_ADDRESS_EXTENSION")?;
                self.skip_whitespace_and_comments();
                measurement.ecu_address_extension = Some(self.parse_u16()? as u8);
            } else if self.peek_keyword("FORMAT") {
                self.expect_keyword("FORMAT")?;
                self.skip_whitespace_and_comments();
                measurement.format = Some(self.parse_string()?);
            } else if self.peek_keyword("PHYS_UNIT") {
                self.expect_keyword("PHYS_UNIT")?;
                self.skip_whitespace_and_comments();
                measurement.phys_unit = Some(self.parse_string()?);
            } else if self.peek_keyword("BIT_MASK") {
                self.expect_keyword("BIT_MASK")?;
                self.skip_whitespace_and_comments();
                measurement.bit_mask = Some(self.parse_hex_or_dec()?);
            } else if self.peek_keyword("BYTE_ORDER") {
                self.expect_keyword("BYTE_ORDER")?;
                self.skip_whitespace_and_comments();
                let order = self.parse_identifier()?;
                measurement.byte_order = Some(if order == "MSB_FIRST" || order == "MSB_LAST" {
                    A2lByteOrder::BigEndian
                } else {
                    A2lByteOrder::LittleEndian
                });
            } else if self.peek_keyword("DISCRETE") {
                self.expect_keyword("DISCRETE")?;
                measurement.discrete = true;
            } else if self.peek_keyword("READ_WRITE") {
                self.expect_keyword("READ_WRITE")?;
                measurement.read_write = true;
            } else if self.peek_keyword("DISPLAY_IDENTIFIER") {
                self.expect_keyword("DISPLAY_IDENTIFIER")?;
                self.skip_whitespace_and_comments();
                measurement.display_identifier = Some(self.parse_identifier()?);
            } else if self.peek_keyword("MATRIX_DIM") {
                self.expect_keyword("MATRIX_DIM")?;
                self.skip_whitespace_and_comments();
                let mut dims = Vec::new();
                while let Ok(dim) = self.parse_u32() {
                    dims.push(dim);
                    self.skip_whitespace_and_comments();
                    if self.peek_keyword("/end") || self.peek_keyword("ECU_ADDRESS") {
                        break;
                    }
                }
                measurement.matrix_dim = Some(dims);
            } else if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();
                if self.peek_keyword("IF_DATA") {
                    self.expect_keyword("IF_DATA")?;
                    measurement.if_data.push(self.parse_if_data()?);
                } else if self.peek_keyword("ANNOTATION") {
                    self.expect_keyword("ANNOTATION")?;
                    measurement.annotations.push(self.parse_annotation()?);
                } else {
                    self.skip_block()?;
                }
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("MEASUREMENT")?;

        Ok(measurement)
    }


    fn parse_characteristic(&mut self) -> Result<A2lCharacteristic> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let char_type_str = self.parse_identifier()?;
        let char_type = CharacteristicType::from_str(&char_type_str).unwrap_or_default();
        self.skip_whitespace_and_comments();
        let address = self.parse_hex_or_dec()?;
        self.skip_whitespace_and_comments();
        let record_layout = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let max_diff = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let compu_method = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let lower_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let upper_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();

        let mut characteristic = A2lCharacteristic {
            name,
            long_identifier,
            char_type,
            address,
            record_layout,
            max_diff,
            compu_method,
            lower_limit,
            upper_limit,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("FORMAT") {
                self.expect_keyword("FORMAT")?;
                self.skip_whitespace_and_comments();
                characteristic.format = Some(self.parse_string()?);
            } else if self.peek_keyword("PHYS_UNIT") {
                self.expect_keyword("PHYS_UNIT")?;
                self.skip_whitespace_and_comments();
                characteristic.phys_unit = Some(self.parse_string()?);
            } else if self.peek_keyword("EXTENDED_LIMITS") {
                self.expect_keyword("EXTENDED_LIMITS")?;
                self.skip_whitespace_and_comments();
                let lower = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let upper = self.parse_f64()?;
                characteristic.extended_limits = Some((lower, upper));
            } else if self.peek_keyword("READ_ONLY") {
                self.expect_keyword("READ_ONLY")?;
                characteristic.read_only = true;
            } else if self.peek_keyword("DISCRETE") {
                self.expect_keyword("DISCRETE")?;
                characteristic.discrete = true;
            } else if self.peek_keyword("GUARD_RAILS") {
                self.expect_keyword("GUARD_RAILS")?;
                characteristic.guard_rails = true;
            } else if self.peek_keyword("STEP_SIZE") {
                self.expect_keyword("STEP_SIZE")?;
                self.skip_whitespace_and_comments();
                characteristic.step_size = Some(self.parse_f64()?);
            } else if self.peek_keyword("NUMBER") {
                self.expect_keyword("NUMBER")?;
                self.skip_whitespace_and_comments();
                characteristic.number = Some(self.parse_u32()?);
            } else if self.peek_keyword("MATRIX_DIM") {
                self.expect_keyword("MATRIX_DIM")?;
                self.skip_whitespace_and_comments();
                let mut dims = Vec::new();
                while let Ok(dim) = self.parse_u32() {
                    dims.push(dim);
                    self.skip_whitespace_and_comments();
                    if self.peek_keyword("/end") || self.peek_keyword("/begin") {
                        break;
                    }
                }
                characteristic.matrix_dim = Some(dims);
            } else if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();
                if self.peek_keyword("AXIS_DESCR") {
                    self.expect_keyword("AXIS_DESCR")?;
                    characteristic.axis_descr.push(self.parse_axis_descr()?);
                } else if self.peek_keyword("IF_DATA") {
                    self.expect_keyword("IF_DATA")?;
                    characteristic.if_data.push(self.parse_if_data()?);
                } else if self.peek_keyword("ANNOTATION") {
                    self.expect_keyword("ANNOTATION")?;
                    characteristic.annotations.push(self.parse_annotation()?);
                } else {
                    self.skip_block()?;
                }
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("CHARACTERISTIC")?;

        Ok(characteristic)
    }

    fn parse_axis_descr(&mut self) -> Result<A2lAxisDescr> {
        self.skip_whitespace_and_comments();
        let axis_type_str = self.parse_identifier()?;
        let axis_type = AxisType::from_str(&axis_type_str).unwrap_or_default();
        self.skip_whitespace_and_comments();
        let input_quantity = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let compu_method = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let max_axis_points = self.parse_u16()?;
        self.skip_whitespace_and_comments();
        let lower_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let upper_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();

        let mut axis_descr = A2lAxisDescr {
            axis_type,
            input_quantity,
            compu_method,
            max_axis_points,
            lower_limit,
            upper_limit,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("EXTENDED_LIMITS") {
                self.expect_keyword("EXTENDED_LIMITS")?;
                self.skip_whitespace_and_comments();
                let lower = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let upper = self.parse_f64()?;
                axis_descr.extended_limits = Some((lower, upper));
            } else if self.peek_keyword("READ_ONLY") {
                self.expect_keyword("READ_ONLY")?;
                axis_descr.read_only = true;
            } else if self.peek_keyword("AXIS_PTS_REF") {
                self.expect_keyword("AXIS_PTS_REF")?;
                self.skip_whitespace_and_comments();
                axis_descr.axis_pts_ref = Some(self.parse_identifier()?);
            } else if self.peek_keyword("CURVE_AXIS_REF") {
                self.expect_keyword("CURVE_AXIS_REF")?;
                self.skip_whitespace_and_comments();
                axis_descr.curve_axis_ref = Some(self.parse_identifier()?);
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("AXIS_DESCR")?;

        Ok(axis_descr)
    }


    fn parse_compu_method(&mut self) -> Result<A2lCompuMethod> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let conv_type_str = self.parse_identifier()?;
        let conversion_type = CompuMethodType::from_str(&conv_type_str).unwrap_or_default();
        self.skip_whitespace_and_comments();
        let format = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let unit = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut compu_method = A2lCompuMethod {
            name,
            long_identifier,
            conversion_type,
            format,
            unit,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("COEFFS_LINEAR") {
                self.expect_keyword("COEFFS_LINEAR")?;
                self.skip_whitespace_and_comments();
                let a = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let b = self.parse_f64()?;
                compu_method.coeffs_linear = Some((a, b));
            } else if self.peek_keyword("COEFFS") {
                self.expect_keyword("COEFFS")?;
                self.skip_whitespace_and_comments();
                let coeff_a = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let coeff_b = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let coeff_c = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let coeff_d = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let coeff_e = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let coeff_f = self.parse_f64()?;
                compu_method.coeffs = Some((coeff_a, coeff_b, coeff_c, coeff_d, coeff_e, coeff_f));
            } else if self.peek_keyword("COMPU_TAB_REF") {
                self.expect_keyword("COMPU_TAB_REF")?;
                self.skip_whitespace_and_comments();
                compu_method.compu_tab_ref = Some(self.parse_identifier()?);
            } else if self.peek_keyword("REF_UNIT") {
                self.expect_keyword("REF_UNIT")?;
                self.skip_whitespace_and_comments();
                compu_method.ref_unit = Some(self.parse_identifier()?);
            } else if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();
                if self.peek_keyword("FORMULA") {
                    self.expect_keyword("FORMULA")?;
                    self.skip_whitespace_and_comments();
                    compu_method.formula = Some(self.parse_string()?);
                    // Skip to end of FORMULA block
                    while !self.peek_keyword("/end") {
                        if self.at_end() {
                            break;
                        }
                        self.advance_char();
                    }
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("FORMULA")?;
                } else {
                    self.skip_block()?;
                }
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("COMPU_METHOD")?;

        Ok(compu_method)
    }

    fn parse_compu_tab(&mut self) -> Result<A2lCompuTab> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let conv_type_str = self.parse_identifier()?;
        let conversion_type = CompuMethodType::from_str(&conv_type_str).unwrap_or_default();
        self.skip_whitespace_and_comments();
        let number_value_pairs = self.parse_u16()?;
        self.skip_whitespace_and_comments();

        let mut values = Vec::new();
        for _ in 0..number_value_pairs {
            let input = self.parse_f64()?;
            self.skip_whitespace_and_comments();
            let output = self.parse_f64()?;
            self.skip_whitespace_and_comments();
            values.push((input, output));
        }

        let mut compu_tab = A2lCompuTab {
            name,
            long_identifier,
            conversion_type,
            number_value_pairs,
            values,
            default_value: None,
            default_value_numeric: None,
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("DEFAULT_VALUE") {
                self.expect_keyword("DEFAULT_VALUE")?;
                self.skip_whitespace_and_comments();
                compu_tab.default_value = Some(self.parse_f64()?);
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("COMPU_TAB")?;

        Ok(compu_tab)
    }

    fn parse_compu_vtab(&mut self) -> Result<A2lCompuVtab> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let conv_type_str = self.parse_identifier()?;
        let conversion_type = CompuMethodType::from_str(&conv_type_str).unwrap_or_default();
        self.skip_whitespace_and_comments();
        let number_value_pairs = self.parse_u16()?;
        self.skip_whitespace_and_comments();

        let mut values = Vec::new();
        for _ in 0..number_value_pairs {
            let input = self.parse_i64()?;
            self.skip_whitespace_and_comments();
            let output = self.parse_string()?;
            self.skip_whitespace_and_comments();
            values.push((input, output));
        }

        let mut compu_vtab = A2lCompuVtab {
            name,
            long_identifier,
            conversion_type,
            number_value_pairs,
            values,
            default_value: None,
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("DEFAULT_VALUE") {
                self.expect_keyword("DEFAULT_VALUE")?;
                self.skip_whitespace_and_comments();
                compu_vtab.default_value = Some(self.parse_string()?);
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("COMPU_VTAB")?;

        Ok(compu_vtab)
    }


    fn parse_record_layout(&mut self) -> Result<A2lRecordLayout> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();

        let mut record_layout = A2lRecordLayout {
            name,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("FNC_VALUES") {
                self.expect_keyword("FNC_VALUES")?;
                self.skip_whitespace_and_comments();
                let position = self.parse_u16()?;
                self.skip_whitespace_and_comments();
                let data_type_str = self.parse_identifier()?;
                let data_type = A2lDataType::from_str(&data_type_str).unwrap_or_default();
                self.skip_whitespace_and_comments();
                let index_mode = self.parse_identifier()?;
                self.skip_whitespace_and_comments();
                let address_type = self.parse_identifier()?;
                record_layout.fnc_values = Some(A2lFncValues {
                    position,
                    data_type,
                    index_mode,
                    address_type,
                });
            } else if self.peek_keyword("STATIC_RECORD_LAYOUT") {
                self.expect_keyword("STATIC_RECORD_LAYOUT")?;
                record_layout.static_record_layout = true;
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("RECORD_LAYOUT")?;

        Ok(record_layout)
    }

    fn parse_axis_pts(&mut self) -> Result<A2lAxisPts> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let address = self.parse_hex_or_dec()?;
        self.skip_whitespace_and_comments();
        let input_quantity = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let record_layout = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let max_diff = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let compu_method = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let max_axis_points = self.parse_u16()?;
        self.skip_whitespace_and_comments();
        let lower_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();
        let upper_limit = self.parse_f64()?;
        self.skip_whitespace_and_comments();

        let mut axis_pts = A2lAxisPts {
            name,
            long_identifier,
            address,
            input_quantity,
            record_layout,
            max_diff,
            compu_method,
            max_axis_points,
            lower_limit,
            upper_limit,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("READ_ONLY") {
                self.expect_keyword("READ_ONLY")?;
                axis_pts.read_only = true;
            } else if self.peek_keyword("GUARD_RAILS") {
                self.expect_keyword("GUARD_RAILS")?;
                axis_pts.guard_rails = true;
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("AXIS_PTS")?;

        Ok(axis_pts)
    }

    fn parse_function(&mut self) -> Result<A2lFunction> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut function = A2lFunction {
            name,
            long_identifier,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("FUNCTION_VERSION") {
                self.expect_keyword("FUNCTION_VERSION")?;
                self.skip_whitespace_and_comments();
                function.version = Some(self.parse_string()?);
            } else if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();
                if self.peek_keyword("DEF_CHARACTERISTIC") {
                    self.expect_keyword("DEF_CHARACTERISTIC")?;
                    function.def_characteristic = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("DEF_CHARACTERISTIC")?;
                } else if self.peek_keyword("REF_CHARACTERISTIC") {
                    self.expect_keyword("REF_CHARACTERISTIC")?;
                    function.ref_characteristic = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("REF_CHARACTERISTIC")?;
                } else if self.peek_keyword("IN_MEASUREMENT") {
                    self.expect_keyword("IN_MEASUREMENT")?;
                    function.in_measurement = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("IN_MEASUREMENT")?;
                } else if self.peek_keyword("OUT_MEASUREMENT") {
                    self.expect_keyword("OUT_MEASUREMENT")?;
                    function.out_measurement = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("OUT_MEASUREMENT")?;
                } else if self.peek_keyword("LOC_MEASUREMENT") {
                    self.expect_keyword("LOC_MEASUREMENT")?;
                    function.loc_measurement = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("LOC_MEASUREMENT")?;
                } else if self.peek_keyword("SUB_FUNCTION") {
                    self.expect_keyword("SUB_FUNCTION")?;
                    function.sub_function = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("SUB_FUNCTION")?;
                } else {
                    self.skip_block()?;
                }
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("FUNCTION")?;

        Ok(function)
    }


    fn parse_group(&mut self) -> Result<A2lGroup> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();

        let mut group = A2lGroup {
            name,
            long_identifier,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("ROOT") {
                self.expect_keyword("ROOT")?;
                group.root = true;
            } else if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();
                if self.peek_keyword("REF_CHARACTERISTIC") {
                    self.expect_keyword("REF_CHARACTERISTIC")?;
                    group.ref_characteristic = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("REF_CHARACTERISTIC")?;
                } else if self.peek_keyword("REF_MEASUREMENT") {
                    self.expect_keyword("REF_MEASUREMENT")?;
                    group.ref_measurement = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("REF_MEASUREMENT")?;
                } else if self.peek_keyword("SUB_GROUP") {
                    self.expect_keyword("SUB_GROUP")?;
                    group.sub_group = self.parse_identifier_list()?;
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("SUB_GROUP")?;
                } else {
                    self.skip_block()?;
                }
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("GROUP")?;

        Ok(group)
    }

    fn parse_unit(&mut self) -> Result<A2lUnit> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();
        let long_identifier = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let display = self.parse_string()?;
        self.skip_whitespace_and_comments();
        let unit_type = self.parse_identifier()?;
        self.skip_whitespace_and_comments();

        let mut unit = A2lUnit {
            name,
            long_identifier,
            display,
            unit_type,
            ..Default::default()
        };

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("REF_UNIT") {
                self.expect_keyword("REF_UNIT")?;
                self.skip_whitespace_and_comments();
                unit.ref_unit = Some(self.parse_identifier()?);
            } else if self.peek_keyword("SI_EXPONENTS") {
                self.expect_keyword("SI_EXPONENTS")?;
                self.skip_whitespace_and_comments();
                let length = self.parse_i8()?;
                self.skip_whitespace_and_comments();
                let mass = self.parse_i8()?;
                self.skip_whitespace_and_comments();
                let time = self.parse_i8()?;
                self.skip_whitespace_and_comments();
                let current = self.parse_i8()?;
                self.skip_whitespace_and_comments();
                let temp = self.parse_i8()?;
                self.skip_whitespace_and_comments();
                let amount = self.parse_i8()?;
                self.skip_whitespace_and_comments();
                let luminosity = self.parse_i8()?;
                unit.si_exponents = Some((length, mass, time, current, temp, amount, luminosity));
            } else if self.peek_keyword("UNIT_CONVERSION") {
                self.expect_keyword("UNIT_CONVERSION")?;
                self.skip_whitespace_and_comments();
                let slope = self.parse_f64()?;
                self.skip_whitespace_and_comments();
                let offset = self.parse_f64()?;
                unit.unit_conversion = Some((slope, offset));
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("UNIT")?;

        Ok(unit)
    }

    fn parse_if_data(&mut self) -> Result<A2lIfData> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();

        // Capture raw content until /end IF_DATA
        let start_pos = self.pos;
        let mut depth = 1;
        while depth > 0 && !self.at_end() {
            if self.peek_keyword("/begin") {
                depth += 1;
                self.advance_n(6);
            } else if self.peek_keyword("/end") {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                self.advance_n(4);
            } else {
                self.advance_char();
            }
        }
        let content = self.input[start_pos..self.pos].trim().to_string();

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("IF_DATA")?;

        Ok(A2lIfData { name, content })
    }

    fn parse_annotation(&mut self) -> Result<A2lAnnotation> {
        self.skip_whitespace_and_comments();

        let mut annotation = A2lAnnotation::default();

        while !self.peek_keyword("/end") {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("ANNOTATION_LABEL") {
                self.expect_keyword("ANNOTATION_LABEL")?;
                self.skip_whitespace_and_comments();
                annotation.label = Some(self.parse_string()?);
            } else if self.peek_keyword("ANNOTATION_ORIGIN") {
                self.expect_keyword("ANNOTATION_ORIGIN")?;
                self.skip_whitespace_and_comments();
                annotation.origin = Some(self.parse_string()?);
            } else if self.peek_keyword("/begin") {
                self.expect_keyword("/begin")?;
                self.skip_whitespace_and_comments();
                if self.peek_keyword("ANNOTATION_TEXT") {
                    self.expect_keyword("ANNOTATION_TEXT")?;
                    self.skip_whitespace_and_comments();
                    while !self.peek_keyword("/end") {
                        if self.at_end() {
                            break;
                        }
                        if self.peek_char() == Some('"') {
                            annotation.text.push(self.parse_string()?);
                            self.skip_whitespace_and_comments();
                        } else {
                            self.advance_char();
                        }
                    }
                    self.expect_keyword("/end")?;
                    self.skip_whitespace_and_comments();
                    self.expect_keyword("ANNOTATION_TEXT")?;
                } else {
                    self.skip_block()?;
                }
            } else if self.at_end() || self.peek_keyword("/end") {
                break;
            } else {
                self.advance_char();
            }
        }

        self.expect_keyword("/end")?;
        self.skip_whitespace_and_comments();
        self.expect_keyword("ANNOTATION")?;

        Ok(annotation)
    }


    // ========================================================================
    // Helper methods
    // ========================================================================

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(c) = self.peek_char() {
                if c.is_whitespace() {
                    if c == '\n' {
                        self.line += 1;
                    }
                    self.advance_char();
                } else {
                    break;
                }
            }

            // Skip comments
            if self.peek_str("//") {
                // Single-line comment
                while let Some(c) = self.peek_char() {
                    self.advance_char();
                    if c == '\n' {
                        self.line += 1;
                        break;
                    }
                }
            } else if self.peek_str("/*") {
                // Multi-line comment
                self.advance_n(2);
                while !self.peek_str("*/") && !self.at_end() {
                    if self.peek_char() == Some('\n') {
                        self.line += 1;
                    }
                    self.advance_char();
                }
                if self.peek_str("*/") {
                    self.advance_n(2);
                }
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_str(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn peek_keyword(&self, keyword: &str) -> bool {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with(keyword) {
            let after = &remaining[keyword.len()..];
            after.is_empty()
                || after.starts_with(|c: char| c.is_whitespace() || c == '"' || c == '/')
        } else {
            false
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<()> {
        self.skip_whitespace_and_comments();
        if self.peek_keyword(keyword) {
            self.advance_n(keyword.len());
            Ok(())
        } else {
            Err(BusmasterError::DatabaseParse {
                message: format!("Expected '{}' at line {}", keyword, self.line),
                line: self.line,
            })
        }
    }

    fn advance_char(&mut self) {
        if let Some(c) = self.peek_char() {
            self.pos += c.len_utf8();
        }
    }

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance_char();
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn parse_identifier(&mut self) -> Result<String> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '[' || c == ']' {
                self.advance_char();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(BusmasterError::DatabaseParse {
                message: format!("Expected identifier at line {}", self.line),
                line: self.line,
            });
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_string(&mut self) -> Result<String> {
        self.skip_whitespace_and_comments();
        if self.peek_char() != Some('"') {
            return Err(BusmasterError::DatabaseParse {
                message: format!("Expected string at line {}", self.line),
                line: self.line,
            });
        }
        self.advance_char(); // Skip opening quote

        let mut result = String::new();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.advance_char();
                // Check for escaped quote
                if self.peek_char() == Some('"') {
                    result.push('"');
                    self.advance_char();
                } else {
                    break;
                }
            } else if c == '\\' {
                self.advance_char();
                if let Some(escaped) = self.peek_char() {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        _ => {
                            result.push('\\');
                            result.push(escaped);
                        }
                    }
                    self.advance_char();
                }
            } else {
                if c == '\n' {
                    self.line += 1;
                }
                result.push(c);
                self.advance_char();
            }
        }
        Ok(result)
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<String>> {
        let mut list = Vec::new();
        self.skip_whitespace_and_comments();
        while !self.peek_keyword("/end") && !self.at_end() {
            if let Ok(id) = self.parse_identifier() {
                list.push(id);
            }
            self.skip_whitespace_and_comments();
        }
        Ok(list)
    }


    fn parse_u16(&mut self) -> Result<u16> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(BusmasterError::DatabaseParse {
                message: format!("Expected number at line {}", self.line),
                line: self.line,
            });
        }
        self.input[start..self.pos]
            .parse()
            .map_err(|_| BusmasterError::DatabaseParse {
                message: format!("Invalid number at line {}", self.line),
                line: self.line,
            })
    }

    fn parse_u32(&mut self) -> Result<u32> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(BusmasterError::DatabaseParse {
                message: format!("Expected number at line {}", self.line),
                line: self.line,
            });
        }
        self.input[start..self.pos]
            .parse()
            .map_err(|_| BusmasterError::DatabaseParse {
                message: format!("Invalid number at line {}", self.line),
                line: self.line,
            })
    }

    fn parse_i8(&mut self) -> Result<i8> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        if self.peek_char() == Some('-') {
            self.advance_char();
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }
        if self.pos == start || (self.pos == start + 1 && self.input[start..].starts_with('-')) {
            return Err(BusmasterError::DatabaseParse {
                message: format!("Expected number at line {}", self.line),
                line: self.line,
            });
        }
        self.input[start..self.pos]
            .parse()
            .map_err(|_| BusmasterError::DatabaseParse {
                message: format!("Invalid number at line {}", self.line),
                line: self.line,
            })
    }

    fn parse_i64(&mut self) -> Result<i64> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        if self.peek_char() == Some('-') {
            self.advance_char();
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }
        if self.pos == start || (self.pos == start + 1 && self.input[start..].starts_with('-')) {
            return Err(BusmasterError::DatabaseParse {
                message: format!("Expected number at line {}", self.line),
                line: self.line,
            });
        }
        self.input[start..self.pos]
            .parse()
            .map_err(|_| BusmasterError::DatabaseParse {
                message: format!("Invalid number at line {}", self.line),
                line: self.line,
            })
    }

    fn parse_f64(&mut self) -> Result<f64> {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        if self.peek_char() == Some('-') || self.peek_char() == Some('+') {
            self.advance_char();
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }
        if self.peek_char() == Some('.') {
            self.advance_char();
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    self.advance_char();
                } else {
                    break;
                }
            }
        }
        // Handle scientific notation
        if self.peek_char() == Some('e') || self.peek_char() == Some('E') {
            self.advance_char();
            if self.peek_char() == Some('-') || self.peek_char() == Some('+') {
                self.advance_char();
            }
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    self.advance_char();
                } else {
                    break;
                }
            }
        }
        if self.pos == start {
            return Err(BusmasterError::DatabaseParse {
                message: format!("Expected number at line {}", self.line),
                line: self.line,
            });
        }
        self.input[start..self.pos]
            .parse()
            .map_err(|_| BusmasterError::DatabaseParse {
                message: format!("Invalid number at line {}", self.line),
                line: self.line,
            })
    }

    fn parse_hex_or_dec(&mut self) -> Result<u64> {
        self.skip_whitespace_and_comments();
        if self.peek_str("0x") || self.peek_str("0X") {
            self.advance_n(2);
            let start = self.pos;
            while let Some(c) = self.peek_char() {
                if c.is_ascii_hexdigit() {
                    self.advance_char();
                } else {
                    break;
                }
            }
            u64::from_str_radix(&self.input[start..self.pos], 16).map_err(|_| {
                BusmasterError::DatabaseParse {
                    message: format!("Invalid hex number at line {}", self.line),
                    line: self.line,
                }
            })
        } else {
            let start = self.pos;
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    self.advance_char();
                } else {
                    break;
                }
            }
            self.input[start..self.pos]
                .parse()
                .map_err(|_| BusmasterError::DatabaseParse {
                    message: format!("Invalid number at line {}", self.line),
                    line: self.line,
                })
        }
    }

    fn skip_block(&mut self) -> Result<()> {
        let mut depth = 1;
        while depth > 0 && !self.at_end() {
            self.skip_whitespace_and_comments();
            if self.peek_keyword("/begin") {
                depth += 1;
                self.advance_n(6);
            } else if self.peek_keyword("/end") {
                depth -= 1;
                self.advance_n(4);
                if depth == 0 {
                    // Skip the block name after /end
                    self.skip_whitespace_and_comments();
                    let _ = self.parse_identifier();
                }
            } else {
                self.advance_char();
            }
        }
        Ok(())
    }
}


// ============================================================================
// A2L Generator - Create A2L files programmatically
// ============================================================================

/// A2L file generator for creating ASAM MCD-2 MC compliant files
pub struct A2lGenerator;

impl A2lGenerator {
    /// Generate A2L file content from database
    pub fn generate(db: &A2lDatabase) -> String {
        let mut output = String::new();

        // ASAP2 version
        output.push_str(&format!(
            "ASAP2_VERSION {} {}\n\n",
            db.asap2_version.major, db.asap2_version.minor
        ));

        // A2ML version if present
        if let Some(ref a2ml) = db.a2ml_version {
            output.push_str(&format!("A2ML_VERSION {} {}\n\n", a2ml.major, a2ml.minor));
        }

        // Project
        output.push_str(&format!(
            "/begin PROJECT {} \"{}\"\n",
            db.project.name, db.project.long_identifier
        ));

        // Header
        if let Some(ref header) = db.project.header {
            output.push_str("  /begin HEADER\n");
            output.push_str(&format!("    \"{}\"\n", header.comment));
            if let Some(ref pno) = header.project_no {
                output.push_str(&format!("    PROJECT_NO {}\n", pno));
            }
            if let Some(ref ver) = header.version {
                output.push_str(&format!("    VERSION \"{}\"\n", ver));
            }
            output.push_str("  /end HEADER\n\n");
        }

        // Modules
        for module in &db.project.modules {
            Self::generate_module(&mut output, module);
        }

        output.push_str("/end PROJECT\n");
        output
    }

    fn generate_module(output: &mut String, module: &A2lModule) {
        output.push_str(&format!(
            "  /begin MODULE {} \"{}\"\n\n",
            module.name, module.long_identifier
        ));

        // MOD_COMMON
        if let Some(ref mc) = module.mod_common {
            output.push_str("    /begin MOD_COMMON\n");
            output.push_str(&format!("      \"{}\"\n", mc.comment));
            if let Some(ref bo) = mc.byte_order {
                let bo_str = match bo {
                    A2lByteOrder::BigEndian => "MSB_FIRST",
                    A2lByteOrder::LittleEndian => "LSB_FIRST",
                };
                output.push_str(&format!("      BYTE_ORDER {}\n", bo_str));
            }
            if let Some(ds) = mc.data_size {
                output.push_str(&format!("      DATA_SIZE {}\n", ds));
            }
            output.push_str("    /end MOD_COMMON\n\n");
        }

        // COMPU_METHODs
        for cm in &module.compu_methods {
            Self::generate_compu_method(output, cm);
        }

        // RECORD_LAYOUTs
        for rl in &module.record_layouts {
            Self::generate_record_layout(output, rl);
        }

        // MEASUREMENTs
        for meas in &module.measurements {
            Self::generate_measurement(output, meas);
        }

        // CHARACTERISTICs
        for char in &module.characteristics {
            Self::generate_characteristic(output, char);
        }

        // AXIS_PTS
        for axis in &module.axis_pts {
            Self::generate_axis_pts(output, axis);
        }

        // FUNCTIONs
        for func in &module.functions {
            Self::generate_function(output, func);
        }

        // GROUPs
        for group in &module.groups {
            Self::generate_group(output, group);
        }

        output.push_str("  /end MODULE\n\n");
    }

    fn generate_compu_method(output: &mut String, cm: &A2lCompuMethod) {
        let conv_type = match cm.conversion_type {
            CompuMethodType::Identical => "IDENTICAL",
            CompuMethodType::Linear => "LINEAR",
            CompuMethodType::RatFunc => "RAT_FUNC",
            CompuMethodType::TabIntp => "TAB_INTP",
            CompuMethodType::TabNoIntp => "TAB_NOINTP",
            CompuMethodType::TabVerb => "TAB_VERB",
            CompuMethodType::Form => "FORM",
        };
        output.push_str(&format!(
            "    /begin COMPU_METHOD {} \"{}\" {} \"{}\" \"{}\"\n",
            cm.name, cm.long_identifier, conv_type, cm.format, cm.unit
        ));
        if let Some((a, b)) = cm.coeffs_linear {
            output.push_str(&format!("      COEFFS_LINEAR {} {}\n", a, b));
        }
        if let Some((a, b, c, d, e, f)) = cm.coeffs {
            output.push_str(&format!(
                "      COEFFS {} {} {} {} {} {}\n",
                a, b, c, d, e, f
            ));
        }
        if let Some(ref tab) = cm.compu_tab_ref {
            output.push_str(&format!("      COMPU_TAB_REF {}\n", tab));
        }
        output.push_str("    /end COMPU_METHOD\n\n");
    }

    fn generate_record_layout(output: &mut String, rl: &A2lRecordLayout) {
        output.push_str(&format!("    /begin RECORD_LAYOUT {}\n", rl.name));
        if let Some(ref fnc) = rl.fnc_values {
            let dt = Self::data_type_str(fnc.data_type);
            output.push_str(&format!(
                "      FNC_VALUES {} {} {} {}\n",
                fnc.position, dt, fnc.index_mode, fnc.address_type
            ));
        }
        if rl.static_record_layout {
            output.push_str("      STATIC_RECORD_LAYOUT\n");
        }
        output.push_str("    /end RECORD_LAYOUT\n\n");
    }

    fn generate_measurement(output: &mut String, meas: &A2lMeasurement) {
        let dt = Self::data_type_str(meas.data_type);
        output.push_str(&format!(
            "    /begin MEASUREMENT {} \"{}\" {} {} {} {} {} {}\n",
            meas.name,
            meas.long_identifier,
            dt,
            meas.compu_method,
            meas.resolution,
            meas.accuracy,
            meas.lower_limit,
            meas.upper_limit
        ));
        if let Some(addr) = meas.ecu_address {
            output.push_str(&format!("      ECU_ADDRESS 0x{:08X}\n", addr));
        }
        if let Some(ref fmt) = meas.format {
            output.push_str(&format!("      FORMAT \"{}\"\n", fmt));
        }
        if let Some(ref unit) = meas.phys_unit {
            output.push_str(&format!("      PHYS_UNIT \"{}\"\n", unit));
        }
        if meas.read_write {
            output.push_str("      READ_WRITE\n");
        }
        output.push_str("    /end MEASUREMENT\n\n");
    }

    fn generate_characteristic(output: &mut String, char: &A2lCharacteristic) {
        let ct = match char.char_type {
            CharacteristicType::Value => "VALUE",
            CharacteristicType::Ascii => "ASCII",
            CharacteristicType::ValBlk => "VAL_BLK",
            CharacteristicType::Curve => "CURVE",
            CharacteristicType::Map => "MAP",
            CharacteristicType::Cuboid => "CUBOID",
            CharacteristicType::Cube4 => "CUBE_4",
            CharacteristicType::Cube5 => "CUBE_5",
        };
        output.push_str(&format!(
            "    /begin CHARACTERISTIC {} \"{}\" {} 0x{:08X} {} {} {} {} {}\n",
            char.name,
            char.long_identifier,
            ct,
            char.address,
            char.record_layout,
            char.max_diff,
            char.compu_method,
            char.lower_limit,
            char.upper_limit
        ));
        if let Some(ref fmt) = char.format {
            output.push_str(&format!("      FORMAT \"{}\"\n", fmt));
        }
        if char.read_only {
            output.push_str("      READ_ONLY\n");
        }
        // Axis descriptions
        for axis in &char.axis_descr {
            Self::generate_axis_descr(output, axis);
        }
        output.push_str("    /end CHARACTERISTIC\n\n");
    }

    fn generate_axis_descr(output: &mut String, axis: &A2lAxisDescr) {
        let at = match axis.axis_type {
            AxisType::StdAxis => "STD_AXIS",
            AxisType::FixAxis => "FIX_AXIS",
            AxisType::ComAxis => "COM_AXIS",
            AxisType::CurveAxis => "CURVE_AXIS",
            AxisType::ResAxis => "RES_AXIS",
        };
        output.push_str(&format!(
            "      /begin AXIS_DESCR {} {} {} {} {} {}\n",
            at,
            axis.input_quantity,
            axis.compu_method,
            axis.max_axis_points,
            axis.lower_limit,
            axis.upper_limit
        ));
        if let Some(ref pts_ref) = axis.axis_pts_ref {
            output.push_str(&format!("        AXIS_PTS_REF {}\n", pts_ref));
        }
        output.push_str("      /end AXIS_DESCR\n");
    }

    fn generate_axis_pts(output: &mut String, axis: &A2lAxisPts) {
        output.push_str(&format!(
            "    /begin AXIS_PTS {} \"{}\" 0x{:08X} {} {} {} {} {} {} {}\n",
            axis.name,
            axis.long_identifier,
            axis.address,
            axis.input_quantity,
            axis.record_layout,
            axis.max_diff,
            axis.compu_method,
            axis.max_axis_points,
            axis.lower_limit,
            axis.upper_limit
        ));
        if axis.read_only {
            output.push_str("      READ_ONLY\n");
        }
        output.push_str("    /end AXIS_PTS\n\n");
    }

    fn generate_function(output: &mut String, func: &A2lFunction) {
        output.push_str(&format!(
            "    /begin FUNCTION {} \"{}\"\n",
            func.name, func.long_identifier
        ));
        if !func.def_characteristic.is_empty() {
            output.push_str("      /begin DEF_CHARACTERISTIC\n");
            for c in &func.def_characteristic {
                output.push_str(&format!("        {}\n", c));
            }
            output.push_str("      /end DEF_CHARACTERISTIC\n");
        }
        if !func.in_measurement.is_empty() {
            output.push_str("      /begin IN_MEASUREMENT\n");
            for m in &func.in_measurement {
                output.push_str(&format!("        {}\n", m));
            }
            output.push_str("      /end IN_MEASUREMENT\n");
        }
        if !func.out_measurement.is_empty() {
            output.push_str("      /begin OUT_MEASUREMENT\n");
            for m in &func.out_measurement {
                output.push_str(&format!("        {}\n", m));
            }
            output.push_str("      /end OUT_MEASUREMENT\n");
        }
        output.push_str("    /end FUNCTION\n\n");
    }

    fn generate_group(output: &mut String, group: &A2lGroup) {
        output.push_str(&format!(
            "    /begin GROUP {} \"{}\"\n",
            group.name, group.long_identifier
        ));
        if group.root {
            output.push_str("      ROOT\n");
        }
        if !group.ref_characteristic.is_empty() {
            output.push_str("      /begin REF_CHARACTERISTIC\n");
            for c in &group.ref_characteristic {
                output.push_str(&format!("        {}\n", c));
            }
            output.push_str("      /end REF_CHARACTERISTIC\n");
        }
        if !group.ref_measurement.is_empty() {
            output.push_str("      /begin REF_MEASUREMENT\n");
            for m in &group.ref_measurement {
                output.push_str(&format!("        {}\n", m));
            }
            output.push_str("      /end REF_MEASUREMENT\n");
        }
        output.push_str("    /end GROUP\n\n");
    }

    fn data_type_str(dt: A2lDataType) -> &'static str {
        match dt {
            A2lDataType::UByte => "UBYTE",
            A2lDataType::SByte => "SBYTE",
            A2lDataType::UWord => "UWORD",
            A2lDataType::SWord => "SWORD",
            A2lDataType::ULong => "ULONG",
            A2lDataType::SLong => "SLONG",
            A2lDataType::AUint64 => "A_UINT64",
            A2lDataType::AInt64 => "A_INT64",
            A2lDataType::Float32Ieee => "FLOAT32_IEEE",
            A2lDataType::Float64Ieee => "FLOAT64_IEEE",
        }
    }
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_a2l() {
        let a2l = r#"
ASAP2_VERSION 1 71
/begin PROJECT TestProject "Test Project Description"
  /begin MODULE TestModule "Test Module"
  /end MODULE
/end PROJECT
"#;
        let db = A2lParser::parse(a2l).unwrap();
        assert_eq!(db.asap2_version.major, 1);
        assert_eq!(db.asap2_version.minor, 71);
        assert_eq!(db.project.name, "TestProject");
        assert_eq!(db.project.modules.len(), 1);
        assert_eq!(db.project.modules[0].name, "TestModule");
    }

    #[test]
    fn test_parse_measurement() {
        let a2l = r#"
ASAP2_VERSION 1 71
/begin PROJECT Test ""
  /begin MODULE Mod ""
    /begin MEASUREMENT EngineSpeed
      "Engine rotational speed"
      UWORD NO_COMPU_METHOD 0 0 0 16383
      ECU_ADDRESS 0x12345678
      FORMAT "%.1f"
      PHYS_UNIT "rpm"
    /end MEASUREMENT
  /end MODULE
/end PROJECT
"#;
        let db = A2lParser::parse(a2l).unwrap();
        let meas = db.find_measurement("EngineSpeed").unwrap();
        assert_eq!(meas.name, "EngineSpeed");
        assert_eq!(meas.data_type, A2lDataType::UWord);
        assert_eq!(meas.ecu_address, Some(0x12345678));
        assert_eq!(meas.upper_limit, 16383.0);
        assert_eq!(meas.phys_unit, Some("rpm".to_string()));
    }

    #[test]
    fn test_parse_characteristic() {
        let a2l = r#"
ASAP2_VERSION 1 71
/begin PROJECT Test ""
  /begin MODULE Mod ""
    /begin CHARACTERISTIC FuelMap
      "Fuel injection map"
      MAP 0xABCD0000 FuelMapLayout 0.0 NO_COMPU_METHOD 0 255
      /begin AXIS_DESCR STD_AXIS EngineSpeed NO_COMPU_METHOD 16 0 8000
      /end AXIS_DESCR
      /begin AXIS_DESCR STD_AXIS EngineLoad NO_COMPU_METHOD 16 0 100
      /end AXIS_DESCR
    /end CHARACTERISTIC
  /end MODULE
/end PROJECT
"#;
        let db = A2lParser::parse(a2l).unwrap();
        let char = db.find_characteristic("FuelMap").unwrap();
        assert_eq!(char.name, "FuelMap");
        assert_eq!(char.char_type, CharacteristicType::Map);
        assert_eq!(char.address, 0xABCD0000);
        assert_eq!(char.axis_descr.len(), 2);
        assert_eq!(char.axis_descr[0].input_quantity, "EngineSpeed");
        assert_eq!(char.axis_descr[1].input_quantity, "EngineLoad");
    }

    #[test]
    fn test_parse_compu_method_linear() {
        let a2l = r#"
ASAP2_VERSION 1 71
/begin PROJECT Test ""
  /begin MODULE Mod ""
    /begin COMPU_METHOD CM_Speed
      "Speed conversion"
      LINEAR "%.2f" "km/h"
      COEFFS_LINEAR 0.01 0
    /end COMPU_METHOD
  /end MODULE
/end PROJECT
"#;
        let db = A2lParser::parse(a2l).unwrap();
        let cm = db.find_compu_method("CM_Speed").unwrap();
        assert_eq!(cm.name, "CM_Speed");
        assert_eq!(cm.conversion_type, CompuMethodType::Linear);
        assert_eq!(cm.coeffs_linear, Some((0.01, 0.0)));
        assert_eq!(cm.unit, "km/h");
    }

    #[test]
    fn test_compu_method_conversion() {
        let cm = A2lCompuMethod::linear("test", "test", 0.25, -40.0);
        // Internal value 200 -> physical = 0.25 * 200 - 40 = 10
        assert!((cm.convert_to_physical(200.0) - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_data_type_properties() {
        assert_eq!(A2lDataType::UByte.size_bytes(), 1);
        assert_eq!(A2lDataType::UWord.size_bytes(), 2);
        assert_eq!(A2lDataType::ULong.size_bytes(), 4);
        assert_eq!(A2lDataType::Float64Ieee.size_bytes(), 8);

        assert!(!A2lDataType::UByte.is_signed());
        assert!(A2lDataType::SByte.is_signed());
        assert!(A2lDataType::Float32Ieee.is_signed());
    }

    #[test]
    fn test_generate_a2l() {
        let mut db = A2lDatabase::new();
        db.asap2_version = Asap2Version::new(1, 71);
        db.project.name = "TestProject".to_string();
        db.project.long_identifier = "Test Project".to_string();

        let mut module = A2lModule::default();
        module.name = "TestModule".to_string();
        module.long_identifier = "Test Module".to_string();

        let meas = A2lMeasurement::new("Speed", "Vehicle speed", A2lDataType::UWord)
            .with_address(0x1000)
            .with_limits(0.0, 300.0)
            .with_compu_method("CM_Speed");
        module.measurements.push(meas);

        db.project.modules.push(module);

        let output = A2lGenerator::generate(&db);
        assert!(output.contains("ASAP2_VERSION 1 71"));
        assert!(output.contains("PROJECT TestProject"));
        assert!(output.contains("MEASUREMENT Speed"));
        assert!(output.contains("ECU_ADDRESS 0x00001000"));
    }

    #[test]
    fn test_parse_with_comments() {
        let a2l = r#"
// This is a single-line comment
ASAP2_VERSION 1 71
/* This is a
   multi-line comment */
/begin PROJECT Test ""
  /begin MODULE Mod "" // inline comment
  /end MODULE
/end PROJECT
"#;
        let db = A2lParser::parse(a2l).unwrap();
        assert_eq!(db.project.name, "Test");
    }

    #[test]
    fn test_parse_function() {
        let a2l = r#"
ASAP2_VERSION 1 71
/begin PROJECT Test ""
  /begin MODULE Mod ""
    /begin FUNCTION EngineControl "Engine control function"
      /begin DEF_CHARACTERISTIC
        FuelMap
        IgnitionMap
      /end DEF_CHARACTERISTIC
      /begin IN_MEASUREMENT
        EngineSpeed
        EngineLoad
      /end IN_MEASUREMENT
      /begin OUT_MEASUREMENT
        FuelInjection
      /end OUT_MEASUREMENT
    /end FUNCTION
  /end MODULE
/end PROJECT
"#;
        let db = A2lParser::parse(a2l).unwrap();
        let func = &db.project.modules[0].functions[0];
        assert_eq!(func.name, "EngineControl");
        assert_eq!(func.def_characteristic.len(), 2);
        assert_eq!(func.in_measurement.len(), 2);
        assert_eq!(func.out_measurement.len(), 1);
    }

    #[test]
    fn test_parse_group() {
        let a2l = r#"
ASAP2_VERSION 1 71
/begin PROJECT Test ""
  /begin MODULE Mod ""
    /begin GROUP EngineParams "Engine parameters"
      ROOT
      /begin REF_CHARACTERISTIC
        FuelMap
      /end REF_CHARACTERISTIC
      /begin REF_MEASUREMENT
        EngineSpeed
      /end REF_MEASUREMENT
    /end GROUP
  /end MODULE
/end PROJECT
"#;
        let db = A2lParser::parse(a2l).unwrap();
        let group = &db.project.modules[0].groups[0];
        assert_eq!(group.name, "EngineParams");
        assert!(group.root);
        assert_eq!(group.ref_characteristic.len(), 1);
        assert_eq!(group.ref_measurement.len(), 1);
    }

    #[test]
    fn test_asap2_version_string() {
        let ver = Asap2Version::new(1, 71);
        assert_eq!(ver.to_version_string(), "1.71");
    }

    #[test]
    fn test_characteristic_type_from_str() {
        assert_eq!(
            CharacteristicType::from_str("VALUE"),
            Some(CharacteristicType::Value)
        );
        assert_eq!(
            CharacteristicType::from_str("CURVE"),
            Some(CharacteristicType::Curve)
        );
        assert_eq!(
            CharacteristicType::from_str("MAP"),
            Some(CharacteristicType::Map)
        );
        assert_eq!(
            CharacteristicType::from_str("CUBOID"),
            Some(CharacteristicType::Cuboid)
        );
        assert_eq!(CharacteristicType::from_str("INVALID"), None);
    }

    #[test]
    fn test_axis_type_from_str() {
        assert_eq!(AxisType::from_str("STD_AXIS"), Some(AxisType::StdAxis));
        assert_eq!(AxisType::from_str("FIX_AXIS"), Some(AxisType::FixAxis));
        assert_eq!(AxisType::from_str("COM_AXIS"), Some(AxisType::ComAxis));
        assert_eq!(AxisType::from_str("INVALID"), None);
    }
}
