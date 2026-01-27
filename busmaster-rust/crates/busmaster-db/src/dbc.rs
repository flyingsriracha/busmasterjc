//! DBC (CAN Database) file parser
//!
//! This module provides parsing for Vector DBC files, which define CAN messages,
//! signals, nodes, and their relationships.
//!
//! # DBC Format
//!
//! DBC files contain:
//! - VERSION - File version
//! - BU_ - Network nodes (ECUs)
//! - BO_ - Messages
//! - SG_ - Signals within messages
//! - CM_ - Comments
//! - VAL_ - Value descriptions (enumerations)
//! - BA_DEF_ - Attribute definitions
//! - BA_ - Attribute values
//!
//! # Example
//!
//! ```
//! use busmaster_db::dbc::DbcParser;
//!
//! let dbc = r#"
//! VERSION ""
//! BU_: ECU1 ECU2
//! BO_ 100 EngineData: 8 ECU1
//!  SG_ EngineSpeed : 0|16@1+ (0.25,0) [0|16383.75] "rpm" ECU2
//!  SG_ EngineTemp : 16|8@1+ (1,-40) [-40|215] "degC" ECU2
//! "#;
//!
//! let db = DbcParser::parse(dbc).unwrap();
//! assert_eq!(db.nodes.len(), 2);
//! assert_eq!(db.messages.len(), 1);
//! assert_eq!(db.messages[0].signals.len(), 2);
//! ```

use busmaster_core::{BusmasterError, ByteOrder, Result, SignalDef, ValueType};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, digit1, line_ending, multispace0, space0, space1},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DBC database containing all parsed information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbcDatabase {
    /// File version string
    pub version: String,
    /// Network nodes (ECUs)
    pub nodes: Vec<String>,
    /// CAN messages
    pub messages: Vec<DbcMessage>,
    /// Comments (message ID -> comment)
    pub comments: HashMap<u32, String>,
    /// Value descriptions (message ID, signal name -> values)
    pub value_descriptions: HashMap<(u32, String), HashMap<i64, String>>,
}

impl DbcDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            version: String::new(),
            nodes: Vec::new(),
            messages: Vec::new(),
            comments: HashMap::new(),
            value_descriptions: HashMap::new(),
        }
    }

    /// Find a message by ID
    pub fn find_message(&self, id: u32) -> Option<&DbcMessage> {
        self.messages.iter().find(|m| m.id == id)
    }

    /// Find a signal by message ID and signal name
    pub fn find_signal(&self, message_id: u32, signal_name: &str) -> Option<&DbcSignal> {
        self.find_message(message_id)?
            .signals
            .iter()
            .find(|s| s.name == signal_name)
    }
}

impl Default for DbcDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// CAN message definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbcMessage {
    /// Message ID (11-bit or 29-bit)
    pub id: u32,
    /// Message name
    pub name: String,
    /// Data length in bytes (up to 64 for CAN FD)
    pub length: u8,
    /// Transmitting node
    pub transmitter: String,
    /// Signals in this message
    pub signals: Vec<DbcSignal>,
    /// Whether this is a CAN FD message (length > 8)
    pub is_fd: bool,
}

impl DbcMessage {
    /// Create a new message
    pub fn new(id: u32, name: String, length: u8, transmitter: String) -> Self {
        Self {
            id,
            name,
            length,
            transmitter,
            signals: Vec::new(),
            is_fd: length > 8,
        }
    }

    /// Add a signal to this message
    pub fn add_signal(&mut self, signal: DbcSignal) {
        self.signals.push(signal);
    }

    /// Check if this message requires CAN FD
    #[must_use]
    pub fn requires_fd(&self) -> bool {
        self.is_fd || self.length > 8
    }

