//! ODX (Open Diagnostic Data Exchange) Parser
//!
//! This module provides parsing for ASAM MCD-2 D (ODX) files, which describe
//! the diagnostic capabilities of automotive ECUs.
//!
//! # ODX Format Overview
//!
//! ODX is an XML-based format standardized by ASAM and ISO 22901-1. It describes:
//! - Diagnostic services (requests and responses)
//! - Data parameters and their encoding
//! - Communication parameters
//! - ECU variants and configurations
//! - Diagnostic trouble codes (DTCs)
//!
//! # ODX Categories
//!
//! ODX files are organized into categories:
//! - **ODX-D** - Diagnostic layer (services, parameters)
//! - **ODX-C** - Communication parameters
//! - **ODX-V** - Vehicle information
//! - **ODX-F** - Flash data
//! - **ODX-M** - Multiple ECU jobs
//! - **ODX-E** - ECU configuration
//!
//! # Example
//!
//! ```ignore
//! use busmaster_db::odx::OdxParser;
//!
//! let odx_content = std::fs::read_to_string("ecu.odx")?;
//! let database = OdxParser::parse(&odx_content)?;
//!
//! // Find a diagnostic service
//! if let Some(service) = database.find_service("ReadDataByIdentifier") {
//!     println!("Service ID: 0x{:02X}", service.service_id);
//! }
//! ```

#![allow(clippy::too_many_lines)]
#![allow(clippy::similar_names)]

use serde::{Deserialize, Serialize};


// ============================================================================
// ODX Database Structure
// ============================================================================

/// ODX Database - root container for parsed ODX data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OdxDatabase {
    /// Database short name
    pub short_name: String,
    /// Database long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// ODX version
    pub version: Option<String>,
    /// Diagnostic layers
    pub diag_layers: Vec<DiagLayer>,
    /// Communication parameters
    pub comparam_specs: Vec<ComparamSpec>,
    /// Vehicle information
    pub vehicle_info: Option<VehicleInfo>,
    /// ECU shared data
    pub ecu_shared_data: Option<EcuSharedData>,
    /// Functional groups
    pub functional_groups: Vec<FunctionalGroup>,
}

impl OdxDatabase {
    /// Create a new empty ODX database
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Find a diagnostic layer by short name
    #[must_use]
    pub fn find_diag_layer(&self, name: &str) -> Option<&DiagLayer> {
        self.diag_layers.iter().find(|dl| dl.short_name == name)
    }

    /// Find a service by name across all layers
    #[must_use]
    pub fn find_service(&self, name: &str) -> Option<&DiagService> {
        for layer in &self.diag_layers {
            if let Some(service) = layer.find_service(name) {
                return Some(service);
            }
        }
        None
    }

    /// Find a DTC by code across all layers
    #[must_use]
    pub fn find_dtc(&self, code: u32) -> Option<&Dtc> {
        for layer in &self.diag_layers {
            if let Some(dtc) = layer.find_dtc(code) {
                return Some(dtc);
            }
        }
        None
    }

    /// Get all services across all layers
    #[must_use]
    pub fn all_services(&self) -> Vec<&DiagService> {
        self.diag_layers
            .iter()
            .flat_map(|dl| dl.services.iter())
            .collect()
    }

    /// Get all DTCs across all layers
    #[must_use]
    pub fn all_dtcs(&self) -> Vec<&Dtc> {
        self.diag_layers
            .iter()
            .flat_map(|dl| dl.dtcs.iter())
            .collect()
    }
}


// ============================================================================
// Diagnostic Layer
// ============================================================================

/// Diagnostic layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiagLayerType {
    /// Protocol layer (base communication)
    Protocol,
    /// Functional group layer
    FunctionalGroup,
    /// Base variant layer
    BaseVariant,
    /// ECU variant layer
    #[default]
    EcuVariant,
    /// ECU shared data layer
    EcuSharedData,
}

impl DiagLayerType {
    /// Parse from string
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PROTOCOL" => Some(Self::Protocol),
            "FUNCTIONAL-GROUP" => Some(Self::FunctionalGroup),
            "BASE-VARIANT" => Some(Self::BaseVariant),
            "ECU-VARIANT" => Some(Self::EcuVariant),
            "ECU-SHARED-DATA" => Some(Self::EcuSharedData),
            _ => None,
        }
    }
}

/// Diagnostic layer - container for diagnostic services
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagLayer {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Layer type
    pub layer_type: DiagLayerType,
    /// Parent layer references
    pub parent_refs: Vec<String>,
    /// Diagnostic services
    pub services: Vec<DiagService>,
    /// Single ECU jobs
    pub single_ecu_jobs: Vec<SingleEcuJob>,
    /// Diagnostic trouble codes
    pub dtcs: Vec<Dtc>,
    /// Data dictionary (parameters)
    pub data_dictionary: Vec<DataDictionaryEntry>,
    /// Communication parameters
    pub comparam_refs: Vec<String>,
    /// Functional classes
    pub functional_classes: Vec<FunctionalClass>,
    /// Audience (access level)
    pub audience: Option<Audience>,
}

impl DiagLayer {
    /// Create a new diagnostic layer
    #[must_use]
    pub fn new(id: &str, short_name: &str, layer_type: DiagLayerType) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            layer_type,
            ..Default::default()
        }
    }

    /// Find a service by name
    #[must_use]
    pub fn find_service(&self, name: &str) -> Option<&DiagService> {
        self.services.iter().find(|s| s.short_name == name)
    }

    /// Find a service by service ID
    #[must_use]
    pub fn find_service_by_id(&self, service_id: u8) -> Option<&DiagService> {
        self.services.iter().find(|s| s.service_id == Some(service_id))
    }

    /// Find a DTC by code
    #[must_use]
    pub fn find_dtc(&self, code: u32) -> Option<&Dtc> {
        self.dtcs.iter().find(|d| d.code == code)
    }

    /// Add a service
    pub fn add_service(&mut self, service: DiagService) {
        self.services.push(service);
    }

    /// Add a DTC
    pub fn add_dtc(&mut self, dtc: Dtc) {
        self.dtcs.push(dtc);
    }
}


// ============================================================================
// Diagnostic Service
// ============================================================================

/// Diagnostic service (request/response pair)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagService {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// UDS Service ID (SID)
    pub service_id: Option<u8>,
    /// Semantic (e.g., "READ", "WRITE", "CONTROL")
    pub semantic: Option<String>,
    /// Request structure
    pub request: Option<Request>,
    /// Positive response structure
    pub positive_response: Option<Response>,
    /// Negative response structure
    pub negative_response: Option<Response>,
    /// Functional class references
    pub functional_class_refs: Vec<String>,
    /// Audience (access level)
    pub audience: Option<Audience>,
    /// Is addressing physical (vs functional)
    pub addressing_physical: bool,
    /// Is addressing functional
    pub addressing_functional: bool,
}

