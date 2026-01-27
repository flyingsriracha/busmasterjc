//! LDF (LIN Description File) Parser
//!
//! This module provides parsing for LIN Description Files, which define LIN networks,
//! nodes, frames, signals, and schedule tables.
//!
//! # LDF Format
//!
//! LDF files contain:
//! - LIN_protocol_version - Protocol version
//! - LIN_language_version - Language version
//! - LIN_speed - Bus speed in kbps
//! - Nodes - Master and slave nodes
//! - Signals - Signal definitions
//! - Frames - Frame definitions with signals
//! - Schedule_tables - Transmission schedules
//!
//! # Example
//!
//! ```
//! use busmaster_db::ldf::LdfParser;
//!
//! let ldf = r#"
//! LIN_description_file;
//! LIN_protocol_version = "2.1";
//! LIN_language_version = "2.1";
//! LIN_speed = 19.2 kbps;
//!
//! Nodes {
//!   Master: Master, 5 ms, 0.1 ms;
//!   Slaves: Slave1, Slave2;
//! }
//! "#;
//!
//! let db = LdfParser::parse(ldf).unwrap();
//! assert_eq!(db.protocol_version, "2.1");
//! ```

use busmaster_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LDF database containing all parsed information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfDatabase {
    /// LIN protocol version (e.g., "2.1")
    pub protocol_version: String,
    /// LIN language version (e.g., "2.1")
    pub language_version: String,
    /// Bus speed in kbps
    pub speed_kbps: f64,
    /// Master node
    pub master: Option<LdfMasterNode>,
    /// Slave nodes
    pub slaves: Vec<LdfSlaveNode>,
    /// Signal definitions
    pub signals: Vec<LdfSignal>,
    /// Frame definitions
    pub frames: Vec<LdfFrame>,
    /// Schedule tables
    pub schedule_tables: Vec<LdfScheduleTable>,
    /// Node attributes
    pub node_attributes: HashMap<String, LdfNodeAttributes>,
}

impl LdfDatabase {
    /// Create a new empty database
    #[must_use]
    pub fn new() -> Self {
        Self {
            protocol_version: String::new(),
            language_version: String::new(),
            speed_kbps: 19.2,
            master: None,
            slaves: Vec::new(),
            signals: Vec::new(),
            frames: Vec::new(),
            schedule_tables: Vec::new(),
            node_attributes: HashMap::new(),
        }
    }

    /// Find a frame by ID
    #[must_use]
    pub fn find_frame(&self, id: u8) -> Option<&LdfFrame> {
        self.frames.iter().find(|f| f.id == id)
    }

    /// Find a frame by name
    #[must_use]
    pub fn find_frame_by_name(&self, name: &str) -> Option<&LdfFrame> {
        self.frames.iter().find(|f| f.name == name)
    }

    /// Find a signal by name
    #[must_use]
    pub fn find_signal(&self, name: &str) -> Option<&LdfSignal> {
        self.signals.iter().find(|s| s.name == name)
    }

    /// Find a schedule table by name
    #[must_use]
    pub fn find_schedule_table(&self, name: &str) -> Option<&LdfScheduleTable> {
        self.schedule_tables.iter().find(|t| t.name == name)
    }
}

impl Default for LdfDatabase {
    fn default() -> Self {
        Self::new()
    }
}


/// LDF master node definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfMasterNode {
    /// Node name
    pub name: String,
    /// Time base in milliseconds
    pub time_base_ms: f64,
    /// Jitter in milliseconds
    pub jitter_ms: f64,
}

impl LdfMasterNode {
    /// Create a new master node
    #[must_use]
    pub fn new(name: &str, time_base_ms: f64, jitter_ms: f64) -> Self {
        Self {
            name: name.to_string(),
            time_base_ms,
            jitter_ms,
        }
    }
}

/// LDF slave node definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfSlaveNode {
    /// Node name
    pub name: String,
    /// Configured NAD (Node Address)
    pub configured_nad: Option<u8>,
    /// Initial NAD
    pub initial_nad: Option<u8>,
    /// Product ID (supplier, function, variant)
    pub product_id: Option<(u16, u16, u8)>,
    /// Response error signal name
    pub response_error: Option<String>,
    /// P2 min time in milliseconds
    pub p2_min_ms: Option<f64>,
    /// ST min time in milliseconds
    pub st_min_ms: Option<f64>,
}