    /// Get the CAN FD DLC value for this message length
    #[must_use]
    pub fn fd_dlc(&self) -> u8 {
        match self.length {
            0..=8 => self.length,
            9..=12 => 12,
            13..=16 => 16,
            17..=20 => 20,
            21..=24 => 24,
            25..=32 => 32,
            33..=48 => 48,
            _ => 64,
        }
    }
}

/// Signal definition within a message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbcSignal {
    /// Signal name
    pub name: String,
    /// Start bit position
    pub start_bit: u16,
    /// Length in bits
    pub length: u16,
    /// Byte order (0 = Motorola/big-endian, 1 = Intel/little-endian)
    pub byte_order: ByteOrder,
    /// Value type (signed/unsigned)
    pub value_type: ValueType,
    /// Scale factor
    pub factor: f64,
    /// Offset
    pub offset: f64,
    /// Minimum value
    pub minimum: f64,
    /// Maximum value
    pub maximum: f64,
    /// Unit string
    pub unit: String,
    /// Receiving nodes
    pub receivers: Vec<String>,
}

impl DbcSignal {
    /// Convert to SignalDef for signal extraction
    pub fn to_signal_def(&self) -> SignalDef {
        SignalDef::new(&self.name, self.start_bit, self.length)
            .with_byte_order(self.byte_order)
            .with_value_type(self.value_type)
            .with_factor(self.factor)
            .with_offset(self.offset)
            .with_unit(&self.unit)
    }
}

/// DBC file parser
pub struct DbcParser;

impl DbcParser {
    /// Parse a DBC file from a string
    ///
    /// # Arguments
    ///
    /// * `input` - DBC file content as a string
    ///
    /// # Returns
    ///
    /// A parsed `DbcDatabase` or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the DBC file is malformed or contains invalid data.
    ///
    /// # Example
    ///
    /// ```
    /// use busmaster_db::dbc::DbcParser;
    ///
    /// let dbc = r#"
    /// VERSION ""
    /// BU_: Node1
    /// BO_ 100 TestMsg: 8 Node1
    ///  SG_ TestSig : 0|8@1+ (1,0) [0|255] "" Node1
    /// "#;
    ///
    /// let db = DbcParser::parse(dbc).unwrap();
    /// assert_eq!(db.messages.len(), 1);
    /// ```
    pub fn parse(input: &str) -> Result<DbcDatabase> {
        match parse_dbc(input) {
            Ok((_, db)) => Ok(db),
            Err(e) => Err(BusmasterError::DatabaseParse {
                message: format!("Failed to parse DBC: {}", e),
                line: 0,
            }),
        }
    }
}

// Parser implementation using nom

fn parse_dbc(input: &str) -> IResult<&str, DbcDatabase> {
    let (input, _) = multispace0(input)?;
    let (input, version) = parse_version(input)?;
    let (input, _) = skip_to_next_section(input)?;
    let (input, nodes) = parse_nodes(input)?;
    let (input, _) = skip_to_next_section(input)?;
    let (input, messages) = parse_messages(input)?;
    let (input, _) = multispace0(input)?;
    let (input, comments) = parse_comments(input)?;
    let (input, _) = multispace0(input)?;
    let (input, value_descriptions) = parse_value_descriptions(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        DbcDatabase {
            version,
            nodes,
            messages,
            comments,
            value_descriptions,
        },
    ))
}

fn parse_version(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("VERSION")(input)?;
    let (input, _) = space1(input)?;
    let (input, version) = delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, version.to_string()))
}

fn skip_to_next_section(input: &str) -> IResult<&str, ()> {
    // Skip NS_ and BS_ sections which can be multi-line
    // Keep consuming lines until we hit BU_: or BO_ or end of input
    let (input, _) = many0(alt((
        // Skip entire NS_ section (can have multiple indented lines)
        map(
            tuple((
                tag("NS_"),
                take_until("\n"),
                line_ending,
                many0(tuple((
                    space1, // Indented lines in NS_ section
                    take_until("\n"),
                    line_ending,
                ))),
            )),
            |_| (),
        ),
        // Skip BS_ line
        map(preceded(tag("BS_"), take_until("\n")), |_| ()),
        // Skip empty lines and whitespace
        map(line_ending, |_| ()),
        map(space1, |_| ()),
    )))(input)?;
    Ok((input, ()))
}