impl DiagService {
    /// Create a new diagnostic service
    #[must_use]
    pub fn new(id: &str, short_name: &str) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            addressing_physical: true,
            ..Default::default()
        }
    }

    /// Set service ID
    #[must_use]
    pub fn with_service_id(mut self, sid: u8) -> Self {
        self.service_id = Some(sid);
        self
    }

    /// Set request
    #[must_use]
    pub fn with_request(mut self, request: Request) -> Self {
        self.request = Some(request);
        self
    }

    /// Set positive response
    #[must_use]
    pub fn with_positive_response(mut self, response: Response) -> Self {
        self.positive_response = Some(response);
        self
    }
}

/// Request structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Request {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Parameters
    pub params: Vec<Param>,
}

impl Request {
    /// Create a new request
    #[must_use]
    pub fn new(id: &str, short_name: &str) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            params: Vec::new(),
        }
    }

    /// Add a parameter
    pub fn add_param(&mut self, param: Param) {
        self.params.push(param);
    }

    /// Get total byte length
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.params.iter().map(|p| p.byte_size.unwrap_or(0) as usize).sum()
    }
}

/// Response structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Response {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Parameters
    pub params: Vec<Param>,
}

impl Response {
    /// Create a new response
    #[must_use]
    pub fn new(id: &str, short_name: &str) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            params: Vec::new(),
        }
    }

    /// Add a parameter
    pub fn add_param(&mut self, param: Param) {
        self.params.push(param);
    }
}


// ============================================================================
// Parameters
// ============================================================================

/// Parameter semantic type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ParamSemantic {
    /// Service identifier
    ServiceId,
    /// Sub-function
    SubFunction,
    /// Data identifier
    #[default]
    Data,
    /// Table key
    TableKey,
    /// Table struct
    TableStruct,
    /// Table row
    TableRow,
    /// Table entry
    TableEntry,
}

/// Parameter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ParamType {
    /// Coded constant
    CodedConst,
    /// Matching request parameter
    MatchingRequestParam,
    /// Value parameter
    #[default]
    Value,
    /// Reserved parameter
    Reserved,
    /// Table key parameter
    TableKey,
    /// Table struct parameter
    TableStruct,
    /// Table entry parameter
    TableEntry,
    /// Length key parameter
    LengthKey,
    /// Dynamic parameter
    Dynamic,
    /// End of PDU field
    EndOfPduField,
    /// NRC constant
    NrcConst,
}

/// Parameter definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Param {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Parameter type
    pub param_type: ParamType,
    /// Semantic
    pub semantic: ParamSemantic,
    /// Byte position
    pub byte_position: Option<u32>,
    /// Bit position within byte
    pub bit_position: Option<u8>,
    /// Byte size
    pub byte_size: Option<u32>,
    /// Bit size
    pub bit_size: Option<u32>,
    /// Coded value (for constants)
    pub coded_value: Option<i64>,
    /// Data object property reference
    pub dop_ref: Option<String>,
    /// Physical default value
    pub physical_default: Option<String>,
    /// Is mandatory
    pub mandatory: bool,
}

impl Param {
    /// Create a new parameter
    #[must_use]
    pub fn new(id: &str, short_name: &str, param_type: ParamType) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            param_type,
            mandatory: true,
            ..Default::default()
        }
    }

    /// Create a coded constant parameter
    #[must_use]
    pub fn coded_const(id: &str, short_name: &str, value: i64, byte_pos: u32) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            param_type: ParamType::CodedConst,
            byte_position: Some(byte_pos),
            coded_value: Some(value),
            byte_size: Some(1),
            mandatory: true,
            ..Default::default()
        }
    }

    /// Create a value parameter
    #[must_use]
    pub fn value(id: &str, short_name: &str, byte_pos: u32, byte_size: u32) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            param_type: ParamType::Value,
            byte_position: Some(byte_pos),
            byte_size: Some(byte_size),
            mandatory: true,
            ..Default::default()
        }
    }

    /// Set DOP reference
    #[must_use]
    pub fn with_dop_ref(mut self, dop_ref: &str) -> Self {
        self.dop_ref = Some(dop_ref.to_string());
        self
    }
}


// ============================================================================
// Data Object Properties (DOP)
// ============================================================================

/// Data Object Property - defines data encoding/decoding
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataObjectProperty {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Computation method
    pub compu_method: Option<CompuMethod>,
    /// Physical type
    pub physical_type: Option<PhysicalType>,
    /// Diag coded type
    pub diag_coded_type: Option<DiagCodedType>,
    /// Unit reference
    pub unit_ref: Option<String>,
}

impl DataObjectProperty {
    /// Create a new DOP
    #[must_use]
    pub fn new(id: &str, short_name: &str) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            ..Default::default()
        }
    }
}

/// Physical type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhysicalType {
    /// Base data type
    pub base_data_type: BaseDataType,
    /// Display radix
    pub display_radix: Option<String>,
}

/// Base data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BaseDataType {
    /// Unsigned integer
    #[default]
    AUint32,
    /// Signed integer
    AInt32,
    /// Float
    AFloat32,
    /// Double
    AFloat64,
    /// ASCII string
    AAsciiString,
    /// Unicode string
    AUnicodeString,
    /// Byte field
    AByteField,
}

/// Diag coded type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagCodedType {
    /// Base data type
    pub base_data_type: BaseDataType,
    /// Bit length
    pub bit_length: Option<u32>,
    /// Byte order
    pub byte_order: Option<ByteOrder>,
    /// Is high-low byte order
    pub is_highlow_byte_order: bool,
}

/// Byte order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ByteOrder {
    /// Little endian (LSB first)
    #[default]
    LittleEndian,
    /// Big endian (MSB first)
    BigEndian,
}

/// Computation method for value conversion
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompuMethod {
    /// Category
    pub category: CompuCategory,
    /// Internal to physical coefficients (for LINEAR)
    pub compu_internal_to_phys: Option<CompuScale>,
    /// Physical to internal coefficients
    pub compu_phys_to_internal: Option<CompuScale>,
    /// Computation scales (for SCALE-LINEAR, TAB-INTP)
    pub compu_scales: Vec<CompuScale>,
    /// Default value
    pub compu_default_value: Option<String>,
}

/// Computation category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CompuCategory {
    /// Identical (no conversion)
    #[default]
    Identical,
    /// Linear conversion
    Linear,
    /// Scale linear (piecewise linear)
    ScaleLinear,
    /// Table interpolation
    TabIntp,
    /// Table no interpolation
    TabNoIntp,
    /// Text table
    TextTable,
    /// Bit-to-text table
    BitToTextTable,
}