impl LdfSlaveNode {
    /// Create a new slave node
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            configured_nad: None,
            initial_nad: None,
            product_id: None,
            response_error: None,
            p2_min_ms: None,
            st_min_ms: None,
        }
    }
}


/// LDF signal definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfSignal {
    /// Signal name
    pub name: String,
    /// Signal size in bits
    pub size: u8,
    /// Initial value
    pub init_value: u64,
    /// Publisher node name
    pub publisher: String,
    /// Subscriber node names
    pub subscribers: Vec<String>,
}

impl LdfSignal {
    /// Create a new signal
    #[must_use]
    pub fn new(name: &str, size: u8, init_value: u64, publisher: &str) -> Self {
        Self {
            name: name.to_string(),
            size,
            init_value,
            publisher: publisher.to_string(),
            subscribers: Vec::new(),
        }
    }

    /// Add a subscriber
    pub fn add_subscriber(&mut self, subscriber: &str) {
        self.subscribers.push(subscriber.to_string());
    }
}

/// LDF frame signal reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfFrameSignal {
    /// Signal name
    pub name: String,
    /// Bit offset within frame
    pub offset: u8,
}

impl LdfFrameSignal {
    /// Create a new frame signal reference
    #[must_use]
    pub fn new(name: &str, offset: u8) -> Self {
        Self {
            name: name.to_string(),
            offset,
        }
    }
}


/// LDF frame definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfFrame {
    /// Frame name
    pub name: String,
    /// Frame ID (0-63)
    pub id: u8,
    /// Publisher node name
    pub publisher: String,
    /// Frame length in bytes
    pub length: u8,
    /// Signals in this frame
    pub signals: Vec<LdfFrameSignal>,
}

impl LdfFrame {
    /// Create a new frame
    #[must_use]
    pub fn new(name: &str, id: u8, publisher: &str, length: u8) -> Self {
        Self {
            name: name.to_string(),
            id,
            publisher: publisher.to_string(),
            length,
            signals: Vec::new(),
        }
    }

    /// Add a signal to the frame
    pub fn add_signal(&mut self, name: &str, offset: u8) {
        self.signals.push(LdfFrameSignal::new(name, offset));
    }
}

/// LDF schedule table entry type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LdfScheduleEntryType {
    /// Unconditional frame
    Frame(String),
    /// Master request frame
    MasterReq,
    /// Slave response frame
    SlaveResp,
    /// Assign NAD command
    AssignNad {
        /// Target node name
        node: String,
    },
    /// Conditional change NAD command
    ConditionalChangeNad {
        /// Current NAD
        nad: u8,
        /// Identifier
        id: u8,
        /// Byte position
        byte: u8,
        /// Mask value
        mask: u8,
        /// Invert flag
        inv: u8,
        /// New NAD value
        new_nad: u8,
    },
    /// Data dump command
    DataDump {
        /// Target node name
        node: String,
        /// Data bytes
        data: Vec<u8>,
    },
    /// Save configuration command
    SaveConfiguration {
        /// Target node name
        node: String,
    },
    /// Assign frame ID range command
    AssignFrameIdRange {
        /// Target node name
        node: String,
        /// Frame index
        frame_index: u8,
    },
    /// Free format command
    FreeFormat {
        /// Raw data bytes
        data: Vec<u8>,
    },
}


/// LDF schedule table entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfScheduleEntry {
    /// Entry type
    pub entry_type: LdfScheduleEntryType,
    /// Delay in milliseconds
    pub delay_ms: f64,
}

impl LdfScheduleEntry {
    /// Create a new schedule entry
    #[must_use]
    pub fn new(entry_type: LdfScheduleEntryType, delay_ms: f64) -> Self {
        Self { entry_type, delay_ms }
    }

    /// Create a frame entry
    #[must_use]
    pub fn frame(name: &str, delay_ms: f64) -> Self {
        Self::new(LdfScheduleEntryType::Frame(name.to_string()), delay_ms)
    }

    /// Create a master request entry
    #[must_use]
    pub fn master_req(delay_ms: f64) -> Self {
        Self::new(LdfScheduleEntryType::MasterReq, delay_ms)
    }

    /// Create a slave response entry
    #[must_use]
    pub fn slave_resp(delay_ms: f64) -> Self {
        Self::new(LdfScheduleEntryType::SlaveResp, delay_ms)
    }
}

