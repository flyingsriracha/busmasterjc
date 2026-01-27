//! AUTOSAR XML (ARXML) Parser
//!
//! This module provides a parser for AUTOSAR XML files (.arxml).
//! ARXML is the standard exchange format for AUTOSAR system descriptions.
//!
//! # Format Overview
//!
//! ARXML files are XML documents with the root element `<AUTOSAR>`.
//! Key elements for CAN communication include:
//! - `<AR-PACKAGES>` - Container for packages
//! - `<AR-PACKAGE>` - Package containing elements
//! - `<FRAME>` - CAN frame definition
//! - `<I-SIGNAL-I-PDU>` - Signal-based PDU
//! - `<I-SIGNAL>` - Signal definition
//! - `<COMPU-METHOD>` - Conversion method
//!
//! # Example
//!
//! ```
//! use busmaster_db::arxml::ArxmlParser;
//!
//! let arxml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
//! <AUTOSAR xmlns="http://autosar.org/schema/r4.0">
//!   <AR-PACKAGES>
//!     <AR-PACKAGE>
//!       <SHORT-NAME>TestPackage</SHORT-NAME>
//!     </AR-PACKAGE>
//!   </AR-PACKAGES>
//! </AUTOSAR>
//! "#;
//!
//! let database = ArxmlParser::parse(arxml_content).unwrap();
//! assert_eq!(database.packages.len(), 1);
//! ```

use std::fmt;

/// Error type for ARXML parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ArxmlParseError {
    /// Invalid XML format
    InvalidXml(String),
    /// Missing required element
    MissingElement(String),
    /// Invalid value
    InvalidValue(String),
    /// Unsupported AUTOSAR version
    UnsupportedVersion(String),
}

impl fmt::Display for ArxmlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(msg) => write!(f, "Invalid XML: {}", msg),
            Self::MissingElement(elem) => write!(f, "Missing element: {}", elem),
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Self::UnsupportedVersion(ver) => write!(f, "Unsupported AUTOSAR version: {}", ver),
        }
    }
}

impl std::error::Error for ArxmlParseError {}

/// AUTOSAR version
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutosarVersion {
    /// Major version
    pub major: u8,
    /// Minor version
    pub minor: u8,
    /// Patch version
    pub patch: u8,
}

impl AutosarVersion {
    /// Create a new version
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch }
    }
    
    /// Parse version from schema URL
    pub fn from_schema_url(url: &str) -> Option<Self> {
        // Extract version from URLs like "http://autosar.org/schema/r4.0"
        if let Some(pos) = url.rfind("/r") {
            let version_str = &url[pos + 2..];
            let parts: Vec<&str> = version_str.split('.').collect();
            if !parts.is_empty() {
                let major = parts[0].parse().ok()?;
                let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                return Some(Self { major, minor, patch });
            }
        }
        None
    }
}

impl fmt::Display for AutosarVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Byte order for signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArxmlByteOrder {
    /// Most significant byte first (big-endian)
    #[default]
    MostSignificantByteFirst,
    /// Most significant byte last (little-endian)
    MostSignificantByteLast,
    /// Opaque (no byte order)
    Opaque,
}

impl ArxmlByteOrder {
    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "MOST-SIGNIFICANT-BYTE-LAST" => Self::MostSignificantByteLast,
            "OPAQUE" => Self::Opaque,
            _ => Self::MostSignificantByteFirst,
        }
    }
}

/// Base data type category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseTypeCategory {
    /// Fixed length type
    #[default]
    FixedLength,
    /// Variable length type
    VariableLength,
}

/// Base data type encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseTypeEncoding {
    /// No encoding
    #[default]
    None,
    /// Signed encoding
    Signed,
    /// Unsigned encoding
    Unsigned,
    /// IEEE 754 floating point
    Ieee754,
    /// Boolean
    Boolean,
}

impl BaseTypeEncoding {
    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "SIGNED" => Self::Signed,
            "UNSIGNED" | "2C" => Self::Unsigned,
            "IEEE754" => Self::Ieee754,
            "BOOLEAN" => Self::Boolean,
            _ => Self::None,
        }
    }
}

/// SW Base Type - defines the physical encoding of data
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SwBaseType {
    /// Short name
    pub short_name: String,
    /// Category
    pub category: BaseTypeCategory,
    /// Size in bits
    pub base_type_size: u32,
    /// Encoding
    pub base_type_encoding: BaseTypeEncoding,
    /// Native declaration (e.g., "uint8")
    pub native_declaration: String,
}

impl SwBaseType {
    /// Create a new base type
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
}

/// Computation method category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompuMethodCategory {
    /// Identical (no conversion)
    #[default]
    Identical,
    /// Linear conversion (y = ax + b)
    Linear,
    /// Scale-linear conversion
    ScaleLinear,
    /// Text table (enumeration)
    TextTable,
    /// Tab-no-interpolation
    TabNoInterpretation,
    /// Rational function
    RatFunc,
    /// Scale-rational function
    ScaleRatFunc,
}

impl CompuMethodCategory {
    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "LINEAR" => Self::Linear,
            "SCALE-LINEAR" => Self::ScaleLinear,
            "TEXTTABLE" => Self::TextTable,
            "TAB-NOINTP" => Self::TabNoInterpretation,
            "RAT-FUNC" => Self::RatFunc,
            "SCALE-RAT-FUNC" => Self::ScaleRatFunc,
            _ => Self::Identical,
        }
    }
}

/// Computation scale for linear conversion
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompuScale {
    /// Lower limit
    pub lower_limit: f64,
    /// Upper limit
    pub upper_limit: f64,
    /// Numerator coefficients
    pub numerator: Vec<f64>,
    /// Denominator coefficients
    pub denominator: Vec<f64>,
    /// Text value (for text tables)
    pub vt: String,
}

impl CompuScale {
    /// Create a linear scale (y = ax + b)
    pub fn linear(factor: f64, offset: f64) -> Self {
        Self {
            numerator: vec![offset, factor],
            denominator: vec![1.0],
            ..Default::default()
        }
    }
    