fn parse_nodes(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = tag("BU_:")(input)?;
    let (input, _) = space0(input)?;
    let (input, nodes) = separated_list0(space1, parse_identifier)(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, nodes))
}

fn parse_messages(input: &str) -> IResult<&str, Vec<DbcMessage>> {
    many0(preceded(multispace0, parse_message))(input) // Skip whitespace before each message
}

fn parse_message(input: &str) -> IResult<&str, DbcMessage> {
    let (input, _) = tag("BO_")(input)?;
    let (input, _) = space1(input)?;
    let (input, id) = parse_u32(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, length) = parse_u8(input)?;
    let (input, _) = space1(input)?;
    let (input, transmitter) = parse_identifier(input)?;
    let (input, _) = opt(line_ending)(input)?; // FIX: Only consume newline, not leading space
    let (input, signals) = many0(parse_signal)(input)?;

    Ok((
        input,
        DbcMessage {
            id,
            name,
            length,
            transmitter,
            signals,
            is_fd: length > 8,
        },
    ))
}

fn parse_signal(input: &str) -> IResult<&str, DbcSignal> {
    let (input, _) = space1(input)?;
    let (input, _) = tag("SG_")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, (start_bit, length, byte_order, value_type)) = parse_signal_layout(input)?;
    let (input, _) = space0(input)?;
    let (input, (factor, offset)) = parse_factor_offset(input)?;
    let (input, _) = space0(input)?;
    let (input, (minimum, maximum)) = parse_min_max(input)?;
    let (input, _) = space0(input)?;
    let (input, unit) = delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)?;
    let (input, _) = space0(input)?;
    let (input, receivers) = separated_list0(char(','), parse_identifier)(input)?;
    let (input, _) = opt(line_ending)(input)?; // FIX: Only consume newline, not leading space of next signal

    Ok((
        input,
        DbcSignal {
            name,
            start_bit,
            length,
            byte_order,
            value_type,
            factor,
            offset,
            minimum,
            maximum,
            unit: unit.to_string(),
            receivers,
        },
    ))
}

fn parse_signal_layout(input: &str) -> IResult<&str, (u16, u16, ByteOrder, ValueType)> {
    let (input, start_bit) = parse_u16(input)?;
    let (input, _) = char('|')(input)?;
    let (input, length) = parse_u16(input)?;
    let (input, _) = char('@')(input)?;
    let (input, byte_order_char) = alt((char('0'), char('1')))(input)?;
    let (input, value_type_char) = alt((char('+'), char('-')))(input)?;

    let byte_order = if byte_order_char == '1' {
        ByteOrder::LittleEndian
    } else {
        ByteOrder::BigEndian
    };

    let value_type = if value_type_char == '+' {
        ValueType::Unsigned
    } else {
        ValueType::Signed
    };

    Ok((input, (start_bit, length, byte_order, value_type)))
}

fn parse_factor_offset(input: &str) -> IResult<&str, (f64, f64)> {
    let (input, _) = char('(')(input)?;
    let (input, factor) = parse_f64(input)?;
    let (input, _) = char(',')(input)?;
    let (input, offset) = parse_f64(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, (factor, offset)))
}

fn parse_min_max(input: &str) -> IResult<&str, (f64, f64)> {
    let (input, _) = char('[')(input)?;
    let (input, minimum) = parse_f64(input)?;
    let (input, _) = char('|')(input)?;
    let (input, maximum) = parse_f64(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, (minimum, maximum)))
}

fn parse_comments(input: &str) -> IResult<&str, HashMap<u32, String>> {
    let (input, comments) = many0(parse_comment)(input)?;
    Ok((input, comments.into_iter().collect()))
}

