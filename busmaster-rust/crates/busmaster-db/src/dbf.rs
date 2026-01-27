//! BUSMASTER Database File (DBF) Parser
//!
//! This module provides a parser for the BUSMASTER native database format (.dbf).
//! DBF files are text-based and contain CAN message and signal definitions.
//!
//! # Format Overview
//!
//! DBF files use a tag-based format with sections enclosed in brackets:
//! - `[DATABASE_VERSION]` - Version of the database format
//! - `[PROTOCOL]` - Protocol type (CAN, J1939, etc.)
//! - `[BUSMASTER_VERSION]` - BUSMASTER version that created the file
//! - `[NUMBER_OF_MESSAGES]` - Count of messages in the database
//! - `[START_MSG]` / `[END_MSG]` - Message definition block
//! - `[START_SIGNALS]` - Signal definition within a message
//! - `[NODE]` - Network node definitions
//! - `[VALUE_DESCRIPTION]` - Signal value descriptions
//! - `[START_VALUE_TABLE]` / `[END_VALUE_TABLE]` - Value table definitions
//! - `[START_PARAM]` / `[END_PARAM]` - Parameter definitions
//! - `[START_DESC]` / `[END_DESC]` - Comment/description sections
//!
//! # Example
//!
//! ```
//! use busmaster_db::dbf::DbfParser;
//!
//! let dbf_content = r#"
//! //******************************BUSMASTER Messages and signals Database ******************************//
//!
//! [DATABASE_VERSION] 1.3
//!
//! [PROTOCOL] CAN
//!
//! [BUSMASTER_VERSION] [2.3.0]
//!
//! [NUMBER_OF_MESSAGES] 1
//!
//! [START_MSG] TestMsg,100,8,1,1,S
//! [START_SIGNALS] TestSignal,8,1,0,U,255,0,1,0.0,1.0,unit,
//! [END_MSG]
//! "#;
//!
//! let database = DbfParser::parse(dbf_content).unwrap();
//! assert_eq!(database.messages.len(), 1);
//! ```

use std::fmt;
use std::fmt::Write as _;

/// Error type for DBF parsing
#[derive(Debug, Clone, PartialEq)]
pub enum DbfParseError {
    /// Invalid file format
    InvalidFormat(String),
    /// Missing required field
    MissingField(String),
    /// Invalid value
    InvalidValue(String),
    /// Unexpected end of file
    UnexpectedEof,
    /// Invalid message definition
    InvalidMessage(String),
    /// Invalid signal definition
    InvalidSignal(String),
}

impl fmt::Display for DbfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(msg) => write!(f, "Invalid DBF format: {}", msg),
            Self::MissingField(field) => write!(f, "Missing required field: {}", field),
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Self::UnexpectedEof => write!(f, "Unexpected end of file"),
            Self::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            Self::InvalidSignal(msg) => write!(f, "Invalid signal: {}", msg),
        }
    }
}

impl std::error::Error for DbfParseError {}

/// Protocol type in DBF file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DbfProtocol {
    /// CAN protocol
    #[default]
    Can,
    /// J1939 protocol
    J1939,
    /// LIN protocol
    Lin,
}

impl DbfProtocol {
    /// Parse protocol from string
    pub fn parse_from(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "J1939" => Self::J1939,
            "LIN" => Self::Lin,
            _ => Self::Can,
        }
    }
}

/// Frame format (Standard or Extended)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DbfFrameFormat {
    /// Standard 11-bit identifier
    #[default]
    Standard,
    /// Extended 29-bit identifier
    Extended,
}

impl DbfFrameFormat {
    /// Parse frame format from character
    pub fn from_char(c: char) -> Self {
        match c {
            'X' | 'x' => Self::Extended,
            _ => Self::Standard,
        }
    }
    
    /// Convert to character representation
    pub fn to_char(self) -> char {
        match self {
            Self::Standard => 'S',
            Self::Extended => 'X',
        }
    }
}

/// Signal data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DbfSignalType {
    /// Boolean (1 bit)
    Bool,
    /// Unsigned integer
    #[default]
    Unsigned,
    /// Signed integer
    Signed,
    /// 32-bit float
    Float,
    /// 64-bit double
    Double,
}

impl DbfSignalType {
    /// Parse signal type from character
    pub fn from_char(c: char) -> Self {
        match c {
            'B' | 'b' => Self::Bool,
            'I' | 'i' => Self::Signed,
            'F' | 'f' => Self::Float,
            'D' | 'd' => Self::Double,
            _ => Self::Unsigned,
        }
    }
    
    /// Convert to character representation
    pub fn to_char(self) -> char {
        match self {
            Self::Bool => 'B',
            Self::Unsigned => 'U',
            Self::Signed => 'I',
            Self::Float => 'F',
            Self::Double => 'D',
        }
    }
}

/// Signal byte order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DbfByteOrder {
    /// Intel byte order (little-endian)
    #[default]
    Intel,
    /// Motorola byte order (big-endian)
    Motorola,
}

impl DbfByteOrder {
    /// Parse byte order from character
    pub fn from_char(c: char) -> Self {
        match c {
            '0' => Self::Motorola,
            _ => Self::Intel,
        }
    }
    
    /// Convert to character representation
    pub fn to_char(self) -> char {
        match self {
            Self::Intel => '1',
            Self::Motorola => '0',
        }
    }
}

/// Value descriptor for a signal
#[derive(Debug, Clone, PartialEq)]
pub struct DbfValueDescriptor {
    /// Numeric value
    pub value: i64,
    /// Description string
    pub description: String,
}