/// LDF schedule table
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdfScheduleTable {
    /// Table name
    pub name: String,
    /// Schedule entries
    pub entries: Vec<LdfScheduleEntry>,
}

impl LdfScheduleTable {
    /// Create a new schedule table
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Vec::new(),
        }
    }

    /// Add an entry to the schedule
    pub fn add_entry(&mut self, entry: LdfScheduleEntry) {
        self.entries.push(entry);
    }

    /// Get total cycle time in milliseconds
    #[must_use]
    pub fn cycle_time_ms(&self) -> f64 {
        self.entries.iter().map(|e| e.delay_ms).sum()
    }
}


/// LDF node attributes
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LdfNodeAttributes {
    /// LIN protocol version
    pub lin_protocol: Option<String>,
    /// Configured NAD
    pub configured_nad: Option<u8>,
    /// Initial NAD
    pub initial_nad: Option<u8>,
    /// Product ID
    pub product_id: Option<(u16, u16, u8)>,
    /// Response error signal
    pub response_error: Option<String>,
    /// Configurable frames
    pub configurable_frames: Vec<(String, Option<u8>)>,
}

/// LDF Parser
pub struct LdfParser;

#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::while_let_on_iterator)]
#[allow(clippy::needless_continue)]
#[allow(clippy::redundant_else)]
#[allow(clippy::trim_split_whitespace)]
impl LdfParser {
    /// Parse an LDF file content
    pub fn parse(content: &str) -> Result<LdfDatabase> {
        let mut db = LdfDatabase::new();
        let mut lines = content.lines().peekable();

        while let Some(line) = lines.next() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Parse LIN_description_file header
            if line.starts_with("LIN_description_file") {
                continue;
            }

            // Parse protocol version
            if line.starts_with("LIN_protocol_version") {
                if let Some(version) = Self::parse_quoted_value(line) {
                    db.protocol_version = version;
                }
                continue;
            }

            // Parse language version
            if line.starts_with("LIN_language_version") {
                if let Some(version) = Self::parse_quoted_value(line) {
                    db.language_version = version;
                }
                continue;
            }

            // Parse speed
            if line.starts_with("LIN_speed") {
                if let Some(speed) = Self::parse_speed(line) {
                    db.speed_kbps = speed;
                }
                continue;
            }


            // Parse Nodes section
            if line.starts_with("Nodes") {
                Self::parse_nodes_section(&mut db, &mut lines)?;
                continue;
            }

            // Parse Signals section
            if line.starts_with("Signals") {
                Self::parse_signals_section(&mut db, &mut lines)?;
                continue;
            }

            // Parse Frames section
            if line.starts_with("Frames") {
                Self::parse_frames_section(&mut db, &mut lines)?;
                continue;
            }

            // Parse Schedule_tables section
            if line.starts_with("Schedule_tables") {
                Self::parse_schedule_tables_section(&mut db, &mut lines)?;
                continue;
            }

            // Parse Node_attributes section
            if line.starts_with("Node_attributes") {
                Self::parse_node_attributes_section(&mut db, &mut lines)?;
                continue;
            }
        }