/// Computation scale
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompuScale {
    /// Lower limit
    pub lower_limit: Option<f64>,
    /// Upper limit
    pub upper_limit: Option<f64>,
    /// Coefficients (a, b, c, d, e, f) for rational function
    pub compu_rational_coeffs: Option<CompuRationalCoeffs>,
    /// Constant value
    pub compu_const: Option<String>,
    /// Short label
    pub short_label: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Rational function coefficients
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompuRationalCoeffs {
    /// Numerator coefficients
    pub numerators: Vec<f64>,
    /// Denominator coefficients
    pub denominators: Vec<f64>,
}


// ============================================================================
// Diagnostic Trouble Codes (DTCs)
// ============================================================================

/// DTC severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DtcSeverity {
    /// No severity
    #[default]
    NoSeverity,
    /// Maintenance only
    MaintenanceOnly,
    /// Check at next halt
    CheckAtNextHalt,
    /// Check immediately
    CheckImmediately,
}

/// Diagnostic Trouble Code
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dtc {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// DTC code (3-byte value)
    pub code: u32,
    /// Display code (e.g., "P0123")
    pub display_code: Option<String>,
    /// Severity
    pub severity: DtcSeverity,
    /// Functional class references
    pub functional_class_refs: Vec<String>,
    /// Text (human-readable description)
    pub text: Option<String>,
    /// Level (for hierarchical DTCs)
    pub level: Option<u8>,
}

impl Dtc {
    /// Create a new DTC
    #[must_use]
    pub fn new(id: &str, short_name: &str, code: u32) -> Self {
        Self {
            id: id.to_string(),
            short_name: short_name.to_string(),
            code,
            ..Default::default()
        }
    }

    /// Set display code
    #[must_use]
    pub fn with_display_code(mut self, display_code: &str) -> Self {
        self.display_code = Some(display_code.to_string());
        self
    }

    /// Set description text
    #[must_use]
    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    /// Set severity
    #[must_use]
    pub fn with_severity(mut self, severity: DtcSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Get DTC as OBD-II style code (e.g., "P0123")
    #[must_use]
    pub fn as_obd_code(&self) -> String {
        if let Some(ref code) = self.display_code {
            return code.clone();
        }

        // Convert 3-byte DTC to OBD-II format
        let byte1 = ((self.code >> 16) & 0xFF) as u8;
        let byte2 = ((self.code >> 8) & 0xFF) as u8;
        let byte3 = (self.code & 0xFF) as u8;

        // First nibble determines category
        let category = match (byte1 >> 6) & 0x03 {
            1 => 'C', // Chassis
            2 => 'B', // Body
            3 => 'U', // Network
            _ => 'P', // Powertrain (0 and default)
        };

        // Second nibble
        let second = (byte1 >> 4) & 0x03;

        format!("{}{}{:02X}{:02X}", category, second, byte2, byte3)
    }
}


// ============================================================================
// Communication Parameters
// ============================================================================

/// Communication parameter specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComparamSpec {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Description
    pub description: Option<String>,
    /// Protocol type
    pub protocol_type: Option<String>,
    /// Communication parameters
    pub comparams: Vec<Comparam>,
}

/// Communication parameter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comparam {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Description
    pub description: Option<String>,
    /// Parameter type
    pub param_class: ComparamClass,
    /// Physical default value
    pub physical_default: Option<String>,
    /// Data object property reference
    pub dop_ref: Option<String>,
}

/// Communication parameter class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ComparamClass {
    /// Timing parameter
    #[default]
    Timing,
    /// Initialization parameter
    Init,
    /// Error handling parameter
    ErrorHandling,
    /// Unique response ID
    UniqueRespId,
    /// Bus type
    BusType,
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Vehicle information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VehicleInfo {
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Logical links
    pub logical_links: Vec<LogicalLink>,
}

/// Logical link (ECU connection)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogicalLink {
    /// Short name
    pub short_name: String,
    /// Description
    pub description: Option<String>,
    /// Protocol reference
    pub protocol_ref: Option<String>,
    /// Physical vehicle link reference
    pub physical_vehicle_link_ref: Option<String>,
}

/// ECU shared data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EcuSharedData {
    /// Short name
    pub short_name: String,
    /// Description
    pub description: Option<String>,
    /// Shared DOPs
    pub dops: Vec<DataObjectProperty>,
    /// Shared DTCs
    pub dtcs: Vec<Dtc>,
}

/// Functional group
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionalGroup {
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Functional class references
    pub functional_class_refs: Vec<String>,
}

/// Functional class
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionalClass {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Long name
    pub long_name: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Single ECU job
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SingleEcuJob {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Description
    pub description: Option<String>,
    /// Functional class references
    pub functional_class_refs: Vec<String>,
    /// Audience
    pub audience: Option<Audience>,
    /// Programming language
    pub prog_language: Option<String>,
    /// Code (script)
    pub code: Option<String>,
}

/// Audience (access level)
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Audience {
    /// Is development
    pub is_development: bool,
    /// Is manufacturing
    pub is_manufacturing: bool,
    /// Is supplier
    pub is_supplier: bool,
    /// Is aftersales
    pub is_aftersales: bool,
    /// Is aftermarket
    pub is_aftermarket: bool,
}

/// Data dictionary entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataDictionaryEntry {
    /// Unique identifier
    pub id: String,
    /// Short name
    pub short_name: String,
    /// Data object property
    pub dop: DataObjectProperty,
}


// ============================================================================
// ODX Parser
// ============================================================================

/// ODX Parser for parsing ODX XML files
pub struct OdxParser;

impl OdxParser {
    /// Parse ODX content from string
    ///
    /// # Errors
    /// Returns error if the ODX content is malformed
    pub fn parse(content: &str) -> Result<OdxDatabase, OdxParseError> {
        let mut parser = OdxParserInternal::new(content);
        parser.parse()
    }

    /// Parse ODX file
    ///
    /// # Errors
    /// Returns error if the file cannot be read or parsed
    pub fn parse_file(path: &str) -> Result<OdxDatabase, OdxParseError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| OdxParseError::IoError(e.to_string()))?;
        Self::parse(&content)
    }
}

/// ODX parse error
#[derive(Debug, Clone)]
pub enum OdxParseError {
    /// IO error
    IoError(String),
    /// XML parse error
    XmlError(String),
    /// Missing required element
    MissingElement(String),
    /// Invalid value
    InvalidValue(String),
}

impl std::fmt::Display for OdxParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::XmlError(msg) => write!(f, "XML error: {}", msg),
            Self::MissingElement(msg) => write!(f, "Missing element: {}", msg),
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for OdxParseError {}