    /// Create a text table entry
    pub fn text_entry(value: f64, text: impl Into<String>) -> Self {
        Self {
            lower_limit: value,
            upper_limit: value,
            vt: text.into(),
            ..Default::default()
        }
    }
}

/// Computation method - defines how to convert raw to physical values
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompuMethod {
    /// Short name
    pub short_name: String,
    /// Category
    pub category: CompuMethodCategory,
    /// Unit reference
    pub unit_ref: String,
    /// Computation scales
    pub compu_scales: Vec<CompuScale>,
}

impl CompuMethod {
    /// Create a new computation method
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
    
    /// Create an identical (no conversion) method
    pub fn identical(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            category: CompuMethodCategory::Identical,
            ..Default::default()
        }
    }
    
    /// Create a linear conversion method
    pub fn linear(name: impl Into<String>, factor: f64, offset: f64) -> Self {
        Self {
            short_name: name.into(),
            category: CompuMethodCategory::Linear,
            compu_scales: vec![CompuScale::linear(factor, offset)],
            ..Default::default()
        }
    }
}

/// Unit definition
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Unit {
    /// Short name
    pub short_name: String,
    /// Display name
    pub display_name: String,
    /// Factor to SI unit
    pub factor_si_to_unit: f64,
    /// Offset to SI unit
    pub offset_si_to_unit: f64,
}

impl Unit {
    /// Create a new unit
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            factor_si_to_unit: 1.0,
            ..Default::default()
        }
    }
}

/// I-Signal definition
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ISignal {
    /// Short name
    pub short_name: String,
    /// Length in bits
    pub i_signal_length: u32,
    /// Init value
    pub init_value: Option<f64>,
    /// System signal reference
    pub system_signal_ref: String,
    /// Computation method reference
    pub compu_method_ref: String,
    /// Base type reference
    pub base_type_ref: String,
}

impl ISignal {
    /// Create a new I-Signal
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
}

/// I-Signal to PDU mapping
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ISignalToPduMapping {
    /// Short name
    pub short_name: String,
    /// I-Signal reference
    pub i_signal_ref: String,
    /// Packing byte order
    pub packing_byte_order: ArxmlByteOrder,
    /// Start position in bits
    pub start_position: u32,
}

impl ISignalToPduMapping {
    /// Create a new mapping
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
}

/// I-Signal I-PDU (Signal-based Protocol Data Unit)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ISignalIPdu {
    /// Short name
    pub short_name: String,
    /// Length in bytes
    pub length: u32,
    /// Signal to PDU mappings
    pub i_signal_to_pdu_mappings: Vec<ISignalToPduMapping>,
}

impl ISignalIPdu {
    /// Create a new I-Signal I-PDU
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
    
    /// Add a signal mapping
    pub fn add_signal_mapping(&mut self, mapping: ISignalToPduMapping) {
        self.i_signal_to_pdu_mappings.push(mapping);
    }
}

/// Frame triggering type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameTriggeringType {
    /// CAN frame triggering
    #[default]
    Can,
    /// LIN frame triggering
    Lin,
    /// FlexRay frame triggering
    FlexRay,
    /// Ethernet frame triggering
    Ethernet,
}

/// CAN Frame triggering
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanFrameTriggering {
    /// Short name
    pub short_name: String,
    /// CAN ID
    pub identifier: u32,
    /// Frame reference
    pub frame_ref: String,
    /// PDU triggerings
    pub pdu_triggerings: Vec<String>,
}

impl CanFrameTriggering {
    /// Create a new CAN frame triggering
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
}

/// CAN Frame definition
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanFrame {
    /// Short name
    pub short_name: String,
    /// Frame length in bytes
    pub frame_length: u8,
    /// PDU to frame mappings
    pub pdu_to_frame_mappings: Vec<PduToFrameMapping>,
}

impl CanFrame {
    /// Create a new CAN frame
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            frame_length: 8,
            ..Default::default()
        }
    }
}

/// PDU to Frame mapping
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PduToFrameMapping {
    /// Short name
    pub short_name: String,
    /// Packing byte order
    pub packing_byte_order: ArxmlByteOrder,
    /// PDU reference
    pub pdu_ref: String,
    /// Start position in bits
    pub start_position: u32,
}

impl PduToFrameMapping {
    /// Create a new mapping
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
}

/// ECU Instance
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EcuInstance {
    /// Short name
    pub short_name: String,
    /// COM I-PDU groups
    pub com_i_pdu_groups: Vec<String>,
}

impl EcuInstance {
    /// Create a new ECU instance
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
}

/// CAN Cluster
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanCluster {
    /// Short name
    pub short_name: String,
    /// Baudrate in bits/second
    pub baudrate: u32,
    /// CAN FD baudrate
    pub can_fd_baudrate: Option<u32>,
    /// Physical channels
    pub physical_channels: Vec<CanPhysicalChannel>,
}

impl CanCluster {
    /// Create a new CAN cluster
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            baudrate: 500_000,
            ..Default::default()
        }
    }
}

/// CAN Physical Channel
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanPhysicalChannel {
    /// Short name
    pub short_name: String,
    /// Frame triggerings
    pub frame_triggerings: Vec<CanFrameTriggering>,
}

impl CanPhysicalChannel {
    /// Create a new physical channel
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
}

/// AUTOSAR Package
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArPackage {
    /// Short name
    pub short_name: String,
    /// Sub-packages
    pub sub_packages: Vec<ArPackage>,
    /// I-Signals
    pub i_signals: Vec<ISignal>,
    /// I-Signal I-PDUs
    pub i_signal_i_pdus: Vec<ISignalIPdu>,
    /// CAN Frames
    pub can_frames: Vec<CanFrame>,
    /// CAN Clusters
    pub can_clusters: Vec<CanCluster>,
    /// ECU Instances
    pub ecu_instances: Vec<EcuInstance>,
    /// Computation methods
    pub compu_methods: Vec<CompuMethod>,
    /// Units
    pub units: Vec<Unit>,
    /// Base types
    pub base_types: Vec<SwBaseType>,
}