impl DbfValueDescriptor {
    /// Create a new value descriptor
    pub fn new(value: i64, description: impl Into<String>) -> Self {
        Self {
            value,
            description: description.into(),
        }
    }
}

/// Signal definition in DBF format
#[derive(Debug, Clone, PartialEq)]
pub struct DbfSignal {
    /// Signal name
    pub name: String,
    /// Signal length in bits
    pub length: u8,
    /// Which byte the signal starts in (1-based)
    pub which_byte: u8,
    /// Start bit within the byte (0-based)
    pub start_bit: u8,
    /// Signal data type
    pub signal_type: DbfSignalType,
    /// Maximum value (raw)
    pub max_value: f64,
    /// Minimum value (raw)
    pub min_value: f64,
    /// Byte order
    pub byte_order: DbfByteOrder,
    /// Offset for physical value calculation
    pub offset: f64,
    /// Scale factor for physical value calculation
    pub scale_factor: f64,
    /// Unit string
    pub unit: String,
    /// Multiplexer indicator (empty, "M", or "m<n>")
    pub multiplex: String,
    /// Receiving nodes
    pub receivers: Vec<String>,
    /// Value descriptors
    pub value_descriptors: Vec<DbfValueDescriptor>,
    /// Comment
    pub comment: String,
}

impl Default for DbfSignal {
    fn default() -> Self {
        Self {
            name: String::new(),
            length: 1,
            which_byte: 1,
            start_bit: 0,
            signal_type: DbfSignalType::default(),
            max_value: 0.0,
            min_value: 0.0,
            byte_order: DbfByteOrder::default(),
            offset: 0.0,
            scale_factor: 1.0,
            unit: String::new(),
            multiplex: String::new(),
            receivers: Vec::new(),
            value_descriptors: Vec::new(),
            comment: String::new(),
        }
    }
}

impl DbfSignal {
    /// Create a new signal with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    
    /// Calculate the absolute start bit position
    pub fn absolute_start_bit(&self) -> u16 {
        u16::from(self.start_bit) + (u16::from(self.which_byte) - 1) * 8
    }
    
    /// Convert raw value to physical value
    pub fn raw_to_physical(&self, raw: f64) -> f64 {
        raw * self.scale_factor + self.offset
    }
    
    /// Convert physical value to raw value
    pub fn physical_to_raw(&self, physical: f64) -> f64 {
        (physical - self.offset) / self.scale_factor
    }
    
    /// Get the physical minimum value
    pub fn physical_min(&self) -> f64 {
        self.raw_to_physical(self.min_value)
    }
    
    /// Get the physical maximum value
    pub fn physical_max(&self) -> f64 {
        self.raw_to_physical(self.max_value)
    }
}

/// Message definition in DBF format
#[derive(Debug, Clone, PartialEq)]
pub struct DbfMessage {
    /// Message name
    pub name: String,
    /// Message ID
    pub id: u32,
    /// Message length in bytes
    pub length: u8,
    /// Number of signals
    pub signal_count: u16,
    /// Data format (unused, typically 1)
    pub data_format: u8,
    /// Frame format (Standard or Extended)
    pub frame_format: DbfFrameFormat,
    /// Transmitting node
    pub transmitter: String,
    /// Signals in this message
    pub signals: Vec<DbfSignal>,
    /// Comment
    pub comment: String,
}

impl Default for DbfMessage {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            length: 8,
            signal_count: 0,
            data_format: 1,
            frame_format: DbfFrameFormat::default(),
            transmitter: String::new(),
            signals: Vec::new(),
            comment: String::new(),
        }
    }
}

impl DbfMessage {
    /// Create a new message with the given name and ID
    pub fn new(name: impl Into<String>, id: u32) -> Self {
        Self {
            name: name.into(),
            id,
            ..Default::default()
        }
    }
    
    /// Add a signal to this message
    #[allow(clippy::cast_possible_truncation)]
    pub fn add_signal(&mut self, signal: DbfSignal) {
        self.signals.push(signal);
        self.signal_count = self.signals.len() as u16;
    }
    
    /// Find a signal by name
    pub fn find_signal(&self, name: &str) -> Option<&DbfSignal> {
        self.signals.iter().find(|s| s.name == name)
    }
    
    /// Find a signal by name (mutable)
    pub fn find_signal_mut(&mut self, name: &str) -> Option<&mut DbfSignal> {
        self.signals.iter_mut().find(|s| s.name == name)
    }
}

/// Value table definition
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DbfValueTable {
    /// Table name
    pub name: String,
    /// Value descriptors
    pub values: Vec<DbfValueDescriptor>,
}

impl DbfValueTable {
    /// Create a new value table
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: Vec::new(),
        }
    }
    
    /// Add a value descriptor
    pub fn add_value(&mut self, value: i64, description: impl Into<String>) {
        self.values.push(DbfValueDescriptor::new(value, description));
    }
}

/// Parameter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DbfParamType {
    /// Integer parameter
    #[default]
    Int,
    /// Hex parameter
    Hex,
    /// Float parameter
    Float,
    /// String parameter
    String,
    /// Enum parameter
    Enum,
}

/// Parameter definition
#[derive(Debug, Clone, PartialEq)]
pub struct DbfParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: DbfParamType,
    /// Minimum value (for numeric types)
    pub min_value: f64,
    /// Maximum value (for numeric types)
    pub max_value: f64,
    /// Default value (as string)
    pub default_value: String,
    /// Enum values (for enum type)
    pub enum_values: Vec<String>,
}