/// Internal parser state
struct OdxParserInternal<'a> {
    content: &'a str,
    pos: usize,
}

impl<'a> OdxParserInternal<'a> {
    fn new(content: &'a str) -> Self {
        Self { content, pos: 0 }
    }

    fn parse(&mut self) -> Result<OdxDatabase, OdxParseError> {
        let mut db = OdxDatabase::new();

        // Skip XML declaration and whitespace
        self.skip_whitespace();
        self.skip_xml_declaration();
        self.skip_whitespace();

        // Parse root element
        if self.peek_str("<?") {
            self.skip_until("?>");
            self.skip_whitespace();
        }

        // Look for ODX-D, CATALOG, or other root elements
        if self.peek_str("<ODX") {
            self.parse_odx_root(&mut db)?;
        } else if self.peek_str("<CATALOG") {
            self.parse_catalog(&mut db)?;
        }

        Ok(db)
    }

    fn parse_odx_root(&mut self, db: &mut OdxDatabase) -> Result<(), OdxParseError> {
        // Skip opening tag
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</ODX") {
            self.skip_whitespace();

            if self.peek_str("<COMPARAM-SPEC") {
                if let Some(spec) = self.parse_comparam_spec()? {
                    db.comparam_specs.push(spec);
                }
            } else if self.peek_str("<DIAG-LAYER-CONTAINER") {
                self.parse_diag_layer_container(db)?;
            } else if self.peek_str("<VEHICLE-INFO") {
                db.vehicle_info = self.parse_vehicle_info()?;
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                // Skip unknown element
                self.skip_element();
            } else {
                self.advance();
            }
        }