fn parse_comment(input: &str) -> IResult<&str, (u32, String)> {
    let (input, _) = tag("CM_")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("BO_")(input)?;
    let (input, _) = space1(input)?;
    let (input, id) = parse_u32(input)?;
    let (input, _) = space1(input)?;
    let (input, comment) = delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)?;
    let (input, _) = terminated(tag(";"), multispace0)(input)?;
    Ok((input, (id, comment.to_string())))
}

fn parse_value_descriptions(
    input: &str,
) -> IResult<&str, HashMap<(u32, String), HashMap<i64, String>>> {
    let (input, descriptions) = many0(parse_value_description)(input)?;
    Ok((input, descriptions.into_iter().collect()))
}

fn parse_value_description(input: &str) -> IResult<&str, ((u32, String), HashMap<i64, String>)> {
    let (input, _) = tag("VAL_")(input)?;
    let (input, _) = space1(input)?;
    let (input, msg_id) = parse_u32(input)?;
    let (input, _) = space1(input)?;
    let (input, signal_name) = parse_identifier(input)?;
    let (input, _) = space1(input)?;
    let (input, values) = many0(parse_value_pair)(input)?;
    let (input, _) = terminated(tag(";"), multispace0)(input)?;
    Ok((input, ((msg_id, signal_name), values.into_iter().collect())))
}

fn parse_value_pair(input: &str) -> IResult<&str, (i64, String)> {
    let (input, value) = parse_i64(input)?;
    let (input, _) = space1(input)?;
    let (input, description) = delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, (value, description.to_string())))
}

// Basic parsers

fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |s: &str| s.to_string(),
    )(input)
}

fn parse_u8(input: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse::<u8>())(input)
}

fn parse_u16(input: &str) -> IResult<&str, u16> {
    map_res(digit1, |s: &str| s.parse::<u16>())(input)
}

fn parse_u32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>())(input)
}

fn parse_i64(input: &str) -> IResult<&str, i64> {
    map_res(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        s.parse::<i64>()
    })(input)
}

fn parse_f64(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(tuple((char('.'), digit1))),
            opt(tuple((
                alt((char('e'), char('E'))),
                opt(alt((char('+'), char('-')))),
                digit1,
            ))),
        ))),
        |s: &str| s.parse::<f64>(),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_dbc() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_:

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert_eq!(db.version, "");
        assert_eq!(db.nodes.len(), 0);
        assert_eq!(db.messages.len(), 0);
    }

    #[test]
    fn test_parse_nodes() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1 ECU2 ECU3

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert_eq!(db.nodes.len(), 3);
        assert_eq!(db.nodes[0], "ECU1");
        assert_eq!(db.nodes[1], "ECU2");
        assert_eq!(db.nodes[2], "ECU3");
    }

    #[test]
    fn test_parse_message() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1

BO_ 100 TestMessage: 8 ECU1

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert_eq!(db.messages.len(), 1);
        assert_eq!(db.messages[0].id, 100);
        assert_eq!(db.messages[0].name, "TestMessage");
        assert_eq!(db.messages[0].length, 8);
        assert_eq!(db.messages[0].transmitter, "ECU1");
    }

    #[test]
    fn test_parse_signal() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1 ECU2

BO_ 100 TestMessage: 8 ECU1
 SG_ TestSignal : 0|8@1+ (1,0) [0|255] "" ECU2

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert_eq!(db.messages[0].signals.len(), 1);
        let signal = &db.messages[0].signals[0];
        assert_eq!(signal.name, "TestSignal");
        assert_eq!(signal.start_bit, 0);
        assert_eq!(signal.length, 8);
        assert_eq!(signal.byte_order, ByteOrder::LittleEndian);
        assert_eq!(signal.value_type, ValueType::Unsigned);
        assert_eq!(signal.factor, 1.0);
        assert_eq!(signal.offset, 0.0);
        assert_eq!(signal.minimum, 0.0);
        assert_eq!(signal.maximum, 255.0);
    }

    #[test]
    fn test_parse_signal_with_factor_offset() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1 ECU2