impl Default for DbfParameter {
    fn default() -> Self {
        Self {
            name: String::new(),
            param_type: DbfParamType::default(),
            min_value: 0.0,
            max_value: 0.0,
            default_value: String::new(),
            enum_values: Vec::new(),
        }
    }
}

/// Comment/description entry
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DbfComment {
    /// Element name (node, message, or signal name)
    pub element_name: String,
    /// Message ID (for message and signal comments)
    pub message_id: Option<u32>,
    /// Comment text
    pub comment: String,
}

/// DBF Database - root container for parsed DBF data
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DbfDatabase {
    /// Database version
    pub version: String,
    /// Protocol type
    pub protocol: DbfProtocol,
    /// BUSMASTER version
    pub busmaster_version: String,
    /// Network nodes
    pub nodes: Vec<String>,
    /// Messages
    pub messages: Vec<DbfMessage>,
    /// Value tables
    pub value_tables: Vec<DbfValueTable>,
    /// Network parameters
    pub network_params: Vec<DbfParameter>,
    /// Node parameters
    pub node_params: Vec<DbfParameter>,
    /// Message parameters
    pub message_params: Vec<DbfParameter>,
    /// Signal parameters
    pub signal_params: Vec<DbfParameter>,
    /// Network comments
    pub network_comments: Vec<DbfComment>,
    /// Node comments
    pub node_comments: Vec<DbfComment>,
    /// Message comments
    pub message_comments: Vec<DbfComment>,
    /// Signal comments
    pub signal_comments: Vec<DbfComment>,
    /// Unprocessed lines (for round-trip preservation)
    pub not_processed: Vec<String>,
}

impl DbfDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a node to the database
    pub fn add_node(&mut self, name: impl Into<String>) {
        self.nodes.push(name.into());
    }
    
    /// Add a message to the database
    pub fn add_message(&mut self, message: DbfMessage) {
        self.messages.push(message);
    }
    
    /// Find a message by ID
    pub fn find_message_by_id(&self, id: u32) -> Option<&DbfMessage> {
        self.messages.iter().find(|m| m.id == id)
    }
    
    /// Find a message by name
    pub fn find_message_by_name(&self, name: &str) -> Option<&DbfMessage> {
        self.messages.iter().find(|m| m.name == name)
    }
    
    /// Find a message by ID (mutable)
    pub fn find_message_by_id_mut(&mut self, id: u32) -> Option<&mut DbfMessage> {
        self.messages.iter_mut().find(|m| m.id == id)
    }
    
    /// Find a signal by message ID and signal name
    pub fn find_signal(&self, message_id: u32, signal_name: &str) -> Option<&DbfSignal> {
        self.find_message_by_id(message_id)
            .and_then(|m| m.find_signal(signal_name))
    }
    
    /// Get total signal count across all messages
    pub fn total_signal_count(&self) -> usize {
        self.messages.iter().map(|m| m.signals.len()).sum()
    }
}

/// DBF Parser
pub struct DbfParser<'a> {
    input: &'a str,
    pos: usize,
    line_num: usize,
}