        Ok(())
    }

    fn parse_catalog(&mut self, db: &mut OdxDatabase) -> Result<(), OdxParseError> {
        // Skip opening tag
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</CATALOG") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                db.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<LONG-NAME") {
                db.long_name = Some(self.parse_text_element("LONG-NAME")?);
            } else if self.peek_str("<DIAG-LAYER") && !self.peek_str("<DIAG-LAYER-CONTAINER") {
                if let Some(layer) = self.parse_diag_layer()? {
                    db.diag_layers.push(layer);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        Ok(())
    }

    fn parse_diag_layer_container(&mut self, db: &mut OdxDatabase) -> Result<(), OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</DIAG-LAYER-CONTAINER") {
            self.skip_whitespace();

            if self.peek_str("<PROTOCOL") || self.peek_str("<BASE-VARIANT") ||
               self.peek_str("<ECU-VARIANT") || self.peek_str("<FUNCTIONAL-GROUP") {
                if let Some(layer) = self.parse_diag_layer()? {
                    db.diag_layers.push(layer);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(())
    }

    fn parse_diag_layer(&mut self) -> Result<Option<DiagLayer>, OdxParseError> {
        // Determine layer type from tag
        let layer_type = if self.peek_str("<PROTOCOL") {
            DiagLayerType::Protocol
        } else if self.peek_str("<BASE-VARIANT") {
            DiagLayerType::BaseVariant
        } else if self.peek_str("<ECU-VARIANT") {
            DiagLayerType::EcuVariant
        } else if self.peek_str("<FUNCTIONAL-GROUP") {
            DiagLayerType::FunctionalGroup
        } else if self.peek_str("<DIAG-LAYER") {
            DiagLayerType::EcuVariant
        } else {
            return Ok(None);
        };

        // Get ID attribute
        let id = self.parse_attribute("ID").unwrap_or_default();
        self.skip_until(">");
        self.skip_whitespace();

        let mut layer = DiagLayer::new(&id, "", layer_type);

        // Find end tag
        let end_tag = match layer_type {
            DiagLayerType::Protocol => "</PROTOCOL>",
            DiagLayerType::BaseVariant => "</BASE-VARIANT>",
            DiagLayerType::EcuVariant => "</ECU-VARIANT>",
            DiagLayerType::FunctionalGroup => "</FUNCTIONAL-GROUP>",
            DiagLayerType::EcuSharedData => "</ECU-SHARED-DATA>",
        };

        while !self.is_at_end() && !self.peek_str(end_tag) {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                layer.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<LONG-NAME") {
                layer.long_name = Some(self.parse_text_element("LONG-NAME")?);
            } else if self.peek_str("<DESC") {
                layer.description = Some(self.parse_text_element("DESC")?);
            } else if self.peek_str("<DIAG-COMMS") {
                self.parse_diag_comms(&mut layer)?;
            } else if self.peek_str("<DIAG-SERVICES") {
                self.parse_diag_services(&mut layer)?;
            } else if self.peek_str("<ALL-DTC") {
                self.parse_dtcs(&mut layer, true)?;
            } else if self.peek_str("<DTCS") {
                self.parse_dtcs(&mut layer, false)?;
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(layer))
    }
}


impl OdxParserInternal<'_> {
    fn parse_diag_comms(&mut self, layer: &mut DiagLayer) -> Result<(), OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</DIAG-COMMS") {
            self.skip_whitespace();

            if self.peek_str("<DIAG-SERVICE") {
                if let Some(service) = self.parse_diag_service()? {
                    layer.services.push(service);
                }
            } else if self.peek_str("<SINGLE-ECU-JOB") {
                if let Some(job) = self.parse_single_ecu_job()? {
                    layer.single_ecu_jobs.push(job);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(())
    }

    fn parse_diag_services(&mut self, layer: &mut DiagLayer) -> Result<(), OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</DIAG-SERVICES") {
            self.skip_whitespace();

            if self.peek_str("<DIAG-SERVICE") {
                if let Some(service) = self.parse_diag_service()? {
                    layer.services.push(service);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(())
    }

    fn parse_diag_service(&mut self) -> Result<Option<DiagService>, OdxParseError> {
        let id = self.parse_attribute("ID").unwrap_or_default();
        self.skip_until(">");
        self.skip_whitespace();

        let mut service = DiagService::new(&id, "");

        while !self.is_at_end() && !self.peek_str("</DIAG-SERVICE") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                service.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<LONG-NAME") {
                service.long_name = Some(self.parse_text_element("LONG-NAME")?);
            } else if self.peek_str("<DESC") {
                service.description = Some(self.parse_text_element("DESC")?);
            } else if self.peek_str("<SEMANTIC") {
                service.semantic = Some(self.parse_text_element("SEMANTIC")?);
            } else if self.peek_str("<REQUEST-REF") || self.peek_str("<REQUEST") {
                service.request = self.parse_request()?;
            } else if self.peek_str("<POS-RESPONSE-REF") || self.peek_str("<POS-RESPONSE") {
                service.positive_response = self.parse_response()?;
            } else if self.peek_str("<NEG-RESPONSE-REF") || self.peek_str("<NEG-RESPONSE") {
                service.negative_response = self.parse_response()?;
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(service))
    }

    fn parse_request(&mut self) -> Result<Option<Request>, OdxParseError> {
        let id = self.parse_attribute("ID").unwrap_or_default();

        // Check for self-closing tag
        if self.peek_str("/>") {
            self.skip_until(">");
            return Ok(Some(Request::new(&id, "")));
        }

        self.skip_until(">");
        self.skip_whitespace();

        let mut request = Request::new(&id, "");

        while !self.is_at_end() && !self.peek_str("</REQUEST") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                request.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<PARAM") {
                if let Some(param) = self.parse_param()? {
                    request.params.push(param);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("</") {
                break;
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(request))
    }

    fn parse_response(&mut self) -> Result<Option<Response>, OdxParseError> {
        let id = self.parse_attribute("ID").unwrap_or_default();

        // Check for self-closing tag
        if self.peek_str("/>") {
            self.skip_until(">");
            return Ok(Some(Response::new(&id, "")));
        }

        self.skip_until(">");
        self.skip_whitespace();

        let mut response = Response::new(&id, "");

        while !self.is_at_end() && !self.peek_str("</POS-RESPONSE") && !self.peek_str("</NEG-RESPONSE") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                response.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<PARAM") {
                if let Some(param) = self.parse_param()? {
                    response.params.push(param);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("</") {
                break;
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(response))
    }

    fn parse_param(&mut self) -> Result<Option<Param>, OdxParseError> {
        // Get semantic attribute
        let semantic_str = self.parse_attribute("SEMANTIC").unwrap_or_default();
        let semantic = match semantic_str.to_uppercase().as_str() {
            "ID" | "SERVICE-ID" => ParamSemantic::ServiceId,
            "SUBFUNCTION" => ParamSemantic::SubFunction,
            _ => ParamSemantic::Data,
        };

        self.skip_until(">");
        self.skip_whitespace();

        let mut param = Param::new("", "", ParamType::Value);
        param.semantic = semantic;

        while !self.is_at_end() && !self.peek_str("</PARAM") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                param.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<BYTE-POSITION") {
                let pos_str = self.parse_text_element("BYTE-POSITION")?;
                param.byte_position = pos_str.parse().ok();
            } else if self.peek_str("<BIT-POSITION") {
                let pos_str = self.parse_text_element("BIT-POSITION")?;
                param.bit_position = pos_str.parse().ok();
            } else if self.peek_str("<CODED-VALUE") {
                let val_str = self.parse_text_element("CODED-VALUE")?;
                param.coded_value = Self::parse_number(&val_str);
                param.param_type = ParamType::CodedConst;
            } else if self.peek_str("<DOP-REF") {
                param.dop_ref = self.parse_attribute("ID-REF");
                self.skip_element();
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("</") {
                // Found a closing tag, break out of the loop
                break;
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(param))
    }

    fn parse_dtcs(&mut self, layer: &mut DiagLayer, is_all_dtc: bool) -> Result<(), OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        let end_tag = if is_all_dtc {
            "</ALL-DTC>"
        } else {
            "</DTCS>"
        };

        while !self.is_at_end() && !self.peek_str(end_tag) {
            self.skip_whitespace();

            // Check for DTC element but not DTCS
            if self.peek_str("<DTC ") || self.peek_str("<DTC>") {
                if let Some(dtc) = self.parse_dtc()? {
                    layer.dtcs.push(dtc);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(())
    }

    fn parse_dtc(&mut self) -> Result<Option<Dtc>, OdxParseError> {
        let id = self.parse_attribute("ID").unwrap_or_default();
        self.skip_until(">");
        self.skip_whitespace();

        let mut dtc = Dtc::new(&id, "", 0);

        while !self.is_at_end() && !self.peek_str("</DTC>") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                dtc.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<LONG-NAME") {
                dtc.long_name = Some(self.parse_text_element("LONG-NAME")?);
            } else if self.peek_str("<DESC") || self.peek_str("<TEXT") {
                dtc.text = Some(self.parse_text_element_any()?);
            } else if self.peek_str("<TROUBLE-CODE") {
                let code_str = self.parse_text_element("TROUBLE-CODE")?;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    dtc.code = Self::parse_number(&code_str).unwrap_or(0) as u32;
                }
            } else if self.peek_str("<DISPLAY-TROUBLE-CODE") {
                dtc.display_code = Some(self.parse_text_element("DISPLAY-TROUBLE-CODE")?);
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("</") {
                // Found a closing tag, break out of the loop
                break;
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(dtc))
    }
}


// ============================================================================
// Helper Methods
// ============================================================================

impl OdxParserInternal<'_> {
    fn parse_single_ecu_job(&mut self) -> Result<Option<SingleEcuJob>, OdxParseError> {
        let id = self.parse_attribute("ID").unwrap_or_default();
        self.skip_until(">");
        self.skip_whitespace();

        let mut job = SingleEcuJob {
            id,
            ..Default::default()
        };

        while !self.is_at_end() && !self.peek_str("</SINGLE-ECU-JOB") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                job.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<DESC") {
                job.description = Some(self.parse_text_element("DESC")?);
            } else if self.peek_str("<PROG-CODE") {
                self.parse_prog_code(&mut job)?;
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(job))
    }

    fn parse_prog_code(&mut self, job: &mut SingleEcuJob) -> Result<(), OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</PROG-CODE") {
            self.skip_whitespace();

            if self.peek_str("<CODE-FILE") {
                job.code = Some(self.parse_text_element("CODE-FILE")?);
            } else if self.peek_str("<SYNTAX") {
                job.prog_language = Some(self.parse_text_element("SYNTAX")?);
            } else if self.peek_str("<SOURCE-CODE") {
                job.code = Some(self.parse_text_element("SOURCE-CODE")?);
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(())
    }

    fn parse_comparam_spec(&mut self) -> Result<Option<ComparamSpec>, OdxParseError> {
        let id = self.parse_attribute("ID").unwrap_or_default();
        self.skip_until(">");
        self.skip_whitespace();

        let mut spec = ComparamSpec {
            id,
            ..Default::default()
        };

        while !self.is_at_end() && !self.peek_str("</COMPARAM-SPEC") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                spec.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<DESC") {
                spec.description = Some(self.parse_text_element("DESC")?);
            } else if self.peek_str("<PROT-STACK-SNREF") {
                spec.protocol_type = self.parse_attribute("SHORT-NAME");
                self.skip_element();
            } else if self.peek_str("<COMPARAMS") {
                self.parse_comparams(&mut spec)?;
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(spec))
    }

    fn parse_comparams(&mut self, spec: &mut ComparamSpec) -> Result<(), OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</COMPARAMS") {
            self.skip_whitespace();

            if self.peek_str("<COMPARAM") {
                if let Some(param) = self.parse_comparam()? {
                    spec.comparams.push(param);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(())
    }

    fn parse_comparam(&mut self) -> Result<Option<Comparam>, OdxParseError> {
        let id = self.parse_attribute("ID").unwrap_or_default();
        let param_class_str = self.parse_attribute("PARAM-CLASS").unwrap_or_default();

        let param_class = match param_class_str.to_uppercase().as_str() {
            "INIT" => ComparamClass::Init,
            "ERRHDL" | "ERROR-HANDLING" => ComparamClass::ErrorHandling,
            "UNIQUERESPID" | "UNIQUE-RESP-ID" => ComparamClass::UniqueRespId,
            "BUSTYPE" | "BUS-TYPE" => ComparamClass::BusType,
            _ => ComparamClass::Timing,
        };

        self.skip_until(">");
        self.skip_whitespace();

        let mut comparam = Comparam {
            id,
            param_class,
            ..Default::default()
        };

        while !self.is_at_end() && !self.peek_str("</COMPARAM") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                comparam.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<DESC") {
                comparam.description = Some(self.parse_text_element("DESC")?);
            } else if self.peek_str("<PHYSICAL-DEFAULT-VALUE") {
                comparam.physical_default = Some(self.parse_text_element("PHYSICAL-DEFAULT-VALUE")?);
            } else if self.peek_str("<DOP-REF") {
                comparam.dop_ref = self.parse_attribute("ID-REF");
                self.skip_element();
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(comparam))
    }

    fn parse_vehicle_info(&mut self) -> Result<Option<VehicleInfo>, OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        let mut info = VehicleInfo::default();

        while !self.is_at_end() && !self.peek_str("</VEHICLE-INFO") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                info.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<LONG-NAME") {
                info.long_name = Some(self.parse_text_element("LONG-NAME")?);
            } else if self.peek_str("<DESC") {
                info.description = Some(self.parse_text_element("DESC")?);
            } else if self.peek_str("<LOGICAL-LINKS") {
                self.parse_logical_links(&mut info)?;
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(info))
    }

    fn parse_logical_links(&mut self, info: &mut VehicleInfo) -> Result<(), OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        while !self.is_at_end() && !self.peek_str("</LOGICAL-LINKS") {
            self.skip_whitespace();

            if self.peek_str("<LOGICAL-LINK") {
                if let Some(link) = self.parse_logical_link()? {
                    info.logical_links.push(link);
                }
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(())
    }

    fn parse_logical_link(&mut self) -> Result<Option<LogicalLink>, OdxParseError> {
        self.skip_until(">");
        self.skip_whitespace();

        let mut link = LogicalLink::default();

        while !self.is_at_end() && !self.peek_str("</LOGICAL-LINK") {
            self.skip_whitespace();

            if self.peek_str("<SHORT-NAME") {
                link.short_name = self.parse_text_element("SHORT-NAME")?;
            } else if self.peek_str("<DESC") {
                link.description = Some(self.parse_text_element("DESC")?);
            } else if self.peek_str("<PROTOCOL-REF") {
                link.protocol_ref = self.parse_attribute("ID-REF");
                self.skip_element();
            } else if self.peek_str("<PHYSICAL-VEHICLE-LINK-REF") {
                link.physical_vehicle_link_ref = self.parse_attribute("ID-REF");
                self.skip_element();
            } else if self.peek_str("<!--") {
                self.skip_comment();
            } else if self.peek_str("<") {
                self.skip_element();
            } else {
                self.advance();
            }
        }

        self.skip_until(">");
        Ok(Some(link))
    }

    // ========================================================================
    // Low-level parsing helpers
    // ========================================================================

    fn skip_whitespace(&mut self) {
        while self.pos < self.content.len() {
            let ch = self.content.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_xml_declaration(&mut self) {
        if self.peek_str("<?xml") {
            self.skip_until("?>");
        }
    }

    fn skip_comment(&mut self) {
        if self.peek_str("<!--") {
            self.skip_until("-->");
        }
    }

    fn skip_element(&mut self) {
        // Skip opening tag name
        if !self.peek_str("<") {
            return;
        }
        self.advance(); // Skip '<'

        // Get tag name
        let start = self.pos;
        while self.pos < self.content.len() {
            let ch = self.content.as_bytes()[self.pos];
            if ch == b' ' || ch == b'>' || ch == b'/' {
                break;
            }
            self.pos += 1;
        }
        let tag_name = self.content[start..self.pos].to_string();

        // Skip to end of opening tag, checking for self-closing
        while !self.is_at_end() && !self.peek_str(">") {
            if self.peek_str("/>") {
                self.pos += 2; // Skip "/>"
                return; // Self-closing tag
            }
            self.advance();
        }
        
        if self.peek_str(">") {
            self.advance(); // Skip '>'
        }

        // Find matching end tag
        let end_tag = format!("</{}>", tag_name);
        let open_tag = format!("<{}", tag_name);
        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            if self.peek_str(&end_tag) {
                depth -= 1;
                if depth == 0 {
                    self.skip_until(">");
                    return;
                }
                self.advance();
            } else if self.peek_str(&open_tag) {
                // Check if it's the same tag (not just a prefix match)
                let after_tag = self.pos + tag_name.len() + 1;
                if after_tag < self.content.len() {
                    let ch = self.content.as_bytes()[after_tag];
                    if ch == b' ' || ch == b'>' || ch == b'/' {
                        depth += 1;
                    }
                }
                self.advance();
            } else {
                self.advance();
            }
        }
    }

    fn skip_until(&mut self, pattern: &str) {
        while !self.is_at_end() && !self.peek_str(pattern) {
            self.advance();
        }
        // Skip past the pattern
        if self.peek_str(pattern) {
            self.pos += pattern.len();
        }
    }

    fn peek_str(&self, s: &str) -> bool {
        if self.pos + s.len() > self.content.len() {
            return false;
        }
        &self.content[self.pos..self.pos + s.len()] == s
    }

    fn advance(&mut self) {
        if self.pos < self.content.len() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.content.len()
    }

    fn parse_attribute(&mut self, name: &str) -> Option<String> {
        // Save position
        let saved_pos = self.pos;

        // Look for attribute in current tag
        while !self.is_at_end() && !self.peek_str(">") {
            self.skip_whitespace();

            // Check if this is the attribute we're looking for
            let attr_pattern = format!("{}=", name);
            if self.peek_str(&attr_pattern) {
                self.pos += attr_pattern.len();
                self.skip_whitespace();

                // Get quote character
                if self.is_at_end() {
                    break;
                }
                let quote = self.content.as_bytes()[self.pos];
                if quote != b'"' && quote != b'\'' {
                    break;
                }
                self.advance(); // Skip opening quote

                // Read value
                let start = self.pos;
                while !self.is_at_end() && self.content.as_bytes()[self.pos] != quote {
                    self.advance();
                }
                let value = self.content[start..self.pos].to_string();
                self.advance(); // Skip closing quote

                return Some(value);
            }

            self.advance();
        }

        // Restore position
        self.pos = saved_pos;
        None
    }

    #[allow(clippy::unnecessary_wraps)]
    fn parse_text_element(&mut self, tag_name: &str) -> Result<String, OdxParseError> {
        // Skip to end of opening tag
        self.skip_until(">");
        self.skip_whitespace();

        // Read text content
        let start = self.pos;
        let end_tag = format!("</{}>", tag_name);

        while !self.is_at_end() && !self.peek_str(&end_tag) && !self.peek_str("<") {
            self.advance();
        }

        let text = self.content[start..self.pos].trim().to_string();

        // Skip any nested elements
        while self.peek_str("<") && !self.peek_str(&end_tag) {
            self.skip_element();
            self.skip_whitespace();
        }

        // Skip end tag
        if self.peek_str(&end_tag) {
            self.skip_until(">");
        }

        Ok(text)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn parse_text_element_any(&mut self) -> Result<String, OdxParseError> {
        // Skip to end of opening tag
        self.skip_until(">");
        self.skip_whitespace();

        // Read text content until we hit any closing tag
        let start = self.pos;

        while !self.is_at_end() && !self.peek_str("</") {
            if self.peek_str("<") && !self.peek_str("</") {
                // Skip nested element
                self.skip_element();
            } else {
                self.advance();
            }
        }

        let text = self.content[start..self.pos].trim().to_string();

        // Skip end tag
        self.skip_until(">");

        Ok(text)
    }

    fn parse_number(s: &str) -> Option<i64> {
        let s = s.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            i64::from_str_radix(&s[2..], 16).ok()
        } else if s.starts_with("0b") || s.starts_with("0B") {
            i64::from_str_radix(&s[2..], 2).ok()
        } else {
            s.parse().ok()
        }
    }
}


// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_odx_database_new() {
        let db = OdxDatabase::new();
        assert!(db.short_name.is_empty());
        assert!(db.diag_layers.is_empty());
    }

    #[test]
    fn test_diag_layer_type_from_str() {
        assert_eq!(DiagLayerType::parse("PROTOCOL"), Some(DiagLayerType::Protocol));
        assert_eq!(DiagLayerType::parse("ECU-VARIANT"), Some(DiagLayerType::EcuVariant));
        assert_eq!(DiagLayerType::parse("BASE-VARIANT"), Some(DiagLayerType::BaseVariant));
        assert_eq!(DiagLayerType::parse("FUNCTIONAL-GROUP"), Some(DiagLayerType::FunctionalGroup));
        assert_eq!(DiagLayerType::parse("unknown"), None);
    }

    #[test]
    fn test_diag_layer_new() {
        let layer = DiagLayer::new("layer1", "TestLayer", DiagLayerType::EcuVariant);
        assert_eq!(layer.id, "layer1");
        assert_eq!(layer.short_name, "TestLayer");
        assert_eq!(layer.layer_type, DiagLayerType::EcuVariant);
    }

    #[test]
    fn test_diag_service_new() {
        let service = DiagService::new("svc1", "ReadDID")
            .with_service_id(0x22);
        assert_eq!(service.id, "svc1");
        assert_eq!(service.short_name, "ReadDID");
        assert_eq!(service.service_id, Some(0x22));
    }

    #[test]
    fn test_request_byte_length() {
        let mut request = Request::new("req1", "TestRequest");
        request.add_param(Param::coded_const("p1", "SID", 0x22, 0));
        request.add_param(Param::value("p2", "DID", 1, 2));
        assert_eq!(request.byte_length(), 3);
    }

    #[test]
    fn test_param_coded_const() {
        let param = Param::coded_const("p1", "ServiceID", 0x22, 0);
        assert_eq!(param.param_type, ParamType::CodedConst);
        assert_eq!(param.coded_value, Some(0x22));
        assert_eq!(param.byte_position, Some(0));
    }

    #[test]
    fn test_param_value() {
        let param = Param::value("p1", "Data", 1, 4);
        assert_eq!(param.param_type, ParamType::Value);
        assert_eq!(param.byte_position, Some(1));
        assert_eq!(param.byte_size, Some(4));
    }

    #[test]
    fn test_dtc_new() {
        let dtc = Dtc::new("dtc1", "EngineOverheat", 0x010203)
            .with_display_code("P0123")
            .with_text("Engine temperature too high")
            .with_severity(DtcSeverity::CheckImmediately);

        assert_eq!(dtc.code, 0x010203);
        assert_eq!(dtc.display_code, Some("P0123".to_string()));
        assert_eq!(dtc.severity, DtcSeverity::CheckImmediately);
    }

    #[test]
    fn test_dtc_as_obd_code() {
        // Test with display code set
        let dtc1 = Dtc::new("dtc1", "Test", 0).with_display_code("P0420");
        assert_eq!(dtc1.as_obd_code(), "P0420");

        // Test code conversion (powertrain)
        let dtc2 = Dtc::new("dtc2", "Test", 0x001234);
        let code = dtc2.as_obd_code();
        assert!(code.starts_with('P'));
    }

    #[test]
    fn test_parse_empty_odx() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<ODX version="2.2.0">