BO_ 200 EngineData: 8 ECU1
 SG_ EngineSpeed : 0|16@1+ (0.25,0) [0|16383.75] "rpm" ECU2
 SG_ EngineTemp : 16|8@1+ (1,-40) [-40|215] "degC" ECU2

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert_eq!(db.messages[0].signals.len(), 2);

        let speed = &db.messages[0].signals[0];
        assert_eq!(speed.name, "EngineSpeed");
        assert_eq!(speed.factor, 0.25);
        assert_eq!(speed.unit, "rpm");

        let temp = &db.messages[0].signals[1];
        assert_eq!(temp.name, "EngineTemp");
        assert_eq!(temp.offset, -40.0);
        assert_eq!(temp.unit, "degC");
    }

    #[test]
    fn test_parse_big_endian_signal() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1

BO_ 300 TestMsg: 8 ECU1
 SG_ BigEndianSignal : 0|16@0+ (1,0) [0|65535] "" ECU1

"#;
        let db = DbcParser::parse(dbc).unwrap();
        let signal = &db.messages[0].signals[0];
        assert_eq!(signal.byte_order, ByteOrder::BigEndian);
    }

    #[test]
    fn test_parse_signed_signal() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1

BO_ 400 TestMsg: 8 ECU1
 SG_ SignedSignal : 0|16@1- (1,0) [-32768|32767] "" ECU1

"#;
        let db = DbcParser::parse(dbc).unwrap();
        let signal = &db.messages[0].signals[0];
        assert_eq!(signal.value_type, ValueType::Signed);
    }

    #[test]
    fn test_find_message() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1

BO_ 100 Msg1: 8 ECU1
BO_ 200 Msg2: 8 ECU1

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert!(db.find_message(100).is_some());
        assert!(db.find_message(200).is_some());
        assert!(db.find_message(300).is_none());
    }

    #[test]
    fn test_find_signal() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1

BO_ 100 TestMsg: 8 ECU1
 SG_ Signal1 : 0|8@1+ (1,0) [0|255] "" ECU1
 SG_ Signal2 : 8|8@1+ (1,0) [0|255] "" ECU1

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert!(db.find_signal(100, "Signal1").is_some());
        assert!(db.find_signal(100, "Signal2").is_some());
        assert!(db.find_signal(100, "Signal3").is_none());
        assert!(db.find_signal(200, "Signal1").is_none());
    }

    #[test]
    fn test_signal_to_signal_def() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1

BO_ 100 TestMsg: 8 ECU1
 SG_ TestSignal : 0|16@1+ (0.1,10) [0|6553.5] "km/h" ECU1

"#;
        let db = DbcParser::parse(dbc).unwrap();
        let signal = &db.messages[0].signals[0];
        let signal_def = signal.to_signal_def();

        assert_eq!(signal_def.name, "TestSignal");
        assert_eq!(signal_def.start_bit, 0);
        assert_eq!(signal_def.bit_length, 16);
        assert_eq!(signal_def.factor, 0.1);
        assert_eq!(signal_def.offset, 10.0);
        assert_eq!(signal_def.unit, "km/h");
    }

    #[test]
    fn test_parse_multiple_messages() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1 ECU2

BO_ 100 Msg1: 8 ECU1
 SG_ Sig1 : 0|8@1+ (1,0) [0|255] "" ECU2

BO_ 200 Msg2: 4 ECU2
 SG_ Sig2 : 0|16@1+ (1,0) [0|65535] "" ECU1