impl<'a> DbfParser<'a> {
    /// Create a new parser for the given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line_num: 1,
        }
    }
    
    /// Parse the input and return a database
    pub fn parse(input: &str) -> Result<DbfDatabase, DbfParseError> {
        let mut parser = DbfParser::new(input);
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
            if c == '\n' {
                self.line_num += 1;
            }
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_to_eol(&mut self) {
        while let Some(c) = self.current_char() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_line(&mut self) -> String {
        let start = self.pos;
        self.skip_to_eol();
        let line = self.input[start..self.pos].trim().to_string();
        if self.current_char() == Some('\n') {
            self.advance();
        }
        line
    }
    
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }
    
    fn read_tag(&mut self) -> Option<String> {
        self.skip_whitespace();
        if self.current_char() != Some('[') {
            return None;
        }
        self.advance();
        
        let start = self.pos;
        while let Some(c) = self.current_char() {
            if c == ']' {
                let tag = self.input[start..self.pos].to_string();
                self.advance();
                return Some(tag);
            }
            self.advance();
        }
        None
    }
    
    fn read_tag_value(&mut self) -> String {
        self.skip_whitespace();
        let start = self.pos;
        self.skip_to_eol();
        let value = self.input[start..self.pos].trim().to_string();
        if self.current_char() == Some('\n') {
            self.advance();
        }
        value
    }

    #[allow(clippy::too_many_lines)]
    fn parse_database(&mut self) -> Result<DbfDatabase, DbfParseError> {
        let mut db = DbfDatabase::new();
        let mut current_message: Option<DbfMessage> = None;
        let mut current_signal: Option<DbfSignal> = None;
        
        while !self.is_eof() {
            self.skip_whitespace();
            
            if self.current_char() == Some('\n') {
                self.advance();
                continue;
            }
            
            if self.starts_with("//") {
                self.skip_to_eol();
                if self.current_char() == Some('\n') {
                    self.advance();
                }
                continue;
            }
            
            if self.current_char() == Some('[') {
                if let Some(tag) = self.read_tag() {
                    self.handle_tag(&tag, &mut db, &mut current_message, &mut current_signal)?;
                }
            } else {
                self.skip_to_eol();
                if self.current_char() == Some('\n') {
                    self.advance();
                }
            }
        }
        
        if let Some(mut msg) = current_message.take() {
            if let Some(sig) = current_signal.take() {
                msg.add_signal(sig);
            }
            db.add_message(msg);
        }
        
        Ok(db)
    }
    
    fn handle_tag(
        &mut self,
        tag: &str,
        db: &mut DbfDatabase,
        current_message: &mut Option<DbfMessage>,
        current_signal: &mut Option<DbfSignal>,
    ) -> Result<(), DbfParseError> {
        match tag {
            "DATABASE_VERSION" => {
                db.version = self.read_tag_value();
            }
            "PROTOCOL" => {
                let proto = self.read_tag_value();
                db.protocol = DbfProtocol::parse_from(&proto);
            }
            "BUSMASTER_VERSION" => {
                let ver = self.read_tag_value();
                db.busmaster_version = ver.trim_matches(|c| c == '[' || c == ']').to_string();
            }
            "NUMBER_OF_MESSAGES" => {
                self.read_tag_value();
            }
            "NODE" => {
                let nodes = self.read_tag_value();
                for node in nodes.split(',') {
                    let node = node.trim();
                    if !node.is_empty() {
                        db.add_node(node);
                    }
                }
            }
            "START_MSG" => {
                if let Some(mut msg) = current_message.take() {
                    if let Some(sig) = current_signal.take() {
                        msg.add_signal(sig);
                    }
                    db.add_message(msg);
                }
                let msg_data = self.read_tag_value();
                *current_message = Some(Self::parse_message(&msg_data)?);
            }
            "END_MSG" => {
                if let Some(mut msg) = current_message.take() {
                    if let Some(sig) = current_signal.take() {
                        msg.add_signal(sig);
                    }
                    db.add_message(msg);
                }
                self.skip_to_eol();
                if self.current_char() == Some('\n') {
                    self.advance();
                }
            }
            "START_SIGNALS" => {
                if let Some(sig) = current_signal.take() {
                    if let Some(ref mut msg) = current_message {
                        msg.add_signal(sig);
                    }
                }
                let sig_data = self.read_tag_value();
                *current_signal = Some(Self::parse_signal(&sig_data)?);
            }
            "VALUE_DESCRIPTION" => {
                let val_data = self.read_tag_value();
                if let Some(ref mut sig) = current_signal {
                    if let Some(vd) = Self::parse_value_descriptor(&val_data) {
                        sig.value_descriptors.push(vd);
                    }
                }
            }
            "START_SIG_LIST" => self.skip_section("END_SIG_LIST"),
            "START_VALUE_TABLE" => self.parse_value_tables(db),
            "START_PARAM" => self.skip_section("END_PARAM"),
            "START_PARAM_VAL" => self.skip_section("END_PARAM_VAL"),
            "START_DESC" => self.parse_descriptions(db),
            "START_NOT_PROCESSED" => self.parse_not_processed(db),
            "START_NOT_SUPPORTED" => self.skip_section("END_NOT_SUPPORTED"),
            _ => {
                self.skip_to_eol();
                if self.current_char() == Some('\n') {
                    self.advance();
                }
            }
        }
        Ok(())
    }

    fn parse_message(data: &str) -> Result<DbfMessage, DbfParseError> {
        let parts: Vec<&str> = data.split(',').collect();
        if parts.len() < 6 {
            return Err(DbfParseError::InvalidMessage(format!(
                "Expected at least 6 fields, got {}: {}",
                parts.len(),
                data
            )));
        }
        
        let name = parts[0].trim().to_string();
        let id: u32 = parts[1].trim().parse().map_err(|_| {
            DbfParseError::InvalidMessage(format!("Invalid message ID: {}", parts[1]))
        })?;
        let length: u8 = parts[2].trim().parse().map_err(|_| {
            DbfParseError::InvalidMessage(format!("Invalid message length: {}", parts[2]))
        })?;
        let signal_count: u16 = parts[3].trim().parse().map_err(|_| {
            DbfParseError::InvalidMessage(format!("Invalid signal count: {}", parts[3]))
        })?;
        let data_format: u8 = parts[4].trim().parse().unwrap_or(1);
        let frame_char = parts[5].trim().chars().next().unwrap_or('S');
        let frame_format = DbfFrameFormat::from_char(frame_char);
        
        let mut final_id = id;
        if (frame_char == 'X' || frame_char == 'x') && id >= 0x8000_0000 {
            final_id = id - 0x8000_0000;
        }
        
        let transmitter = if parts.len() > 6 {
            parts[6].trim().to_string()
        } else {
            String::new()
        };
        
        Ok(DbfMessage {
            name,
            id: final_id,
            length,
            signal_count,
            data_format,
            frame_format,
            transmitter,
            signals: Vec::new(),
            comment: String::new(),
        })
    }

    fn parse_signal(data: &str) -> Result<DbfSignal, DbfParseError> {
        let parts: Vec<&str> = data.split(',').collect();
        if parts.len() < 11 {
            return Err(DbfParseError::InvalidSignal(format!(
                "Expected at least 11 fields, got {}: {}",
                parts.len(),
                data
            )));
        }
        
        let name = parts[0].trim().to_string();
        let length: u8 = parts[1].trim().parse().map_err(|_| {
            DbfParseError::InvalidSignal(format!("Invalid signal length: {}", parts[1]))
        })?;
        let which_byte: u8 = parts[2].trim().parse().map_err(|_| {
            DbfParseError::InvalidSignal(format!("Invalid which_byte: {}", parts[2]))
        })?;
        let start_bit: u8 = parts[3].trim().parse().map_err(|_| {
            DbfParseError::InvalidSignal(format!("Invalid start_bit: {}", parts[3]))
        })?;
        
        let type_char = parts[4].trim().chars().next().unwrap_or('U');
        let signal_type = DbfSignalType::from_char(type_char);
        let max_value: f64 = parts[5].trim().parse().unwrap_or(0.0);
        let min_value: f64 = parts[6].trim().parse().unwrap_or(0.0);
        let order_char = parts[7].trim().chars().next().unwrap_or('1');
        let byte_order = DbfByteOrder::from_char(order_char);
        let offset: f64 = parts[8].trim().parse().unwrap_or(0.0);
        let scale_factor: f64 = parts[9].trim().parse().unwrap_or(1.0);
        let unit = parts[10].trim().to_string();
        
        let multiplex = if parts.len() > 11 && !parts[11].trim().is_empty() {
            parts[11].trim().to_string()
        } else {
            String::new()
        };
        
        let mut receivers = Vec::new();
        if parts.len() > 12 {
            let recv_str = parts[12..].join(",");
            for receiver in recv_str.split(',') {
                let receiver = receiver.trim();
                if !receiver.is_empty() {
                    receivers.push(receiver.to_string());
                }
            }
        }
        
        Ok(DbfSignal {
            name,
            length,
            which_byte,
            start_bit,
            signal_type,
            max_value,
            min_value,
            byte_order,
            offset,
            scale_factor,
            unit,
            multiplex,
            receivers,
            value_descriptors: Vec::new(),
            comment: String::new(),
        })
    }

    fn parse_value_descriptor(data: &str) -> Option<DbfValueDescriptor> {
        let data = data.trim();
        if data.is_empty() {
            return None;
        }
        
        let start = data.find('"')?;
        let end = data[start + 1..].find('"')?;
        let description = data[start + 1..start + 1 + end].to_string();
        let rest = data[start + 1 + end + 1..].trim();
        
        let value_str = rest.strip_prefix(',')?;
        let value: i64 = value_str.trim().parse().ok()?;
        Some(DbfValueDescriptor::new(value, description))
    }
    
    fn skip_section(&mut self, end_tag: &str) {
        let end_marker = format!("[{}]", end_tag);
        while !self.is_eof() {
            let line = self.read_line();
            if line.contains(&end_marker) {
                break;
            }
        }
    }
    
    fn parse_value_tables(&mut self, db: &mut DbfDatabase) {
        while !self.is_eof() {
            self.skip_whitespace();
            if self.current_char() == Some('\n') {
                self.advance();
                continue;
            }
            
            if let Some(tag) = self.read_tag() {
                match tag.as_str() {
                    "END_VALUE_TABLE" => {
                        self.skip_to_eol();
                        if self.current_char() == Some('\n') {
                            self.advance();
                        }
                        break;
                    }
                    "START_TABLE" => {
                        let name = self.read_tag_value();
                        let mut table = DbfValueTable::new(name);
                        
                        while !self.is_eof() {
                            self.skip_whitespace();
                            if self.current_char() == Some('\n') {
                                self.advance();
                                continue;
                            }
                            
                            if self.current_char() == Some('[') {
                                if let Some(inner_tag) = self.read_tag() {
                                    if inner_tag == "END_TABLE" {
                                        self.skip_to_eol();
                                        if self.current_char() == Some('\n') {
                                            self.advance();
                                        }
                                        break;
                                    }
                                }
                            } else {
                                let line = self.read_line();
                                if let Some(vd) = Self::parse_value_descriptor(&line) {
                                    table.values.push(vd);
                                }
                            }
                        }
                        
                        db.value_tables.push(table);
                    }
                    _ => {
                        self.skip_to_eol();
                        if self.current_char() == Some('\n') {
                            self.advance();
                        }
                    }
                }
            } else {
                self.skip_to_eol();
                if self.current_char() == Some('\n') {
                    self.advance();
                }
            }
        }
    }

    fn parse_descriptions(&mut self, db: &mut DbfDatabase) {
        while !self.is_eof() {
            self.skip_whitespace();
            if self.current_char() == Some('\n') {
                self.advance();
                continue;
            }
            
            if let Some(tag) = self.read_tag() {
                match tag.as_str() {
                    "END_DESC" => {
                        self.skip_to_eol();
                        if self.current_char() == Some('\n') {
                            self.advance();
                        }
                        break;
                    }
                    "START_DESC_NET" => self.parse_comment_section(&mut db.network_comments, "END_DESC_NET"),
                    "START_DESC_NODE" => self.parse_comment_section(&mut db.node_comments, "END_DESC_NODE"),
                    "START_DESC_MSG" => self.parse_comment_section(&mut db.message_comments, "END_DESC_MSG"),
                    "START_DESC_SIG" => self.parse_comment_section(&mut db.signal_comments, "END_DESC_SIG"),
                    _ => {
                        self.skip_to_eol();
                        if self.current_char() == Some('\n') {
                            self.advance();
                        }
                    }
                }
            } else {
                self.skip_to_eol();
                if self.current_char() == Some('\n') {
                    self.advance();
                }
            }
        }
    }
    
    fn parse_comment_section(&mut self, comments: &mut Vec<DbfComment>, end_tag: &str) {
        let end_marker = format!("[{}]", end_tag);
        while !self.is_eof() {
            let line = self.read_line();
            if line.contains(&end_marker) {
                break;
            }
            if !line.is_empty() {
                comments.push(DbfComment {
                    comment: line,
                    ..Default::default()
                });
            }
        }
    }
    
    fn parse_not_processed(&mut self, db: &mut DbfDatabase) {
        while !self.is_eof() {
            let line = self.read_line();
            if line.contains("[END_NOT_PROCESSED]") {
                break;
            }
            if !line.is_empty() {
                let decrypted = Self::decrypt_rot13(&line);
                db.not_processed.push(decrypted);
            }
        }
    }
    
    fn decrypt_rot13(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphabetic() {
                    let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                    let offset = (c as u8 - base + 13) % 26;
                    (base + offset) as char
                } else {
                    c
                }
            })
            .collect()
    }
}