</ODX>"#;

        let result = OdxParser::parse(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_odx() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<ODX version="2.2.0">
  <DIAG-LAYER-CONTAINER>
    <ECU-VARIANT ID="EV_Test">
      <SHORT-NAME>TestECU</SHORT-NAME>
      <LONG-NAME>Test ECU Variant</LONG-NAME>
      <DIAG-SERVICES>
        <DIAG-SERVICE ID="DS_ReadDID">
          <SHORT-NAME>ReadDataByIdentifier</SHORT-NAME>
          <SEMANTIC>READ</SEMANTIC>
        </DIAG-SERVICE>
      </DIAG-SERVICES>
    </ECU-VARIANT>
  </DIAG-LAYER-CONTAINER>
</ODX>"#;

        let result = OdxParser::parse(content);
        assert!(result.is_ok());

        let db = result.unwrap();
        assert_eq!(db.diag_layers.len(), 1);

        let layer = &db.diag_layers[0];
        assert_eq!(layer.short_name, "TestECU");
        assert_eq!(layer.layer_type, DiagLayerType::EcuVariant);
        assert_eq!(layer.services.len(), 1);

        let service = &layer.services[0];
        assert_eq!(service.short_name, "ReadDataByIdentifier");
        assert_eq!(service.semantic, Some("READ".to_string()));
    }

    #[test]
    fn test_parse_odx_with_dtcs() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<ODX version="2.2.0">
  <DIAG-LAYER-CONTAINER>
    <ECU-VARIANT ID="EV_Test">
      <SHORT-NAME>TestECU</SHORT-NAME>
      <DTCS>
        <DTC ID="DTC_P0100">
          <SHORT-NAME>MassAirFlowCircuit</SHORT-NAME>
          <TROUBLE-CODE>0x010000</TROUBLE-CODE>
          <DISPLAY-TROUBLE-CODE>P0100</DISPLAY-TROUBLE-CODE>
        </DTC>
        <DTC ID="DTC_P0101">
          <SHORT-NAME>MassAirFlowRange</SHORT-NAME>
          <TROUBLE-CODE>0x010100</TROUBLE-CODE>
          <DISPLAY-TROUBLE-CODE>P0101</DISPLAY-TROUBLE-CODE>
        </DTC>
      </DTCS>
    </ECU-VARIANT>
  </DIAG-LAYER-CONTAINER>