BO_ 300 Msg3: 8 ECU1

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert_eq!(db.messages.len(), 3);
        assert_eq!(db.messages[0].id, 100);
        assert_eq!(db.messages[1].id, 200);
        assert_eq!(db.messages[2].id, 300);
    }

    #[test]
    fn test_parse_canfd_message() {
        let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1 ECU2

BO_ 100 FdMessage: 64 ECU1
 SG_ Signal1 : 0|32@1+ (1,0) [0|4294967295] "" ECU2
 SG_ Signal2 : 32|32@1+ (1,0) [0|4294967295] "" ECU2
 SG_ Signal3 : 256|64@1+ (1,0) [0|0] "" ECU2

"#;
        let db = DbcParser::parse(dbc).unwrap();
        assert_eq!(db.messages.len(), 1);
        let msg = &db.messages[0];
        assert_eq!(msg.length, 64);
        assert!(msg.is_fd);
        assert!(msg.requires_fd());
        assert_eq!(msg.fd_dlc(), 64);
        assert_eq!(msg.signals.len(), 3);
    }

    #[test]
    fn test_canfd_dlc_mapping() {
        // Test the DLC mapping for various lengths
        let msg8 = DbcMessage::new(100, "Msg8".to_string(), 8, "ECU".to_string());
        assert!(!msg8.is_fd);
        assert_eq!(msg8.fd_dlc(), 8);

        let msg12 = DbcMessage::new(101, "Msg12".to_string(), 12, "ECU".to_string());
        assert!(msg12.is_fd);
        assert_eq!(msg12.fd_dlc(), 12);

        let msg16 = DbcMessage::new(102, "Msg16".to_string(), 16, "ECU".to_string());
        assert!(msg16.is_fd);
        assert_eq!(msg16.fd_dlc(), 16);

        let msg20 = DbcMessage::new(103, "Msg20".to_string(), 20, "ECU".to_string());
        assert!(msg20.is_fd);
        assert_eq!(msg20.fd_dlc(), 20);

        let msg24 = DbcMessage::new(104, "Msg24".to_string(), 24, "ECU".to_string());
        assert!(msg24.is_fd);
        assert_eq!(msg24.fd_dlc(), 24);

        let msg32 = DbcMessage::new(105, "Msg32".to_string(), 32, "ECU".to_string());
        assert!(msg32.is_fd);
        assert_eq!(msg32.fd_dlc(), 32);

        let msg48 = DbcMessage::new(106, "Msg48".to_string(), 48, "ECU".to_string());
        assert!(msg48.is_fd);
        assert_eq!(msg48.fd_dlc(), 48);

        let msg64 = DbcMessage::new(107, "Msg64".to_string(), 64, "ECU".to_string());
        assert!(msg64.is_fd);
        assert_eq!(msg64.fd_dlc(), 64);
    }

    #[test]
    fn test_canfd_intermediate_lengths() {
        // Test intermediate lengths that round up to valid FD DLC
        let msg10 = DbcMessage::new(100, "Msg10".to_string(), 10, "ECU".to_string());
        assert!(msg10.is_fd);
        assert_eq!(msg10.fd_dlc(), 12); // 10 rounds up to 12

        let msg15 = DbcMessage::new(101, "Msg15".to_string(), 15, "ECU".to_string());
        assert!(msg15.is_fd);
        assert_eq!(msg15.fd_dlc(), 16); // 15 rounds up to 16

        let msg30 = DbcMessage::new(102, "Msg30".to_string(), 30, "ECU".to_string());
        assert!(msg30.is_fd);
        assert_eq!(msg30.fd_dlc(), 32); // 30 rounds up to 32

        let msg40 = DbcMessage::new(103, "Msg40".to_string(), 40, "ECU".to_string());
        assert!(msg40.is_fd);
        assert_eq!(msg40.fd_dlc(), 48); // 40 rounds up to 48

        let msg50 = DbcMessage::new(104, "Msg50".to_string(), 50, "ECU".to_string());
        assert!(msg50.is_fd);
        assert_eq!(msg50.fd_dlc(), 64); // 50 rounds up to 64
    }
}