/// DBF Generator - writes DBF files
pub struct DbfGenerator;

impl DbfGenerator {
    /// Generate DBF content from a database
    pub fn generate(db: &DbfDatabase) -> String {
        let mut output = String::new();
        
        output.push_str("//******************************BUSMASTER Messages and signals Database ******************************//\n\n");
        
        let version = if db.version.is_empty() { "1.3" } else { &db.version };
        let _ = writeln!(output, "[DATABASE_VERSION] {}\n", version);
        
        let proto = match db.protocol {
            DbfProtocol::Can => "CAN",
            DbfProtocol::J1939 => "J1939",
            DbfProtocol::Lin => "LIN",
        };
        let _ = writeln!(output, "[PROTOCOL] {}\n", proto);
        
        let bm_version = if db.busmaster_version.is_empty() { "3.0.0" } else { &db.busmaster_version };
        let _ = writeln!(output, "[BUSMASTER_VERSION] [{}]\n", bm_version);
        
        let _ = writeln!(output, "[NUMBER_OF_MESSAGES] {}\n", db.messages.len());
        
        if !db.nodes.is_empty() {
            let _ = writeln!(output, "[NODE] {}\n", db.nodes.join(","));
        }
        
        for msg in &db.messages {
            Self::write_message(&mut output, msg);
        }
        
        if !db.value_tables.is_empty() {
            output.push_str("[START_VALUE_TABLE]\n");
            for table in &db.value_tables {
                Self::write_value_table(&mut output, table);
            }
            output.push_str("[END_VALUE_TABLE]\n\n");
        }
        
        output
    }
    