</ODX>"#;

        let result = OdxParser::parse(content);
        assert!(result.is_ok());

        let db = result.unwrap();
        assert_eq!(db.diag_layers.len(), 1);

        let layer = &db.diag_layers[0];
        assert_eq!(layer.dtcs.len(), 2);

        let dtc1 = &layer.dtcs[0];
        assert_eq!(dtc1.short_name, "MassAirFlowCircuit");
        assert_eq!(dtc1.display_code, Some("P0100".to_string()));
    }

    #[test]
    fn test_parse_odx_with_request_response() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<ODX version="2.2.0">
  <DIAG-LAYER-CONTAINER>
    <ECU-VARIANT ID="EV_Test">
      <SHORT-NAME>TestECU</SHORT-NAME>
      <DIAG-SERVICES>
        <DIAG-SERVICE ID="DS_ReadDID">
          <SHORT-NAME>ReadDID</SHORT-NAME>
          <REQUEST ID="RQ_ReadDID">
            <SHORT-NAME>ReadDID_Request</SHORT-NAME>
            <PARAM SEMANTIC="SERVICE-ID">
              <SHORT-NAME>SID</SHORT-NAME>
              <BYTE-POSITION>0</BYTE-POSITION>
              <CODED-VALUE>0x22</CODED-VALUE>
            </PARAM>
          </REQUEST>
          <POS-RESPONSE ID="PR_ReadDID">
            <SHORT-NAME>ReadDID_Response</SHORT-NAME>
            <PARAM SEMANTIC="SERVICE-ID">
              <SHORT-NAME>SID</SHORT-NAME>
              <BYTE-POSITION>0</BYTE-POSITION>
              <CODED-VALUE>0x62</CODED-VALUE>
            </PARAM>
          </POS-RESPONSE>
        </DIAG-SERVICE>
      </DIAG-SERVICES>
    </ECU-VARIANT>
  </DIAG-LAYER-CONTAINER>