impl ArPackage {
    /// Create a new package
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            short_name: name.into(),
            ..Default::default()
        }
    }
    
    /// Add a sub-package
    pub fn add_sub_package(&mut self, package: ArPackage) {
        self.sub_packages.push(package);
    }
    
    /// Find a sub-package by name
    pub fn find_package(&self, name: &str) -> Option<&ArPackage> {
        if self.short_name == name {
            return Some(self);
        }
        for pkg in &self.sub_packages {
            if let Some(found) = pkg.find_package(name) {
                return Some(found);
            }
        }
        None
    }
}

/// ARXML Database - root container for parsed ARXML data
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArxmlDatabase {
    /// AUTOSAR version
    pub version: AutosarVersion,
    /// Schema location
    pub schema_location: String,
    /// Root packages
    pub packages: Vec<ArPackage>,
}

impl ArxmlDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a package
    pub fn add_package(&mut self, package: ArPackage) {
        self.packages.push(package);
    }
    
    /// Find a package by path (e.g., "/Package1/SubPackage")
    pub fn find_package_by_path(&self, path: &str) -> Option<&ArPackage> {
        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();
        if parts.is_empty() {
            return None;
        }
        
        // Find root package
        let root_name = parts[0];
        let mut current = self.packages.iter().find(|p| p.short_name == root_name)?;
        
        // Navigate to sub-packages
        for name in parts.iter().skip(1) {
            current = current.sub_packages.iter().find(|p| p.short_name == *name)?;
        }
        
        Some(current)
    }
    
    /// Get all I-Signals from all packages
    pub fn all_signals(&self) -> Vec<&ISignal> {
        let mut signals = Vec::new();
        for pkg in &self.packages {
            Self::collect_signals(pkg, &mut signals);
        }
        signals
    }
    
    fn collect_signals<'a>(pkg: &'a ArPackage, signals: &mut Vec<&'a ISignal>) {
        signals.extend(pkg.i_signals.iter());
        for sub in &pkg.sub_packages {
            Self::collect_signals(sub, signals);
        }
    }
    
    /// Get all I-Signal I-PDUs from all packages
    pub fn all_pdus(&self) -> Vec<&ISignalIPdu> {
        let mut pdus = Vec::new();
        for pkg in &self.packages {
            Self::collect_pdus(pkg, &mut pdus);
        }
        pdus
    }
    
    fn collect_pdus<'a>(pkg: &'a ArPackage, pdus: &mut Vec<&'a ISignalIPdu>) {
        pdus.extend(pkg.i_signal_i_pdus.iter());
        for sub in &pkg.sub_packages {
            Self::collect_pdus(sub, pdus);
        }
    }
    
    /// Get all CAN Frames from all packages
    pub fn all_frames(&self) -> Vec<&CanFrame> {
        let mut frames = Vec::new();
        for pkg in &self.packages {
            Self::collect_frames(pkg, &mut frames);
        }
        frames
    }
    
    fn collect_frames<'a>(pkg: &'a ArPackage, frames: &mut Vec<&'a CanFrame>) {
        frames.extend(pkg.can_frames.iter());
        for sub in &pkg.sub_packages {
            Self::collect_frames(sub, frames);
        }
    }
}