    fn write_message(output: &mut String, msg: &DbfMessage) {
        let frame_char = msg.frame_format.to_char();
        let id = if msg.frame_format == DbfFrameFormat::Extended {
            msg.id + 0x8000_0000
        } else {
            msg.id
        };
        
        let tx = if msg.transmitter.is_empty() {
            String::new()
        } else {
            format!(",{}", msg.transmitter)
        };
        
        let _ = writeln!(
            output,
            "[START_MSG] {},{},{},{},1,{}{}",
            msg.name, id, msg.length, msg.signals.len(), frame_char, tx
        );
        
        for sig in &msg.signals {
            Self::write_signal(output, sig);
        }
        
        output.push_str("[END_MSG]\n\n");
    }
    
    fn write_signal(output: &mut String, sig: &DbfSignal) {
        let type_char = sig.signal_type.to_char();
        let order_char = sig.byte_order.to_char();
        
        let _ = write!(
            output,
            "[START_SIGNALS] {},{},{},{},{},{},{},{},{},{},{},",
            sig.name, sig.length, sig.which_byte, sig.start_bit,
            type_char, sig.max_value, sig.min_value, order_char,
            sig.offset, sig.scale_factor, sig.unit
        );
        
        if !sig.multiplex.is_empty() {
            output.push_str(&sig.multiplex);
        }
        output.push(',');
        
        if !sig.receivers.is_empty() {
            output.push_str(&sig.receivers.join(","));
        }
        output.push('\n');
        
        for vd in &sig.value_descriptors {
            let _ = writeln!(output, "[VALUE_DESCRIPTION] \"{}\",{}", vd.description, vd.value);
        }
    }
    