</ODX>"#;

        let result = OdxParser::parse(content);
        assert!(result.is_ok());

        let db = result.unwrap();
        let service = &db.diag_layers[0].services[0];

        assert!(service.request.is_some());
        let request = service.request.as_ref().unwrap();
        assert_eq!(request.short_name, "ReadDID_Request");
        assert_eq!(request.params.len(), 1);
        assert_eq!(request.params[0].coded_value, Some(0x22));

        assert!(service.positive_response.is_some());
        let response = service.positive_response.as_ref().unwrap();
        assert_eq!(response.short_name, "ReadDID_Response");
    }

    #[test]
    fn test_database_find_service() {
        let mut db = OdxDatabase::new();
        let mut layer = DiagLayer::new("l1", "Layer1", DiagLayerType::EcuVariant);
        layer.add_service(DiagService::new("s1", "ReadDID"));
        layer.add_service(DiagService::new("s2", "WriteDID"));
        db.diag_layers.push(layer);

        assert!(db.find_service("ReadDID").is_some());
        assert!(db.find_service("WriteDID").is_some());
        assert!(db.find_service("Unknown").is_none());
    }

    #[test]
    fn test_database_find_dtc() {
        let mut db = OdxDatabase::new();
        let mut layer = DiagLayer::new("l1", "Layer1", DiagLayerType::EcuVariant);
        layer.add_dtc(Dtc::new("d1", "DTC1", 0x010203));
        layer.add_dtc(Dtc::new("d2", "DTC2", 0x040506));
        db.diag_layers.push(layer);

        assert!(db.find_dtc(0x010203).is_some());
        assert!(db.find_dtc(0x040506).is_some());
        assert!(db.find_dtc(0x999999).is_none());
    }

    #[test]
    fn test_database_all_services() {
        let mut db = OdxDatabase::new();

        let mut layer1 = DiagLayer::new("l1", "Layer1", DiagLayerType::EcuVariant);
        layer1.add_service(DiagService::new("s1", "Service1"));
        layer1.add_service(DiagService::new("s2", "Service2"));

        let mut layer2 = DiagLayer::new("l2", "Layer2", DiagLayerType::BaseVariant);
        layer2.add_service(DiagService::new("s3", "Service3"));

        db.diag_layers.push(layer1);
        db.diag_layers.push(layer2);

        let all_services = db.all_services();
        assert_eq!(all_services.len(), 3);
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(OdxParserInternal::parse_number("123"), Some(123));
        assert_eq!(OdxParserInternal::parse_number("0x22"), Some(0x22));
        assert_eq!(OdxParserInternal::parse_number("0X3F"), Some(0x3F));
        assert_eq!(OdxParserInternal::parse_number("0b1010"), Some(10));
        assert_eq!(OdxParserInternal::parse_number("  42  "), Some(42));
        assert_eq!(OdxParserInternal::parse_number("invalid"), None);
    }

    #[test]
    fn test_data_object_property() {
        let dop = DataObjectProperty::new("dop1", "EngineSpeed");
        assert_eq!(dop.id, "dop1");
        assert_eq!(dop.short_name, "EngineSpeed");
    }

    #[test]
    fn test_comparam_spec() {
        let spec = ComparamSpec {
            id: "cp1".to_string(),
            short_name: "ISO15765".to_string(),
            protocol_type: Some("ISO_15765_3".to_string()),
            ..Default::default()
        };
        assert_eq!(spec.short_name, "ISO15765");
    }
}