        Ok(db)
    }

    fn parse_quoted_value(line: &str) -> Option<String> {
        let start = line.find('"')?;
        let end = line[start + 1..].find('"')?;
        Some(line[start + 1..start + 1 + end].to_string())
    }

    fn parse_speed(line: &str) -> Option<f64> {
        // Format: LIN_speed = 19.2 kbps;
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() < 2 {
            return None;
        }
        let value_part = parts[1].trim();
        let speed_str = value_part.split_whitespace().next()?;
        speed_str.parse().ok()
    }


    fn parse_nodes_section<'a, I>(db: &mut LdfDatabase, lines: &mut std::iter::Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        // The section header line may contain the opening brace
        // We need to look for content after the brace on the same line or on subsequent lines

        while let Some(line) = lines.next() {
            let line = line.trim();

            if line == "}" || line.starts_with('}') {
                break;
            }

            if line.is_empty() || line == "{" {
                continue;
            }

            // Parse Master line: Master: NodeName, time_base ms, jitter ms;
            if line.starts_with("Master:") || line.starts_with("Master :") {
                let content = line.trim_start_matches("Master:").trim_start_matches("Master :").trim();
                let parts: Vec<&str> = content.split(',').collect();
                if !parts.is_empty() {
                    let name = parts[0].trim().trim_end_matches(';');
                    let time_base = parts.get(1)
                        .and_then(|s| s.trim().split_whitespace().next())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(5.0);
                    let jitter = parts.get(2)
                        .and_then(|s| s.trim().split_whitespace().next())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.1);
                    db.master = Some(LdfMasterNode::new(name, time_base, jitter));
                }
            }

            // Parse Slaves line: Slaves: Node1, Node2, ...;
            if line.starts_with("Slaves:") || line.starts_with("Slaves :") {
                let content = line.trim_start_matches("Slaves:").trim_start_matches("Slaves :").trim();
                let content = content.trim_end_matches(';');
                for name in content.split(',') {
                    let name = name.trim();
                    if !name.is_empty() {
                        db.slaves.push(LdfSlaveNode::new(name));
                    }
                }
            }
        }

        Ok(())
    }


    fn parse_signals_section<'a, I>(db: &mut LdfDatabase, lines: &mut std::iter::Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        while let Some(line) = lines.next() {
            let line = line.trim();

            if line == "}" || line.starts_with('}') {
                break;
            }

            if line.is_empty() || line.starts_with("//") || line == "{" {
                continue;
            }

            // Parse signal: SignalName: size, init_value, publisher, subscriber1, subscriber2, ...;
            if let Some(signal) = Self::parse_signal_line(line) {
                db.signals.push(signal);
            }
        }

        Ok(())
    }

    fn parse_signal_line(line: &str) -> Option<LdfSignal> {
        let line = line.trim_end_matches(';');
        let colon_pos = line.find(':')?;
        let name = line[..colon_pos].trim();
        let rest = line[colon_pos + 1..].trim();

        let parts: Vec<&str> = rest.split(',').collect();
        if parts.len() < 3 {
            return None;
        }

        let size: u8 = parts[0].trim().parse().ok()?;
        let init_value: u64 = parts[1].trim().parse().ok()?;
        let publisher = parts[2].trim();

        let mut signal = LdfSignal::new(name, size, init_value, publisher);

        // Add subscribers
        for subscriber in parts.iter().skip(3) {
            let subscriber = subscriber.trim();
            if !subscriber.is_empty() {
                signal.add_subscriber(subscriber);
            }
        }

        Some(signal)
    }


    fn parse_frames_section<'a, I>(db: &mut LdfDatabase, lines: &mut std::iter::Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut current_frame: Option<LdfFrame> = None;
        let mut brace_depth = 0;

        while let Some(line) = lines.next() {
            let line = line.trim();

            // Track brace depth
            if line == "{" {
                brace_depth += 1;
                continue;
            }

            if line == "}" {
                if brace_depth > 0 {
                    brace_depth -= 1;
                    // Save current frame when closing its brace
                    if brace_depth == 0 {
                        if let Some(frame) = current_frame.take() {
                            db.frames.push(frame);
                        }
                    }
                    continue;
                } else {
                    // End of Frames section
                    if let Some(frame) = current_frame.take() {
                        db.frames.push(frame);
                    }
                    break;
                }
            }

            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Check if this is a frame definition or signal reference
            if line.contains(':') && brace_depth == 0 {
                // Save previous frame
                if let Some(frame) = current_frame.take() {
                    db.frames.push(frame);
                }

                // Parse new frame: FrameName: id, publisher, length { or FrameName: id, publisher, length;
                if let Some(frame) = Self::parse_frame_header(line) {
                    current_frame = Some(frame);
                    if line.contains('{') {
                        brace_depth = 1;
                    }
                }
            } else if let Some(ref mut frame) = current_frame {
                // Parse signal reference: SignalName, offset;
                if let Some((name, offset)) = Self::parse_frame_signal(line) {
                    frame.add_signal(&name, offset);
                }
            }
        }

        Ok(())
    }

    fn parse_frame_header(line: &str) -> Option<LdfFrame> {
        let line = line.trim_end_matches('{').trim_end_matches(';').trim();
        let colon_pos = line.find(':')?;
        let name = line[..colon_pos].trim();
        let rest = line[colon_pos + 1..].trim();

        let parts: Vec<&str> = rest.split(',').collect();
        if parts.len() < 3 {
            return None;
        }

        // Parse ID - may be decimal or hex (0x prefix)
        let id_str = parts[0].trim();
        let id: u8 = if id_str.starts_with("0x") || id_str.starts_with("0X") {
            u8::from_str_radix(&id_str[2..], 16).ok()?
        } else {
            id_str.parse().ok()?
        };

        let publisher = parts[1].trim();
        let length: u8 = parts[2].trim().parse().ok()?;

        Some(LdfFrame::new(name, id, publisher, length))
    }

    fn parse_frame_signal(line: &str) -> Option<(String, u8)> {
        let line = line.trim_end_matches(';').trim_end_matches('}').trim();
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            return None;
        }

        let name = parts[0].trim().to_string();
        let offset: u8 = parts[1].trim().parse().ok()?;

        Some((name, offset))
    }


    fn parse_schedule_tables_section<'a, I>(db: &mut LdfDatabase, lines: &mut std::iter::Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut current_table: Option<LdfScheduleTable> = None;
        let mut brace_depth = 0;

        while let Some(line) = lines.next() {
            let line = line.trim();

            if line == "{" {
                brace_depth += 1;
                continue;
            }

            if line == "}" {
                if brace_depth > 0 {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        if let Some(table) = current_table.take() {
                            db.schedule_tables.push(table);
                        }
                    }
                    continue;
                } else {
                    if let Some(table) = current_table.take() {
                        db.schedule_tables.push(table);
                    }
                    break;
                }
            }

            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Check if this is a table name or entry
            if line.ends_with('{') && brace_depth == 0 {
                // Save previous table
                if let Some(table) = current_table.take() {
                    db.schedule_tables.push(table);
                }

                // New table: TableName {
                let name = line.trim_end_matches('{').trim();
                current_table = Some(LdfScheduleTable::new(name));
                brace_depth = 1;
            } else if let Some(ref mut table) = current_table {
                // Parse entry: FrameName delay X ms;
                if let Some(entry) = Self::parse_schedule_entry(line) {
                    table.add_entry(entry);
                }
            }
        }

        Ok(())
    }

    fn parse_schedule_entry(line: &str) -> Option<LdfScheduleEntry> {
        let line = line.trim_end_matches(';').trim();

        // Find "delay" keyword
        let delay_pos = line.find("delay")?;
        let frame_part = line[..delay_pos].trim();
        let delay_part = line[delay_pos + 5..].trim();

        // Parse delay value (e.g., "10 ms")
        let delay_ms: f64 = delay_part.split_whitespace().next()?.parse().ok()?;

        // Determine entry type
        let entry_type = match frame_part {
            "MasterReq" => LdfScheduleEntryType::MasterReq,
            "SlaveResp" => LdfScheduleEntryType::SlaveResp,
            name => LdfScheduleEntryType::Frame(name.to_string()),
        };

        Some(LdfScheduleEntry::new(entry_type, delay_ms))
    }


    fn parse_node_attributes_section<'a, I>(db: &mut LdfDatabase, lines: &mut std::iter::Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut current_node: Option<String> = None;
        let mut current_attrs = LdfNodeAttributes::default();
        let mut brace_depth = 0;

        while let Some(line) = lines.next() {
            let line = line.trim();

            if line == "{" {
                brace_depth += 1;
                continue;
            }

            if line == "}" {
                if brace_depth > 0 {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        if let Some(node) = current_node.take() {
                            db.node_attributes.insert(node, current_attrs);
                            current_attrs = LdfNodeAttributes::default();
                        }
                    }
                    continue;
                } else {
                    if let Some(node) = current_node.take() {
                        db.node_attributes.insert(node, current_attrs);
                    }
                    break;
                }
            }

            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Check if this is a node name
            if line.ends_with('{') && brace_depth == 0 {
                // Save previous node
                if let Some(node) = current_node.take() {
                    db.node_attributes.insert(node, current_attrs);
                    current_attrs = LdfNodeAttributes::default();
                }

                let name = line.trim_end_matches('{').trim();
                current_node = Some(name.to_string());
                brace_depth = 1;
            } else if current_node.is_some() {
                // Parse attribute
                Self::parse_node_attribute(line, &mut current_attrs);
            }
        }

        Ok(())
    }

    fn parse_node_attribute(line: &str, attrs: &mut LdfNodeAttributes) {
        let line = line.trim_end_matches(';').trim();

        if line.starts_with("LIN_protocol") {
            if let Some(version) = Self::parse_quoted_value(line) {
                attrs.lin_protocol = Some(version);
            }
        } else if line.starts_with("configured_NAD") {
            if let Some(pos) = line.find('=') {
                if let Ok(nad) = line[pos + 1..].trim().trim_start_matches("0x").parse::<u8>() {
                    attrs.configured_nad = Some(nad);
                }
            }
        } else if line.starts_with("initial_NAD") {
            if let Some(pos) = line.find('=') {
                if let Ok(nad) = line[pos + 1..].trim().trim_start_matches("0x").parse::<u8>() {
                    attrs.initial_nad = Some(nad);
                }
            }
        } else if line.starts_with("response_error") {
            if let Some(pos) = line.find('=') {
                attrs.response_error = Some(line[pos + 1..].trim().to_string());
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_ldf() {
        let ldf = r#"
LIN_description_file;
LIN_protocol_version = "2.1";
LIN_language_version = "2.1";
LIN_speed = 19.2 kbps;

Nodes {
  Master: Master, 5 ms, 0.1 ms;
  Slaves: Slave1, Slave2;
}
"#;

        let db = LdfParser::parse(ldf).unwrap();
        assert_eq!(db.protocol_version, "2.1");
        assert_eq!(db.language_version, "2.1");
        assert!((db.speed_kbps - 19.2).abs() < 0.01);
        assert!(db.master.is_some());
        let master = db.master.as_ref().unwrap();
        assert_eq!(master.name, "Master");
        assert!((master.time_base_ms - 5.0).abs() < 0.01);
        assert_eq!(db.slaves.len(), 2);
        assert_eq!(db.slaves[0].name, "Slave1");
        assert_eq!(db.slaves[1].name, "Slave2");
    }

    #[test]
    fn test_parse_signals() {
        let ldf = r#"
LIN_description_file;
LIN_protocol_version = "2.1";

Signals {
  Signal1: 8, 0, Slave1, Master;
  Signal2: 16, 100, Slave2, Master, Slave1;
}
"#;

        let db = LdfParser::parse(ldf).unwrap();
        assert_eq!(db.signals.len(), 2);

        let sig1 = &db.signals[0];
        assert_eq!(sig1.name, "Signal1");
        assert_eq!(sig1.size, 8);
        assert_eq!(sig1.init_value, 0);
        assert_eq!(sig1.publisher, "Slave1");
        assert_eq!(sig1.subscribers, vec!["Master"]);

        let sig2 = &db.signals[1];
        assert_eq!(sig2.name, "Signal2");
        assert_eq!(sig2.size, 16);
        assert_eq!(sig2.init_value, 100);
        assert_eq!(sig2.subscribers.len(), 2);
    }


    #[test]
    fn test_parse_frames() {
        let ldf = r#"
LIN_description_file;
LIN_protocol_version = "2.1";

Frames {
  Frame1: 0x10, Slave1, 4 {
    Signal1, 0;
    Signal2, 8;
  }
  Frame2: 0x20, Slave2, 8 {
    Signal3, 0;
  }
}
"#;

        let db = LdfParser::parse(ldf).unwrap();
        assert_eq!(db.frames.len(), 2);

        let frame1 = &db.frames[0];
        assert_eq!(frame1.name, "Frame1");
        assert_eq!(frame1.id, 0x10);
        assert_eq!(frame1.publisher, "Slave1");
        assert_eq!(frame1.length, 4);
        assert_eq!(frame1.signals.len(), 2);
        assert_eq!(frame1.signals[0].name, "Signal1");
        assert_eq!(frame1.signals[0].offset, 0);
        assert_eq!(frame1.signals[1].name, "Signal2");
        assert_eq!(frame1.signals[1].offset, 8);

        let frame2 = &db.frames[1];
        assert_eq!(frame2.name, "Frame2");
        assert_eq!(frame2.id, 0x20);
    }

    #[test]
    fn test_parse_schedule_tables() {
        let ldf = r#"
LIN_description_file;
LIN_protocol_version = "2.1";

Schedule_tables {
  MainSchedule {
    Frame1 delay 10 ms;
    Frame2 delay 20 ms;
    MasterReq delay 10 ms;
    SlaveResp delay 10 ms;
  }
}
"#;

        let db = LdfParser::parse(ldf).unwrap();
        assert_eq!(db.schedule_tables.len(), 1);

        let table = &db.schedule_tables[0];
        assert_eq!(table.name, "MainSchedule");
        assert_eq!(table.entries.len(), 4);

        assert!(matches!(table.entries[0].entry_type, LdfScheduleEntryType::Frame(ref n) if n == "Frame1"));
        assert!((table.entries[0].delay_ms - 10.0).abs() < 0.01);

        assert!(matches!(table.entries[2].entry_type, LdfScheduleEntryType::MasterReq));
        assert!(matches!(table.entries[3].entry_type, LdfScheduleEntryType::SlaveResp));

        assert!((table.cycle_time_ms() - 50.0).abs() < 0.01);
    }


    #[test]
    fn test_ldf_database_find_methods() {
        let mut db = LdfDatabase::new();
        db.frames.push(LdfFrame::new("TestFrame", 0x10, "Slave1", 8));
        db.signals.push(LdfSignal::new("TestSignal", 8, 0, "Slave1"));
        db.schedule_tables.push(LdfScheduleTable::new("TestSchedule"));

        assert!(db.find_frame(0x10).is_some());
        assert!(db.find_frame(0x99).is_none());

        assert!(db.find_frame_by_name("TestFrame").is_some());
        assert!(db.find_frame_by_name("Unknown").is_none());

        assert!(db.find_signal("TestSignal").is_some());
        assert!(db.find_signal("Unknown").is_none());

        assert!(db.find_schedule_table("TestSchedule").is_some());
        assert!(db.find_schedule_table("Unknown").is_none());
    }

    #[test]
    fn test_ldf_master_node() {
        let master = LdfMasterNode::new("Master", 5.0, 0.1);
        assert_eq!(master.name, "Master");
        assert!((master.time_base_ms - 5.0).abs() < 0.01);
        assert!((master.jitter_ms - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_ldf_slave_node() {
        let slave = LdfSlaveNode::new("Slave1");
        assert_eq!(slave.name, "Slave1");
        assert!(slave.configured_nad.is_none());
        assert!(slave.initial_nad.is_none());
    }

    #[test]
    fn test_ldf_signal() {
        let mut signal = LdfSignal::new("TestSignal", 16, 100, "Publisher");
        signal.add_subscriber("Sub1");
        signal.add_subscriber("Sub2");

        assert_eq!(signal.name, "TestSignal");
        assert_eq!(signal.size, 16);
        assert_eq!(signal.init_value, 100);
        assert_eq!(signal.publisher, "Publisher");
        assert_eq!(signal.subscribers.len(), 2);
    }

    #[test]
    fn test_ldf_frame() {
        let mut frame = LdfFrame::new("TestFrame", 0x10, "Publisher", 8);
        frame.add_signal("Signal1", 0);
        frame.add_signal("Signal2", 8);

        assert_eq!(frame.name, "TestFrame");
        assert_eq!(frame.id, 0x10);
        assert_eq!(frame.publisher, "Publisher");
        assert_eq!(frame.length, 8);
        assert_eq!(frame.signals.len(), 2);
    }

    #[test]
    fn test_ldf_schedule_entry() {
        let entry1 = LdfScheduleEntry::frame("Frame1", 10.0);
        assert!(matches!(entry1.entry_type, LdfScheduleEntryType::Frame(ref n) if n == "Frame1"));
        assert!((entry1.delay_ms - 10.0).abs() < 0.01);

        let entry2 = LdfScheduleEntry::master_req(5.0);
        assert!(matches!(entry2.entry_type, LdfScheduleEntryType::MasterReq));

        let entry3 = LdfScheduleEntry::slave_resp(5.0);
        assert!(matches!(entry3.entry_type, LdfScheduleEntryType::SlaveResp));
    }

    #[test]
    fn test_ldf_schedule_table() {
        let mut table = LdfScheduleTable::new("TestSchedule");
        table.add_entry(LdfScheduleEntry::frame("Frame1", 10.0));
        table.add_entry(LdfScheduleEntry::frame("Frame2", 20.0));

        assert_eq!(table.name, "TestSchedule");
        assert_eq!(table.entries.len(), 2);
        assert!((table.cycle_time_ms() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_empty_ldf() {
        let ldf = "LIN_description_file;";
        let db = LdfParser::parse(ldf).unwrap();
        assert!(db.protocol_version.is_empty());
        assert!(db.frames.is_empty());
    }

    #[test]
    fn test_parse_ldf_with_comments() {
        let ldf = r#"
// This is a comment
LIN_description_file;
LIN_protocol_version = "2.1";
// Another comment
LIN_speed = 19.2 kbps;
"#;

        let db = LdfParser::parse(ldf).unwrap();
        assert_eq!(db.protocol_version, "2.1");
        assert!((db.speed_kbps - 19.2).abs() < 0.01);
    }
}