    fn write_value_table(output: &mut String, table: &DbfValueTable) {
        let _ = writeln!(output, "[START_TABLE] {}", table.name);
        for vd in &table.values {
            let _ = writeln!(output, "\"{}\",{}", vd.description, vd.value);
        }
        output.push_str("[END_TABLE]\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dbf_protocol_parse_from() {
        assert_eq!(DbfProtocol::parse_from("CAN"), DbfProtocol::Can);
        assert_eq!(DbfProtocol::parse_from("J1939"), DbfProtocol::J1939);
        assert_eq!(DbfProtocol::parse_from("LIN"), DbfProtocol::Lin);
        assert_eq!(DbfProtocol::parse_from("can"), DbfProtocol::Can);
        assert_eq!(DbfProtocol::parse_from("unknown"), DbfProtocol::Can);
    }
    
    #[test]
    fn test_dbf_frame_format() {
        assert_eq!(DbfFrameFormat::from_char('S'), DbfFrameFormat::Standard);
        assert_eq!(DbfFrameFormat::from_char('X'), DbfFrameFormat::Extended);
        assert_eq!(DbfFrameFormat::from_char('x'), DbfFrameFormat::Extended);
        assert_eq!(DbfFrameFormat::Standard.to_char(), 'S');
        assert_eq!(DbfFrameFormat::Extended.to_char(), 'X');
    }
    
    #[test]
    fn test_dbf_signal_type() {
        assert_eq!(DbfSignalType::from_char('B'), DbfSignalType::Bool);
        assert_eq!(DbfSignalType::from_char('U'), DbfSignalType::Unsigned);
        assert_eq!(DbfSignalType::from_char('I'), DbfSignalType::Signed);
        assert_eq!(DbfSignalType::from_char('F'), DbfSignalType::Float);
        assert_eq!(DbfSignalType::from_char('D'), DbfSignalType::Double);
        assert_eq!(DbfSignalType::Unsigned.to_char(), 'U');
    }
    
    #[test]
    fn test_dbf_byte_order() {
        assert_eq!(DbfByteOrder::from_char('0'), DbfByteOrder::Motorola);
        assert_eq!(DbfByteOrder::from_char('1'), DbfByteOrder::Intel);
        assert_eq!(DbfByteOrder::Intel.to_char(), '1');
        assert_eq!(DbfByteOrder::Motorola.to_char(), '0');
    }
    
    #[test]
    fn test_dbf_signal_conversion() {
        let mut sig = DbfSignal::new("TestSignal");
        sig.scale_factor = 0.5;
        sig.offset = 10.0;
        sig.min_value = 0.0;
        sig.max_value = 100.0;
        
        assert_eq!(sig.raw_to_physical(0.0), 10.0);
        assert_eq!(sig.raw_to_physical(100.0), 60.0);
        assert_eq!(sig.physical_to_raw(10.0), 0.0);
        assert_eq!(sig.physical_to_raw(60.0), 100.0);
    }
    
    #[test]
    fn test_dbf_signal_absolute_start_bit() {
        let mut sig = DbfSignal::new("TestSignal");
        sig.which_byte = 2;
        sig.start_bit = 3;
        assert_eq!(sig.absolute_start_bit(), 11);
    }

    #[test]
    fn test_parse_simple_dbf() {
        let dbf_content = r#"
//******************************BUSMASTER Messages and signals Database ******************************//

[DATABASE_VERSION] 1.3

[PROTOCOL] CAN

[BUSMASTER_VERSION] [2.3.0]

[NUMBER_OF_MESSAGES] 1

[START_MSG] msg123,123,8,2,1,S
[START_SIGNALS] Voltage,8,1,0,U,50,0,1,0.000000,0.250000,V,
[START_SIGNALS] RPM,16,2,0,U,10000,0,1,0.000000,0.500000,1/s,
[END_MSG]
"#;
        
        let db = DbfParser::parse(dbf_content).unwrap();
        
        assert_eq!(db.version, "1.3");
        assert_eq!(db.protocol, DbfProtocol::Can);
        assert_eq!(db.busmaster_version, "2.3.0");
        assert_eq!(db.messages.len(), 1);
        
        let msg = &db.messages[0];
        assert_eq!(msg.name, "msg123");
        assert_eq!(msg.id, 123);
        assert_eq!(msg.length, 8);
        assert_eq!(msg.frame_format, DbfFrameFormat::Standard);
        assert_eq!(msg.signals.len(), 2);
        
        let sig1 = &msg.signals[0];
        assert_eq!(sig1.name, "Voltage");
        assert_eq!(sig1.length, 8);
        assert_eq!(sig1.which_byte, 1);
        assert_eq!(sig1.start_bit, 0);
        assert_eq!(sig1.signal_type, DbfSignalType::Unsigned);
        assert_eq!(sig1.max_value, 50.0);
        assert_eq!(sig1.min_value, 0.0);
        assert_eq!(sig1.scale_factor, 0.25);
        assert_eq!(sig1.unit, "V");
        
        let sig2 = &msg.signals[1];
        assert_eq!(sig2.name, "RPM");
        assert_eq!(sig2.length, 16);
    }
    
    #[test]
    fn test_parse_extended_frame() {
        let dbf_content = r#"
[DATABASE_VERSION] 1.3
[PROTOCOL] CAN
[BUSMASTER_VERSION] [3.0.0]
[NUMBER_OF_MESSAGES] 1

[START_MSG] ExtMsg,2147483748,8,1,1,X
[START_SIGNALS] TestSig,8,1,0,U,255,0,1,0,1,unit,
[END_MSG]
"#;
        
        let db = DbfParser::parse(dbf_content).unwrap();
        assert_eq!(db.messages.len(), 1);
        
        let msg = &db.messages[0];
        assert_eq!(msg.name, "ExtMsg");
        assert_eq!(msg.id, 100);
        assert_eq!(msg.frame_format, DbfFrameFormat::Extended);
    }

    #[test]
    fn test_parse_with_nodes() {
        let dbf_content = r#"
[DATABASE_VERSION] 1.3
[PROTOCOL] CAN
[BUSMASTER_VERSION] [3.0.0]
[NUMBER_OF_MESSAGES] 0
[NODE] ECU1,ECU2,Gateway
"#;
        
        let db = DbfParser::parse(dbf_content).unwrap();
        assert_eq!(db.nodes.len(), 3);
        assert_eq!(db.nodes[0], "ECU1");
        assert_eq!(db.nodes[1], "ECU2");
        assert_eq!(db.nodes[2], "Gateway");
    }
    
    #[test]
    fn test_parse_j1939_protocol() {
        let dbf_content = r#"
[DATABASE_VERSION] 1.3
[PROTOCOL] J1939
[BUSMASTER_VERSION] [3.0.0]
[NUMBER_OF_MESSAGES] 0
"#;
        
        let db = DbfParser::parse(dbf_content).unwrap();
        assert_eq!(db.protocol, DbfProtocol::J1939);
    }
    
    #[test]
    fn test_parse_multiple_messages() {
        let dbf_content = r#"
[DATABASE_VERSION] 1.3
[PROTOCOL] CAN
[BUSMASTER_VERSION] [3.0.0]
[NUMBER_OF_MESSAGES] 3

[START_MSG] Msg1,100,8,1,1,S
[START_SIGNALS] Sig1,8,1,0,U,255,0,1,0,1,unit,
[END_MSG]

[START_MSG] Msg2,200,8,1,1,S
[START_SIGNALS] Sig2,16,1,0,I,32767,-32768,1,0,1,unit,
[END_MSG]

[START_MSG] Msg3,300,4,1,1,S
[START_SIGNALS] Sig3,32,1,0,F,0,0,1,0,1,unit,
[END_MSG]
"#;
        
        let db = DbfParser::parse(dbf_content).unwrap();
        assert_eq!(db.messages.len(), 3);
        assert_eq!(db.messages[0].id, 100);
        assert_eq!(db.messages[1].id, 200);
        assert_eq!(db.messages[2].id, 300);
        assert_eq!(db.messages[1].signals[0].signal_type, DbfSignalType::Signed);
        assert_eq!(db.messages[2].signals[0].signal_type, DbfSignalType::Float);
    }
    
    #[test]
    fn test_dbf_message_operations() {
        let mut msg = DbfMessage::new("TestMsg", 100);
        msg.length = 8;
        
        let sig1 = DbfSignal::new("Sig1");
        let sig2 = DbfSignal::new("Sig2");
        
        msg.add_signal(sig1);
        msg.add_signal(sig2);
        
        assert_eq!(msg.signals.len(), 2);
        assert_eq!(msg.signal_count, 2);
        
        assert!(msg.find_signal("Sig1").is_some());
        assert!(msg.find_signal("Sig3").is_none());
    }

    #[test]
    fn test_dbf_database_operations() {
        let mut db = DbfDatabase::new();
        db.add_node("ECU1");
        db.add_node("ECU2");
        
        let mut msg1 = DbfMessage::new("Msg1", 100);
        let sig1 = DbfSignal::new("Sig1");
        msg1.add_signal(sig1);
        db.add_message(msg1);
        
        let mut msg2 = DbfMessage::new("Msg2", 200);
        let sig2 = DbfSignal::new("Sig2");
        msg2.add_signal(sig2);
        db.add_message(msg2);
        
        assert_eq!(db.nodes.len(), 2);
        assert_eq!(db.messages.len(), 2);
        assert_eq!(db.total_signal_count(), 2);
        
        assert!(db.find_message_by_id(100).is_some());
        assert!(db.find_message_by_name("Msg2").is_some());
        assert!(db.find_signal(100, "Sig1").is_some());
        assert!(db.find_signal(100, "Sig2").is_none());
    }
    
    #[test]
    fn test_dbf_generator() {
        let mut db = DbfDatabase::new();
        db.version = "1.3".to_string();
        db.protocol = DbfProtocol::Can;
        db.busmaster_version = "3.0.0".to_string();
        db.add_node("ECU1");
        
        let mut msg = DbfMessage::new("TestMsg", 100);
        msg.length = 8;
        msg.frame_format = DbfFrameFormat::Standard;
        
        let mut sig = DbfSignal::new("TestSig");
        sig.length = 8;
        sig.which_byte = 1;
        sig.start_bit = 0;
        sig.signal_type = DbfSignalType::Unsigned;
        sig.max_value = 255.0;
        sig.min_value = 0.0;
        sig.scale_factor = 1.0;
        sig.offset = 0.0;
        sig.unit = "unit".to_string();
        
        msg.add_signal(sig);
        db.add_message(msg);
        
        let output = DbfGenerator::generate(&db);
        
        assert!(output.contains("[DATABASE_VERSION] 1.3"));
        assert!(output.contains("[PROTOCOL] CAN"));
        assert!(output.contains("[BUSMASTER_VERSION] [3.0.0]"));
        assert!(output.contains("[NODE] ECU1"));
        assert!(output.contains("[START_MSG] TestMsg,100,8,1,1,S"));
        assert!(output.contains("[START_SIGNALS] TestSig,8,1,0,U,255,0,1,0,1,unit,"));
        assert!(output.contains("[END_MSG]"));
    }
    
    #[test]
    fn test_rot13_decryption() {
        assert_eq!(DbfParser::decrypt_rot13("Uryyb"), "Hello");
        assert_eq!(DbfParser::decrypt_rot13("Hello"), "Uryyb");
        assert_eq!(DbfParser::decrypt_rot13("123"), "123");
        assert_eq!(DbfParser::decrypt_rot13("Test123"), "Grfg123");
    }
    
    #[test]
    fn test_value_descriptor() {
        let vd = DbfValueDescriptor::new(1, "On");
        assert_eq!(vd.value, 1);
        assert_eq!(vd.description, "On");
    }
    
    #[test]
    fn test_value_table() {
        let mut table = DbfValueTable::new("GearStates");
        table.add_value(0, "Park");
        table.add_value(1, "Reverse");
        table.add_value(2, "Neutral");
        table.add_value(3, "Drive");
        
        assert_eq!(table.name, "GearStates");
        assert_eq!(table.values.len(), 4);
        assert_eq!(table.values[0].description, "Park");
    }
    
    #[test]
    fn test_parse_empty_dbf() {
        let dbf_content = r#"
[DATABASE_VERSION] 1.3
[PROTOCOL] CAN
[BUSMASTER_VERSION] [3.0.0]
[NUMBER_OF_MESSAGES] 0
"#;
        
        let db = DbfParser::parse(dbf_content).unwrap();
        assert_eq!(db.messages.len(), 0);
        assert_eq!(db.version, "1.3");
    }
}