/// ARXML Parser
pub struct ArxmlParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> ArxmlParser<'a> {
    /// Create a new parser
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
    
    /// Parse the input and return a database
    pub fn parse(input: &str) -> Result<ArxmlDatabase, ArxmlParseError> {
        let mut parser = ArxmlParser::new(input);
        parser.parse_database()
    }
    
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
    
    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }
    
    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.pos += c.len_utf8();
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn peek_str(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }
    
    fn skip_xml_declaration(&mut self) {
        self.skip_whitespace();
        if self.peek_str("<?xml") {
            while !self.is_eof() && !self.peek_str("?>") {
                self.advance();
            }
            if self.peek_str("?>") {
                self.advance();
                self.advance();
            }
        }
    }
    
    fn skip_comment(&mut self) {
        if self.peek_str("<!--") {
            while !self.is_eof() && !self.peek_str("-->") {
                self.advance();
            }
            if self.peek_str("-->") {
                self.advance();
                self.advance();
                self.advance();
            }
        }
    }
    
    fn read_tag_name(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ':' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }
    
    fn read_attribute_value(&mut self) -> String {
        self.skip_whitespace();
        if self.current_char() != Some('"') && self.current_char() != Some('\'') {
            return String::new();
        }
        let quote = self.current_char().unwrap();
        self.advance();
        
        let start = self.pos;
        while let Some(c) = self.current_char() {
            if c == quote {
                break;
            }
            self.advance();
        }
        let value = self.input[start..self.pos].to_string();
        if self.current_char() == Some(quote) {
            self.advance();
        }
        value
    }

    fn read_text_content(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.current_char() {
            if c == '<' {
                break;
            }
            self.advance();
        }
        self.input[start..self.pos].trim().to_string()
    }
    
    fn parse_database(&mut self) -> Result<ArxmlDatabase, ArxmlParseError> {
        let mut db = ArxmlDatabase::new();
        
        self.skip_xml_declaration();
        self.skip_whitespace();
        
        // Skip comments
        while self.peek_str("<!--") {
            self.skip_comment();
            self.skip_whitespace();
        }
        
        // Find AUTOSAR root element
        if !self.peek_str("<AUTOSAR") {
            return Err(ArxmlParseError::MissingElement("AUTOSAR".to_string()));
        }
        
        // Parse AUTOSAR element attributes
        self.advance(); // skip '<'
        self.read_tag_name(); // skip "AUTOSAR"
        
        // Read attributes
        loop {
            self.skip_whitespace();
            if self.current_char() == Some('>') || self.peek_str("/>") {
                break;
            }
            
            let attr_name = self.read_tag_name();
            self.skip_whitespace();
            if self.current_char() == Some('=') {
                self.advance();
                let attr_value = self.read_attribute_value();
                
                if attr_name == "xmlns" || attr_name.starts_with("xmlns:") {
                    if let Some(version) = AutosarVersion::from_schema_url(&attr_value) {
                        db.version = version;
                    }
                } else if attr_name.contains("schemaLocation") {
                    db.schema_location = attr_value;
                }
            }
        }
        
        // Skip to content
        if self.current_char() == Some('>') {
            self.advance();
        }
        
        // Parse content
        self.parse_autosar_content(&mut db)?;
        
        Ok(db)
    }
    
    fn parse_autosar_content(&mut self, db: &mut ArxmlDatabase) -> Result<(), ArxmlParseError> {
        loop {
            self.skip_whitespace();
            
            if self.is_eof() || self.peek_str("</AUTOSAR") {
                break;
            }
            
            if self.peek_str("<!--") {
                self.skip_comment();
                continue;
            }
            
            if self.peek_str("<AR-PACKAGES") {
                self.parse_ar_packages(db)?;
            } else if self.current_char() == Some('<') {
                // Skip unknown element
                self.skip_element();
            } else {
                self.advance();
            }
        }
        Ok(())
    }

    fn parse_ar_packages(&mut self, db: &mut ArxmlDatabase) -> Result<(), ArxmlParseError> {
        // Skip <AR-PACKAGES...>
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</AR-PACKAGES") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<!--") {
                self.skip_comment();
                continue;
            }
            
            if self.peek_str("<AR-PACKAGE") {
                let pkg = self.parse_ar_package()?;
                db.add_package(pkg);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        Ok(())
    }
    
    fn parse_ar_package(&mut self) -> Result<ArPackage, ArxmlParseError> {
        let mut pkg = ArPackage::new("");
        
        // Skip <AR-PACKAGE...>
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</AR-PACKAGE") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<!--") {
                self.skip_comment();
                continue;
            }
            
            if self.peek_str("<SHORT-NAME") {
                pkg.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<AR-PACKAGES") {
                self.parse_sub_packages(&mut pkg)?;
            } else if self.peek_str("<ELEMENTS") {
                self.parse_elements(&mut pkg);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        Ok(pkg)
    }
    
    fn parse_sub_packages(&mut self, pkg: &mut ArPackage) -> Result<(), ArxmlParseError> {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</AR-PACKAGES") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<!--") {
                self.skip_comment();
                continue;
            }
            
            if self.peek_str("<AR-PACKAGE") {
                let sub_pkg = self.parse_ar_package()?;
                pkg.add_sub_package(sub_pkg);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        Ok(())
    }

    fn parse_elements(&mut self, pkg: &mut ArPackage) {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</ELEMENTS") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<!--") {
                self.skip_comment();
                continue;
            }
            
            if self.peek_str("<I-SIGNAL>") || self.peek_str("<I-SIGNAL ") {
                let signal = self.parse_i_signal();
                pkg.i_signals.push(signal);
            } else if self.peek_str("<I-SIGNAL-I-PDU>") || self.peek_str("<I-SIGNAL-I-PDU ") {
                let pdu = self.parse_i_signal_i_pdu();
                pkg.i_signal_i_pdus.push(pdu);
            } else if self.peek_str("<CAN-FRAME>") || self.peek_str("<CAN-FRAME ") {
                let frame = self.parse_can_frame();
                pkg.can_frames.push(frame);
            } else if self.peek_str("<CAN-CLUSTER>") || self.peek_str("<CAN-CLUSTER ") {
                let cluster = self.parse_can_cluster();
                pkg.can_clusters.push(cluster);
            } else if self.peek_str("<COMPU-METHOD>") || self.peek_str("<COMPU-METHOD ") {
                let method = self.parse_compu_method();
                pkg.compu_methods.push(method);
            } else if self.peek_str("<UNIT>") || self.peek_str("<UNIT ") {
                let unit = self.parse_unit();
                pkg.units.push(unit);
            } else if self.peek_str("<SW-BASE-TYPE>") || self.peek_str("<SW-BASE-TYPE ") {
                let base_type = self.parse_sw_base_type();
                pkg.base_types.push(base_type);
            } else if self.peek_str("<ECU-INSTANCE>") || self.peek_str("<ECU-INSTANCE ") {
                let ecu = self.parse_ecu_instance();
                pkg.ecu_instances.push(ecu);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
    }
    
    fn parse_i_signal(&mut self) -> ISignal {
        let mut signal = ISignal::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</I-SIGNAL") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                signal.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<I-SIGNAL-LENGTH") || self.peek_str("<LENGTH") {
                let len_str = self.parse_simple_element_any(&["I-SIGNAL-LENGTH", "LENGTH"]);
                signal.i_signal_length = len_str.parse().unwrap_or(0);
            } else if self.peek_str("<INIT-VALUE") {
                let init_str = self.parse_simple_element("INIT-VALUE");
                signal.init_value = init_str.parse().ok();
            } else if self.peek_str("<SYSTEM-SIGNAL-REF") {
                signal.system_signal_ref = self.parse_ref_element("SYSTEM-SIGNAL-REF");
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        signal
    }

    fn parse_i_signal_i_pdu(&mut self) -> ISignalIPdu {
        let mut pdu = ISignalIPdu::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</I-SIGNAL-I-PDU") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                pdu.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<LENGTH") {
                let len_str = self.parse_simple_element("LENGTH");
                pdu.length = len_str.parse().unwrap_or(0);
            } else if self.peek_str("<I-SIGNAL-TO-PDU-MAPPINGS") {
                self.parse_signal_to_pdu_mappings(&mut pdu);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        pdu
    }
    
    fn parse_signal_to_pdu_mappings(&mut self, pdu: &mut ISignalIPdu) {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</I-SIGNAL-TO-PDU-MAPPINGS") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<I-SIGNAL-TO-PDU-MAPPING") || self.peek_str("<I-SIGNAL-TO-I-PDU-MAPPING") {
                let mapping = self.parse_signal_to_pdu_mapping();
                pdu.add_signal_mapping(mapping);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
    }
    
    fn parse_signal_to_pdu_mapping(&mut self) -> ISignalToPduMapping {
        let mut mapping = ISignalToPduMapping::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</I-SIGNAL-TO-PDU-MAPPING") || self.peek_str("</I-SIGNAL-TO-I-PDU-MAPPING") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                mapping.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<I-SIGNAL-REF") {
                mapping.i_signal_ref = self.parse_ref_element("I-SIGNAL-REF");
            } else if self.peek_str("<PACKING-BYTE-ORDER") {
                let order = self.parse_simple_element("PACKING-BYTE-ORDER");
                mapping.packing_byte_order = ArxmlByteOrder::parse(&order);
            } else if self.peek_str("<START-POSITION") {
                let pos_str = self.parse_simple_element("START-POSITION");
                mapping.start_position = pos_str.parse().unwrap_or(0);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        mapping
    }

    fn parse_can_frame(&mut self) -> CanFrame {
        let mut frame = CanFrame::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</CAN-FRAME") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                frame.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<FRAME-LENGTH") {
                let len_str = self.parse_simple_element("FRAME-LENGTH");
                frame.frame_length = len_str.parse().unwrap_or(8);
            } else if self.peek_str("<PDU-TO-FRAME-MAPPINGS") {
                self.parse_pdu_to_frame_mappings(&mut frame);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        frame
    }
    
    fn parse_pdu_to_frame_mappings(&mut self, frame: &mut CanFrame) {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</PDU-TO-FRAME-MAPPINGS") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<PDU-TO-FRAME-MAPPING") {
                let mapping = self.parse_pdu_to_frame_mapping();
                frame.pdu_to_frame_mappings.push(mapping);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
    }
    
    fn parse_pdu_to_frame_mapping(&mut self) -> PduToFrameMapping {
        let mut mapping = PduToFrameMapping::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</PDU-TO-FRAME-MAPPING") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                mapping.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<PDU-REF") {
                mapping.pdu_ref = self.parse_ref_element("PDU-REF");
            } else if self.peek_str("<PACKING-BYTE-ORDER") {
                let order = self.parse_simple_element("PACKING-BYTE-ORDER");
                mapping.packing_byte_order = ArxmlByteOrder::parse(&order);
            } else if self.peek_str("<START-POSITION") {
                let pos_str = self.parse_simple_element("START-POSITION");
                mapping.start_position = pos_str.parse().unwrap_or(0);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        mapping
    }

    fn parse_can_cluster(&mut self) -> CanCluster {
        let mut cluster = CanCluster::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</CAN-CLUSTER") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                cluster.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<BAUDRATE") {
                let rate_str = self.parse_simple_element("BAUDRATE");
                cluster.baudrate = rate_str.parse().unwrap_or(500_000);
            } else if self.peek_str("<CAN-FD-BAUDRATE") {
                let rate_str = self.parse_simple_element("CAN-FD-BAUDRATE");
                cluster.can_fd_baudrate = rate_str.parse().ok();
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        cluster
    }
    
    fn parse_compu_method(&mut self) -> CompuMethod {
        let mut method = CompuMethod::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</COMPU-METHOD") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                method.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<CATEGORY") {
                let cat = self.parse_simple_element("CATEGORY");
                method.category = CompuMethodCategory::parse(&cat);
            } else if self.peek_str("<UNIT-REF") {
                method.unit_ref = self.parse_ref_element("UNIT-REF");
            } else if self.peek_str("<COMPU-INTERNAL-TO-PHYS") {
                self.parse_compu_internal_to_phys(&mut method);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        method
    }
    
    fn parse_compu_internal_to_phys(&mut self, method: &mut CompuMethod) {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</COMPU-INTERNAL-TO-PHYS") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<COMPU-SCALES") {
                self.parse_compu_scales(method);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
    }
    
    fn parse_compu_scales(&mut self, method: &mut CompuMethod) {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</COMPU-SCALES") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<COMPU-SCALE") {
                let scale = self.parse_compu_scale();
                method.compu_scales.push(scale);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
    }

    fn parse_compu_scale(&mut self) -> CompuScale {
        let mut scale = CompuScale::default();
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</COMPU-SCALE") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<LOWER-LIMIT") {
                let val = self.parse_simple_element("LOWER-LIMIT");
                scale.lower_limit = val.parse().unwrap_or(0.0);
            } else if self.peek_str("<UPPER-LIMIT") {
                let val = self.parse_simple_element("UPPER-LIMIT");
                scale.upper_limit = val.parse().unwrap_or(0.0);
            } else if self.peek_str("<COMPU-RATIONAL-COEFFS") {
                self.parse_compu_rational_coeffs(&mut scale);
            } else if self.peek_str("<VT") {
                scale.vt = self.parse_simple_element("VT");
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        scale
    }
    
    fn parse_compu_rational_coeffs(&mut self, scale: &mut CompuScale) {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</COMPU-RATIONAL-COEFFS") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<COMPU-NUMERATOR") {
                self.parse_compu_coeffs(&mut scale.numerator);
            } else if self.peek_str("<COMPU-DENOMINATOR") {
                self.parse_compu_coeffs(&mut scale.denominator);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
    }
    
    fn parse_compu_coeffs(&mut self, coeffs: &mut Vec<f64>) {
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</COMPU-NUMERATOR") || self.peek_str("</COMPU-DENOMINATOR") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<V") {
                let val = self.parse_simple_element("V");
                if let Ok(v) = val.parse() {
                    coeffs.push(v);
                }
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
    }

    fn parse_unit(&mut self) -> Unit {
        let mut unit = Unit::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</UNIT") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                unit.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<DISPLAY-NAME") {
                unit.display_name = self.parse_simple_element("DISPLAY-NAME");
            } else if self.peek_str("<FACTOR-SI-TO-UNIT") {
                let val = self.parse_simple_element("FACTOR-SI-TO-UNIT");
                unit.factor_si_to_unit = val.parse().unwrap_or(1.0);
            } else if self.peek_str("<OFFSET-SI-TO-UNIT") {
                let val = self.parse_simple_element("OFFSET-SI-TO-UNIT");
                unit.offset_si_to_unit = val.parse().unwrap_or(0.0);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        unit
    }

    fn parse_sw_base_type(&mut self) -> SwBaseType {
        let mut base_type = SwBaseType::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</SW-BASE-TYPE") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                base_type.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<BASE-TYPE-SIZE") {
                let val = self.parse_simple_element("BASE-TYPE-SIZE");
                base_type.base_type_size = val.parse().unwrap_or(0);
            } else if self.peek_str("<BASE-TYPE-ENCODING") {
                let enc = self.parse_simple_element("BASE-TYPE-ENCODING");
                base_type.base_type_encoding = BaseTypeEncoding::parse(&enc);
            } else if self.peek_str("<NATIVE-DECLARATION") {
                base_type.native_declaration = self.parse_simple_element("NATIVE-DECLARATION");
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        base_type
    }

    fn parse_ecu_instance(&mut self) -> EcuInstance {
        let mut ecu = EcuInstance::new("");
        
        self.skip_to_char('>');
        self.advance();
        
        loop {
            self.skip_whitespace();
            
            if self.peek_str("</ECU-INSTANCE") {
                self.skip_element_end();
                break;
            }
            
            if self.peek_str("<SHORT-NAME") {
                ecu.short_name = self.parse_simple_element("SHORT-NAME");
            } else if self.peek_str("<COM-I-PDU-GROUP-REF") {
                let group_ref = self.parse_ref_element("COM-I-PDU-GROUP-REF");
                ecu.com_i_pdu_groups.push(group_ref);
            } else if self.current_char() == Some('<') {
                self.skip_element();
            } else {
                self.advance();
            }
        }
        
        ecu
    }

    // Helper methods
    
    fn parse_simple_element(&mut self, tag: &str) -> String {
        // Skip opening tag
        self.skip_to_char('>');
        self.advance();
        
        // Read text content
        let content = self.read_text_content();
        
        // Skip closing tag
        let close_tag = format!("</{}", tag);
        if self.peek_str(&close_tag) {
            self.skip_element_end();
        }
        
        content
    }
    
    fn parse_simple_element_any(&mut self, tags: &[&str]) -> String {
        // Skip opening tag
        self.skip_to_char('>');
        self.advance();
        
        // Read text content
        let content = self.read_text_content();
        
        // Skip closing tag (try each possible tag)
        for tag in tags {
            let close_tag = format!("</{}", tag);
            if self.peek_str(&close_tag) {
                self.skip_element_end();
                break;
            }
        }
        
        content
    }
    
    fn parse_ref_element(&mut self, tag: &str) -> String {
        // Skip to end of opening tag
        self.skip_to_char('>');
        self.advance();
        
        // Read reference path
        let content = self.read_text_content();
        
        // Skip closing tag
        let close_tag = format!("</{}", tag);
        if self.peek_str(&close_tag) {
            self.skip_element_end();
        }
        
        content
    }
    
    fn skip_element(&mut self) {
        // Skip opening tag name
        if self.current_char() == Some('<') {
            self.advance();
        }
        
        let tag_name = self.read_tag_name();
        if tag_name.is_empty() {
            return;
        }
        
        // Check for self-closing tag
        self.skip_to_char('>');
        let prev_pos = self.pos.saturating_sub(1);
        if prev_pos < self.input.len() && self.input[prev_pos..].starts_with('/') {
            self.advance(); // skip '>'
            return;
        }
        self.advance(); // skip '>'
        
        // Find matching closing tag
        let close_tag = format!("</{}", tag_name);
        let mut depth = 1;
        
        while !self.is_eof() && depth > 0 {
            if self.peek_str(&close_tag) {
                depth -= 1;
                if depth == 0 {
                    self.skip_element_end();
                    break;
                }
            } else if self.peek_str(&format!("<{}", tag_name)) {
                // Check if it's not a self-closing tag
                let saved_pos = self.pos;
                self.skip_to_char('>');
                let is_self_closing = self.pos > 0 && self.input[self.pos - 1..].starts_with('/');
                self.pos = saved_pos;
                if !is_self_closing {
                    depth += 1;
                }
            }
            self.advance();
        }
    }
    
    fn skip_element_end(&mut self) {
        // Skip </TAG>
        while !self.is_eof() && self.current_char() != Some('>') {
            self.advance();
        }
        if self.current_char() == Some('>') {
            self.advance();
        }
    }
    
    fn skip_to_char(&mut self, target: char) {
        while !self.is_eof() && self.current_char() != Some(target) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autosar_version_from_schema_url() {
        let v = AutosarVersion::from_schema_url("http://autosar.org/schema/r4.0").unwrap();
        assert_eq!(v.major, 4);
        assert_eq!(v.minor, 0);
        
        let v = AutosarVersion::from_schema_url("http://autosar.org/schema/r4.2.2").unwrap();
        assert_eq!(v.major, 4);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 2);
    }

    #[test]
    fn test_byte_order_parse() {
        assert_eq!(ArxmlByteOrder::parse("MOST-SIGNIFICANT-BYTE-FIRST"), ArxmlByteOrder::MostSignificantByteFirst);
        assert_eq!(ArxmlByteOrder::parse("MOST-SIGNIFICANT-BYTE-LAST"), ArxmlByteOrder::MostSignificantByteLast);
        assert_eq!(ArxmlByteOrder::parse("OPAQUE"), ArxmlByteOrder::Opaque);
    }

    #[test]
    fn test_base_type_encoding_parse() {
        assert_eq!(BaseTypeEncoding::parse("SIGNED"), BaseTypeEncoding::Signed);
        assert_eq!(BaseTypeEncoding::parse("UNSIGNED"), BaseTypeEncoding::Unsigned);
        assert_eq!(BaseTypeEncoding::parse("2C"), BaseTypeEncoding::Unsigned);
        assert_eq!(BaseTypeEncoding::parse("IEEE754"), BaseTypeEncoding::Ieee754);
        assert_eq!(BaseTypeEncoding::parse("BOOLEAN"), BaseTypeEncoding::Boolean);
    }

    #[test]
    fn test_compu_method_category_parse() {
        assert_eq!(CompuMethodCategory::parse("IDENTICAL"), CompuMethodCategory::Identical);
        assert_eq!(CompuMethodCategory::parse("LINEAR"), CompuMethodCategory::Linear);
        assert_eq!(CompuMethodCategory::parse("TEXTTABLE"), CompuMethodCategory::TextTable);
        assert_eq!(CompuMethodCategory::parse("RAT-FUNC"), CompuMethodCategory::RatFunc);
    }

    #[test]
    fn test_parse_minimal_arxml() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>TestPackage</SHORT-NAME>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.version.major, 4);
        assert_eq!(db.packages.len(), 1);
        assert_eq!(db.packages[0].short_name, "TestPackage");
    }

    #[test]
    fn test_parse_nested_packages() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>RootPackage</SHORT-NAME>
      <AR-PACKAGES>
        <AR-PACKAGE>
          <SHORT-NAME>SubPackage</SHORT-NAME>
        </AR-PACKAGE>
      </AR-PACKAGES>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages.len(), 1);
        assert_eq!(db.packages[0].short_name, "RootPackage");
        assert_eq!(db.packages[0].sub_packages.len(), 1);
        assert_eq!(db.packages[0].sub_packages[0].short_name, "SubPackage");
    }

    #[test]
    fn test_parse_i_signal() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>Signals</SHORT-NAME>
      <ELEMENTS>
        <I-SIGNAL>
          <SHORT-NAME>EngineSpeed</SHORT-NAME>
          <I-SIGNAL-LENGTH>16</I-SIGNAL-LENGTH>
          <INIT-VALUE>0</INIT-VALUE>
        </I-SIGNAL>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].i_signals.len(), 1);
        let signal = &db.packages[0].i_signals[0];
        assert_eq!(signal.short_name, "EngineSpeed");
        assert_eq!(signal.i_signal_length, 16);
        assert_eq!(signal.init_value, Some(0.0));
    }

    #[test]
    fn test_parse_i_signal_i_pdu() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>PDUs</SHORT-NAME>
      <ELEMENTS>
        <I-SIGNAL-I-PDU>
          <SHORT-NAME>EngineData_PDU</SHORT-NAME>
          <LENGTH>8</LENGTH>
          <I-SIGNAL-TO-PDU-MAPPINGS>
            <I-SIGNAL-TO-PDU-MAPPING>
              <SHORT-NAME>EngineSpeed_Mapping</SHORT-NAME>
              <I-SIGNAL-REF>/Signals/EngineSpeed</I-SIGNAL-REF>
              <PACKING-BYTE-ORDER>MOST-SIGNIFICANT-BYTE-LAST</PACKING-BYTE-ORDER>
              <START-POSITION>0</START-POSITION>
            </I-SIGNAL-TO-PDU-MAPPING>
          </I-SIGNAL-TO-PDU-MAPPINGS>
        </I-SIGNAL-I-PDU>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].i_signal_i_pdus.len(), 1);
        let pdu = &db.packages[0].i_signal_i_pdus[0];
        assert_eq!(pdu.short_name, "EngineData_PDU");
        assert_eq!(pdu.length, 8);
        assert_eq!(pdu.i_signal_to_pdu_mappings.len(), 1);
        let mapping = &pdu.i_signal_to_pdu_mappings[0];
        assert_eq!(mapping.short_name, "EngineSpeed_Mapping");
        assert_eq!(mapping.i_signal_ref, "/Signals/EngineSpeed");
        assert_eq!(mapping.packing_byte_order, ArxmlByteOrder::MostSignificantByteLast);
        assert_eq!(mapping.start_position, 0);
    }

    #[test]
    fn test_parse_can_frame() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>Frames</SHORT-NAME>
      <ELEMENTS>
        <CAN-FRAME>
          <SHORT-NAME>EngineData_Frame</SHORT-NAME>
          <FRAME-LENGTH>8</FRAME-LENGTH>
          <PDU-TO-FRAME-MAPPINGS>
            <PDU-TO-FRAME-MAPPING>
              <SHORT-NAME>EngineData_Mapping</SHORT-NAME>
              <PDU-REF>/PDUs/EngineData_PDU</PDU-REF>
              <START-POSITION>0</START-POSITION>
            </PDU-TO-FRAME-MAPPING>
          </PDU-TO-FRAME-MAPPINGS>
        </CAN-FRAME>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].can_frames.len(), 1);
        let frame = &db.packages[0].can_frames[0];
        assert_eq!(frame.short_name, "EngineData_Frame");
        assert_eq!(frame.frame_length, 8);
        assert_eq!(frame.pdu_to_frame_mappings.len(), 1);
    }

    #[test]
    fn test_parse_can_cluster() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>Clusters</SHORT-NAME>
      <ELEMENTS>
        <CAN-CLUSTER>
          <SHORT-NAME>CAN_Network</SHORT-NAME>
          <BAUDRATE>500000</BAUDRATE>
          <CAN-FD-BAUDRATE>2000000</CAN-FD-BAUDRATE>
        </CAN-CLUSTER>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].can_clusters.len(), 1);
        let cluster = &db.packages[0].can_clusters[0];
        assert_eq!(cluster.short_name, "CAN_Network");
        assert_eq!(cluster.baudrate, 500000);
        assert_eq!(cluster.can_fd_baudrate, Some(2000000));
    }

    #[test]
    fn test_parse_compu_method_linear() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>CompuMethods</SHORT-NAME>
      <ELEMENTS>
        <COMPU-METHOD>
          <SHORT-NAME>CM_EngineSpeed</SHORT-NAME>
          <CATEGORY>LINEAR</CATEGORY>
          <UNIT-REF>/Units/rpm</UNIT-REF>
          <COMPU-INTERNAL-TO-PHYS>
            <COMPU-SCALES>
              <COMPU-SCALE>
                <LOWER-LIMIT>0</LOWER-LIMIT>
                <UPPER-LIMIT>8000</UPPER-LIMIT>
                <COMPU-RATIONAL-COEFFS>
                  <COMPU-NUMERATOR>
                    <V>0</V>
                    <V>0.25</V>
                  </COMPU-NUMERATOR>
                  <COMPU-DENOMINATOR>
                    <V>1</V>
                  </COMPU-DENOMINATOR>
                </COMPU-RATIONAL-COEFFS>
              </COMPU-SCALE>
            </COMPU-SCALES>
          </COMPU-INTERNAL-TO-PHYS>
        </COMPU-METHOD>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].compu_methods.len(), 1);
        let method = &db.packages[0].compu_methods[0];
        assert_eq!(method.short_name, "CM_EngineSpeed");
        assert_eq!(method.category, CompuMethodCategory::Linear);
        assert_eq!(method.unit_ref, "/Units/rpm");
        assert_eq!(method.compu_scales.len(), 1);
        let scale = &method.compu_scales[0];
        assert_eq!(scale.lower_limit, 0.0);
        assert_eq!(scale.upper_limit, 8000.0);
        assert_eq!(scale.numerator, vec![0.0, 0.25]);
        assert_eq!(scale.denominator, vec![1.0]);
    }

    #[test]
    fn test_parse_unit() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>Units</SHORT-NAME>
      <ELEMENTS>
        <UNIT>
          <SHORT-NAME>rpm</SHORT-NAME>
          <DISPLAY-NAME>RPM</DISPLAY-NAME>
          <FACTOR-SI-TO-UNIT>60</FACTOR-SI-TO-UNIT>
        </UNIT>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].units.len(), 1);
        let unit = &db.packages[0].units[0];
        assert_eq!(unit.short_name, "rpm");
        assert_eq!(unit.display_name, "RPM");
        assert_eq!(unit.factor_si_to_unit, 60.0);
    }

    #[test]
    fn test_parse_sw_base_type() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>BaseTypes</SHORT-NAME>
      <ELEMENTS>
        <SW-BASE-TYPE>
          <SHORT-NAME>uint16</SHORT-NAME>
          <BASE-TYPE-SIZE>16</BASE-TYPE-SIZE>
          <BASE-TYPE-ENCODING>UNSIGNED</BASE-TYPE-ENCODING>
          <NATIVE-DECLARATION>uint16_t</NATIVE-DECLARATION>
        </SW-BASE-TYPE>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].base_types.len(), 1);
        let bt = &db.packages[0].base_types[0];
        assert_eq!(bt.short_name, "uint16");
        assert_eq!(bt.base_type_size, 16);
        assert_eq!(bt.base_type_encoding, BaseTypeEncoding::Unsigned);
        assert_eq!(bt.native_declaration, "uint16_t");
    }

    #[test]
    fn test_parse_ecu_instance() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>ECUs</SHORT-NAME>
      <ELEMENTS>
        <ECU-INSTANCE>
          <SHORT-NAME>ECU_Engine</SHORT-NAME>
        </ECU-INSTANCE>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages[0].ecu_instances.len(), 1);
        let ecu = &db.packages[0].ecu_instances[0];
        assert_eq!(ecu.short_name, "ECU_Engine");
    }

    #[test]
    fn test_find_package_by_path() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>Root</SHORT-NAME>
      <AR-PACKAGES>
        <AR-PACKAGE>
          <SHORT-NAME>Sub1</SHORT-NAME>
          <AR-PACKAGES>
            <AR-PACKAGE>
              <SHORT-NAME>Sub2</SHORT-NAME>
            </AR-PACKAGE>
          </AR-PACKAGES>
        </AR-PACKAGE>
      </AR-PACKAGES>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        
        let pkg = db.find_package_by_path("/Root").unwrap();
        assert_eq!(pkg.short_name, "Root");
        
        let pkg = db.find_package_by_path("/Root/Sub1").unwrap();
        assert_eq!(pkg.short_name, "Sub1");
        
        let pkg = db.find_package_by_path("/Root/Sub1/Sub2").unwrap();
        assert_eq!(pkg.short_name, "Sub2");
        
        assert!(db.find_package_by_path("/NonExistent").is_none());
    }

    #[test]
    fn test_all_signals() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>Pkg1</SHORT-NAME>
      <ELEMENTS>
        <I-SIGNAL>
          <SHORT-NAME>Signal1</SHORT-NAME>
          <I-SIGNAL-LENGTH>8</I-SIGNAL-LENGTH>
        </I-SIGNAL>
      </ELEMENTS>
      <AR-PACKAGES>
        <AR-PACKAGE>
          <SHORT-NAME>SubPkg</SHORT-NAME>
          <ELEMENTS>
            <I-SIGNAL>
              <SHORT-NAME>Signal2</SHORT-NAME>
              <I-SIGNAL-LENGTH>16</I-SIGNAL-LENGTH>
            </I-SIGNAL>
          </ELEMENTS>
        </AR-PACKAGE>
      </AR-PACKAGES>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        let signals = db.all_signals();
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].short_name, "Signal1");
        assert_eq!(signals[1].short_name, "Signal2");
    }

    #[test]
    fn test_compu_scale_linear() {
        let scale = CompuScale::linear(0.5, 10.0);
        assert_eq!(scale.numerator, vec![10.0, 0.5]);
        assert_eq!(scale.denominator, vec![1.0]);
    }

    #[test]
    fn test_compu_scale_text_entry() {
        let scale = CompuScale::text_entry(1.0, "ON");
        assert_eq!(scale.lower_limit, 1.0);
        assert_eq!(scale.upper_limit, 1.0);
        assert_eq!(scale.vt, "ON");
    }

    #[test]
    fn test_compu_method_constructors() {
        let identical = CompuMethod::identical("CM_Identical");
        assert_eq!(identical.short_name, "CM_Identical");
        assert_eq!(identical.category, CompuMethodCategory::Identical);
        
        let linear = CompuMethod::linear("CM_Linear", 2.0, 5.0);
        assert_eq!(linear.short_name, "CM_Linear");
        assert_eq!(linear.category, CompuMethodCategory::Linear);
        assert_eq!(linear.compu_scales.len(), 1);
    }

    #[test]
    fn test_parse_with_comments() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- This is a comment -->
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <!-- Another comment -->
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>TestPackage</SHORT-NAME>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"#;
        let db = ArxmlParser::parse(arxml).unwrap();
        assert_eq!(db.packages.len(), 1);
        assert_eq!(db.packages[0].short_name, "TestPackage");
    }

    #[test]
    fn test_missing_autosar_element() {
        let arxml = r#"<?xml version="1.0" encoding="UTF-8"?>
<INVALID>
</INVALID>
"#;
        let result = ArxmlParser::parse(arxml);
        assert!(result.is_err());
        match result {
            Err(ArxmlParseError::MissingElement(elem)) => assert_eq!(elem, "AUTOSAR"),
            _ => panic!("Expected MissingElement error"),
        }
    }
}
