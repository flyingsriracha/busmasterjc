//! Signal types for CAN message interpretation
//!
//! This module provides types for defining and extracting signals from CAN frames.
//! Signals are the meaningful data values encoded within CAN message payloads.
//!
//! # Signal Extraction
//!
//! CAN signals are defined by:
//! - Start bit position within the frame data
//! - Bit length
//! - Byte order (little-endian/Intel or big-endian/Motorola)
//! - Value type (unsigned, signed, float)
//! - Factor and offset for physical value conversion
//!
//! # Example
//!
//! ```
//! use busmaster_core::{SignalDef, ByteOrder, ValueType};
//!
//! // Define an engine speed signal
//! let engine_speed = SignalDef::new("EngineSpeed", 8, 16)
//!     .with_byte_order(ByteOrder::LittleEndian)
//!     .with_value_type(ValueType::Unsigned)
//!     .with_factor_offset(0.25, 0.0)
//!     .with_range(0.0, 16383.75)
//!     .with_unit("rpm");
//!
//! assert_eq!(engine_speed.name, "EngineSpeed");
//! assert_eq!(engine_speed.factor, 0.25);
//! ```

use serde::{Deserialize, Serialize};

/// Byte order for signal extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByteOrder {
    /// Little-endian (Intel)
    LittleEndian,
    /// Big-endian (Motorola)
    BigEndian,
}

/// Value type for signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    /// Unsigned integer
    Unsigned,
    /// Signed integer
    Signed,
    /// IEEE 754 float (32-bit)
    Float32,
    /// IEEE 754 double (64-bit)
    Float64,
}

/// Signal definition from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDef {
    /// Signal name
    pub name: String,
    /// Start bit position
    pub start_bit: u16,
    /// Bit length
    pub bit_length: u16,
    /// Byte order
    pub byte_order: ByteOrder,
    /// Value type
    pub value_type: ValueType,
    /// Scale factor
    pub factor: f64,
    /// Offset
    pub offset: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Unit string
    pub unit: String,
    /// Comment/description
    pub comment: Option<String>,
}

impl SignalDef {
    /// Create a new signal definition
    #[must_use]
    pub fn new(name: &str, start_bit: u16, bit_length: u16) -> Self {
        Self {
            name: name.to_string(),
            start_bit,
            bit_length,
            byte_order: ByteOrder::LittleEndian,
            value_type: ValueType::Unsigned,
            factor: 1.0,
            offset: 0.0,
            min: 0.0,
            max: 0.0,
            unit: String::new(),
            comment: None,
        }
    }

    /// Set byte order
    #[must_use]
    pub fn with_byte_order(mut self, order: ByteOrder) -> Self {
        self.byte_order = order;
        self
    }

    /// Set value type
    #[must_use]
    pub fn with_value_type(mut self, vtype: ValueType) -> Self {
        self.value_type = vtype;
        self
    }

    /// Set factor and offset
    #[must_use]
    pub fn with_factor_offset(mut self, factor: f64, offset: f64) -> Self {
        self.factor = factor;
        self.offset = offset;
        self
    }

    /// Set min/max range
    #[must_use]
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set unit
    #[must_use]
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// Set factor
    #[must_use]
    pub fn with_factor(mut self, factor: f64) -> Self {
        self.factor = factor;
        self
    }

    /// Set offset
    #[must_use]
    pub fn with_offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    /// Extract signal value from CAN frame data
    ///
    /// Extracts the signal value from the provided data bytes according to
    /// the signal definition (start bit, length, byte order, etc.).
    ///
    /// # Arguments
    ///
    /// * `data` - CAN frame data bytes
    ///
    /// # Returns
    ///
    /// A `SignalValue` containing the raw and physical values, or an error
    /// if extraction fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The signal extends beyond the data length
    /// - The signal definition is invalid
    ///
    /// # Example
    ///
    /// ```
    /// use busmaster_core::{SignalDef, ByteOrder, ValueType};
    ///
    /// let signal = SignalDef::new("Speed", 0, 16)
    ///     .with_byte_order(ByteOrder::LittleEndian)
    ///     .with_value_type(ValueType::Unsigned)
    ///     .with_factor_offset(0.1, 0.0);
    ///
    /// let data = [0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let value = signal.extract(&data).unwrap();
    /// assert_eq!(value.raw_value, 10000);
    /// assert_eq!(value.physical_value, 1000.0); // 10000 * 0.1
    /// ```
    pub fn extract(&self, data: &[u8]) -> crate::Result<SignalValue> {
        // Validate signal fits in data
        let required_bytes = (self.start_bit + self.bit_length).div_ceil(8) as usize;
        if required_bytes > data.len() {
            return Err(crate::BusmasterError::InvalidSignal {
                message: format!(
                    "Signal '{}' extends beyond data length (needs {} bytes, have {})",
                    self.name,
                    required_bytes,
                    data.len()
                ),
            });
        }

        // Extract raw value based on byte order
        let raw_value = match self.byte_order {
            ByteOrder::LittleEndian => self.extract_little_endian(data)?,
            ByteOrder::BigEndian => self.extract_big_endian(data)?,
        };

        // Apply sign extension if needed
        let signed_value = if self.value_type == ValueType::Signed {
            self.sign_extend(raw_value)
        } else {
            #[allow(clippy::cast_possible_wrap)]
            {
                raw_value as i64
            }
        };

        // Apply factor and offset
        #[allow(clippy::cast_precision_loss)]
        let physical_value = (signed_value as f64) * self.factor + self.offset;

        Ok(SignalValue::new(
            &self.name,
            signed_value,
            physical_value,
            &self.unit,
        ))
    }

    /// Extract signal using little-endian (Intel) byte order
    fn extract_little_endian(&self, data: &[u8]) -> crate::Result<u64> {
        let mut value: u64 = 0;
        let start_byte = (self.start_bit / 8) as usize;
        let start_bit_in_byte = (self.start_bit % 8) as usize;

        let mut bits_remaining = self.bit_length as usize;
        let mut current_byte = start_byte;
        let mut current_bit = start_bit_in_byte;

        while bits_remaining > 0 {
            if current_byte >= data.len() {
                return Err(crate::BusmasterError::InvalidSignal {
                    message: format!("Signal '{}' extends beyond data", self.name),
                });
            }

            let bits_in_this_byte = (8 - current_bit).min(bits_remaining);
            let mask = ((1u64 << bits_in_this_byte) - 1) << current_bit;
            let byte_value = (u64::from(data[current_byte]) & mask) >> current_bit;

            let shift = (self.bit_length as usize) - bits_remaining;
            value |= byte_value << shift;

            bits_remaining -= bits_in_this_byte;
            current_byte += 1;
            current_bit = 0;
        }

        Ok(value)
    }

    /// Extract signal using big-endian (Motorola) byte order
    fn extract_big_endian(&self, data: &[u8]) -> crate::Result<u64> {
        let mut value: u64 = 0;
        let start_byte = (self.start_bit / 8) as usize;
        let start_bit_in_byte = 7 - (self.start_bit % 8) as usize;

        let mut bits_remaining = self.bit_length as usize;
        let mut current_byte = start_byte;
        let mut current_bit = start_bit_in_byte;

        while bits_remaining > 0 {
            if current_byte >= data.len() {
                return Err(crate::BusmasterError::InvalidSignal {
                    message: format!("Signal '{}' extends beyond data", self.name),
                });
            }

            let bits_in_this_byte = (current_bit + 1).min(bits_remaining);
            let shift_in_byte = current_bit + 1 - bits_in_this_byte;
            let mask = ((1u64 << bits_in_this_byte) - 1) << shift_in_byte;
            let byte_value = (u64::from(data[current_byte]) & mask) >> shift_in_byte;

            let shift = bits_remaining - bits_in_this_byte;
            value |= byte_value << shift;

            bits_remaining -= bits_in_this_byte;
            current_byte += 1;
            current_bit = 7;
        }

        Ok(value)
    }

    /// Apply sign extension for signed values
    fn sign_extend(&self, value: u64) -> i64 {
        let sign_bit = 1u64 << (self.bit_length - 1);
        if value & sign_bit != 0 {
            // Negative value - extend sign bits
            let mask = !((1u64 << self.bit_length) - 1);
            #[allow(clippy::cast_possible_wrap)]
            {
                (value | mask) as i64
            }
        } else {
            // Positive value
            #[allow(clippy::cast_possible_wrap)]
            {
                value as i64
            }
        }
    }
}

/// Extracted signal value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalValue {
    /// Signal name
    pub name: String,
    /// Raw value (before factor/offset)
    pub raw_value: i64,
    /// Physical value (after factor/offset)
    pub physical_value: f64,
    /// Unit string
    pub unit: String,
}

impl SignalValue {
    /// Create a new signal value
    #[must_use]
    pub fn new(name: &str, raw: i64, physical: f64, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            raw_value: raw,
            physical_value: physical,
            unit: unit.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_def_builder() {
        let sig = SignalDef::new("EngineSpeed", 8, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned)
            .with_factor_offset(0.25, 0.0)
            .with_range(0.0, 16383.75)
            .with_unit("rpm");

        assert_eq!(sig.name, "EngineSpeed");
        assert_eq!(sig.start_bit, 8);
        assert_eq!(sig.bit_length, 16);
        assert_eq!(sig.factor, 0.25);
        assert_eq!(sig.unit, "rpm");
    }

    #[test]
    fn test_signal_value() {
        let val = SignalValue::new("Speed", 1000, 250.0, "km/h");
        assert_eq!(val.raw_value, 1000);
        assert_eq!(val.physical_value, 250.0);
    }

    #[test]
    fn test_byte_order_variants() {
        assert_ne!(ByteOrder::LittleEndian, ByteOrder::BigEndian);
    }

    #[test]
    fn test_value_type_variants() {
        let types = [
            ValueType::Unsigned,
            ValueType::Signed,
            ValueType::Float32,
            ValueType::Float64,
        ];
        // All variants should be distinct
        for (i, t1) in types.iter().enumerate() {
            for (j, t2) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(t1, t2);
                }
            }
        }
    }

    #[test]
    fn test_signal_def_defaults() {
        let sig = SignalDef::new("Test", 0, 8);
        assert_eq!(sig.byte_order, ByteOrder::LittleEndian);
        assert_eq!(sig.value_type, ValueType::Unsigned);
        assert_eq!(sig.factor, 1.0);
        assert_eq!(sig.offset, 0.0);
        assert!(sig.unit.is_empty());
        assert!(sig.comment.is_none());
    }

    #[test]
    fn test_signal_def_with_comment() {
        let mut sig = SignalDef::new("Test", 0, 8);
        sig.comment = Some("Test comment".to_string());
        assert_eq!(sig.comment, Some("Test comment".to_string()));
    }

    #[test]
    fn test_signal_def_clone() {
        let sig1 = SignalDef::new("Test", 0, 8).with_factor_offset(2.0, 10.0);
        let sig2 = sig1.clone();
        assert_eq!(sig1.name, sig2.name);
        assert_eq!(sig1.factor, sig2.factor);
        assert_eq!(sig1.offset, sig2.offset);
    }

    #[test]
    fn test_signal_value_clone() {
        let val1 = SignalValue::new("Test", 100, 50.0, "unit");
        let val2 = val1.clone();
        assert_eq!(val1.raw_value, val2.raw_value);
        assert_eq!(val1.physical_value, val2.physical_value);
    }

    // Signal extraction tests

    #[test]
    fn test_extract_8bit_unsigned_little_endian() {
        let sig = SignalDef::new("Test", 0, 8)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned);
        let data = [0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, 0x42);
        assert_eq!(val.physical_value, 66.0);
    }

    #[test]
    fn test_extract_16bit_unsigned_little_endian() {
        let sig = SignalDef::new("Test", 0, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned);
        let data = [0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, 10000);
    }

    #[test]
    fn test_extract_with_factor_offset() {
        let sig = SignalDef::new("Speed", 0, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned)
            .with_factor_offset(0.1, 0.0);
        let data = [0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, 10000);
        assert_eq!(val.physical_value, 1000.0);
    }

    #[test]
    fn test_extract_with_offset() {
        let sig = SignalDef::new("Temp", 0, 8)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned)
            .with_factor_offset(1.0, -40.0);
        let data = [100, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, 100);
        assert_eq!(val.physical_value, 60.0); // 100 - 40
    }

    #[test]
    fn test_extract_signed_positive() {
        let sig = SignalDef::new("Test", 0, 8)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Signed);
        let data = [0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, 127);
    }

    #[test]
    fn test_extract_signed_negative() {
        let sig = SignalDef::new("Test", 0, 8)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Signed);
        let data = [0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, -1);
    }

    #[test]
    fn test_extract_signed_16bit_negative() {
        let sig = SignalDef::new("Test", 0, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Signed);
        let data = [0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, -1);
    }

    #[test]
    fn test_extract_big_endian_8bit() {
        let sig = SignalDef::new("Test", 0, 8)
            .with_byte_order(ByteOrder::BigEndian)
            .with_value_type(ValueType::Unsigned);
        let data = [0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, 0x42);
    }

    #[test]
    fn test_extract_big_endian_16bit() {
        let sig = SignalDef::new("Test", 0, 16)
            .with_byte_order(ByteOrder::BigEndian)
            .with_value_type(ValueType::Unsigned);
        let data = [0x27, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        assert_eq!(val.raw_value, 10000);
    }

    #[test]
    fn test_extract_cross_byte_boundary() {
        let sig = SignalDef::new("Test", 4, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned);
        let data = [0x00, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
        let val = sig.extract(&data).unwrap();
        // Bits 4-19: should extract 0x0201 >> 4 = 0x0020 | (0x01 << 12) = 0x1020
        assert!(val.raw_value > 0);
    }

    #[test]
    fn test_extract_signal_too_long() {
        let sig = SignalDef::new("Test", 0, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned);
        let data = [0x42]; // Only 1 byte, need 2
        let result = sig.extract(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_signal_beyond_data() {
        let sig = SignalDef::new("Test", 56, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned);
        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = sig.extract(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_multiple_signals() {
        let data = [0x10, 0x27, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00];

        let sig1 = SignalDef::new("Speed", 0, 16)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned)
            .with_factor_offset(0.1, 0.0)
            .with_unit("km/h");

        let sig2 = SignalDef::new("Temp", 16, 8)
            .with_byte_order(ByteOrder::LittleEndian)
            .with_value_type(ValueType::Unsigned)
            .with_factor_offset(1.0, -40.0)
            .with_unit("degC");

        let val1 = sig1.extract(&data).unwrap();
        assert_eq!(val1.raw_value, 10000);
        assert_eq!(val1.physical_value, 1000.0);
        assert_eq!(val1.unit, "km/h");

        let val2 = sig2.extract(&data).unwrap();
        assert_eq!(val2.raw_value, 100);
        assert_eq!(val2.physical_value, 60.0);
        assert_eq!(val2.unit, "degC");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// **Validates: Requirements 1.2.4, 1.2.9**
        /// Property: SignalDef can be created with any valid bit positions
        #[test]
        fn prop_signal_def_any_bit_position(
            start_bit in 0u16..512,
            bit_length in 1u16..64
        ) {
            let sig = SignalDef::new("Test", start_bit, bit_length);
            prop_assert_eq!(sig.start_bit, start_bit);
            prop_assert_eq!(sig.bit_length, bit_length);
        }

        /// **Validates: Requirements 1.2.4, 1.2.9**
        /// Property: Factor and offset can be any finite f64 values
        #[test]
        fn prop_signal_factor_offset_any_finite(
            factor in prop::num::f64::NORMAL,
            offset in prop::num::f64::NORMAL
        ) {
            let sig = SignalDef::new("Test", 0, 8)
                .with_factor_offset(factor, offset);
            prop_assert_eq!(sig.factor, factor);
            prop_assert_eq!(sig.offset, offset);
        }

        /// **Validates: Requirements 1.2.4, 1.2.9**
        /// Property: SignalValue preserves raw and physical values
        #[test]
        fn prop_signal_value_preserves_values(
            raw in any::<i64>(),
            physical in prop::num::f64::NORMAL
        ) {
            let val = SignalValue::new("Test", raw, physical, "unit");
            prop_assert_eq!(val.raw_value, raw);
            prop_assert_eq!(val.physical_value, physical);
        }

        /// **Validates: Requirements 1.2.7, 1.2.9**
        /// Property: SignalDef serialization roundtrip preserves data
        #[test]
        fn prop_signal_def_serde_roundtrip(
            start_bit in 0u16..512,
            bit_length in 1u16..64,
            factor in prop::num::f64::NORMAL,
            offset in prop::num::f64::NORMAL
        ) {
            let sig = SignalDef::new("TestSignal", start_bit, bit_length)
                .with_factor_offset(factor, offset);
            let json = serde_json::to_string(&sig).unwrap();
            let decoded: SignalDef = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(sig.name, decoded.name);
            prop_assert_eq!(sig.start_bit, decoded.start_bit);
            prop_assert_eq!(sig.bit_length, decoded.bit_length);
            // Use relative epsilon for floating point comparison
            let factor_diff = (sig.factor - decoded.factor).abs();
            let factor_epsilon = sig.factor.abs() * 1e-10 + 1e-15;
            prop_assert!(factor_diff <= factor_epsilon,
                "factor mismatch: {} vs {}", sig.factor, decoded.factor);
            let offset_diff = (sig.offset - decoded.offset).abs();
            let offset_epsilon = sig.offset.abs() * 1e-10 + 1e-15;
            prop_assert!(offset_diff <= offset_epsilon,
                "offset mismatch: {} vs {}", sig.offset, decoded.offset);
        }

        /// **Validates: Requirements 1.2.7, 1.2.9**
        /// Property: SignalValue serialization roundtrip preserves data
        #[test]
        fn prop_signal_value_serde_roundtrip(
            raw in any::<i64>(),
            physical in prop::num::f64::NORMAL
        ) {
            let val = SignalValue::new("TestSignal", raw, physical, "unit");
            let json = serde_json::to_string(&val).unwrap();
            let decoded: SignalValue = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(val.raw_value, decoded.raw_value);
            // Use relative epsilon for floating point comparison
            let diff = (val.physical_value - decoded.physical_value).abs();
            let epsilon = val.physical_value.abs() * 1e-10 + 1e-15;
            prop_assert!(diff <= epsilon,
                "physical_value mismatch: {} vs {}", val.physical_value, decoded.physical_value);
            prop_assert_eq!(val.name, decoded.name);
            prop_assert_eq!(val.unit, decoded.unit);
        }

        /// **Validates: Requirements 1.2.5, 1.2.9**
        /// Property: ByteOrder serialization roundtrip
        #[test]
        fn prop_byte_order_serde_roundtrip(is_little in any::<bool>()) {
            let order = if is_little { ByteOrder::LittleEndian } else { ByteOrder::BigEndian };
            let json = serde_json::to_string(&order).unwrap();
            let decoded: ByteOrder = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(order, decoded);
        }

        /// **Validates: Requirements 1.2.5, 1.2.9**
        /// Property: ValueType serialization roundtrip
        #[test]
        fn prop_value_type_serde_roundtrip(idx in 0usize..4) {
            let types = [
                ValueType::Unsigned,
                ValueType::Signed,
                ValueType::Float32,
                ValueType::Float64,
            ];
            let vtype = types[idx];
            let json = serde_json::to_string(&vtype).unwrap();
            let decoded: ValueType = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(vtype, decoded);
        }

        /// **Validates: Requirements 2.2.1, 2.2.2**
        /// Property: Extracting an 8-bit unsigned signal always succeeds with valid data
        #[test]
        fn prop_extract_8bit_unsigned(value in 0u8..=255) {
            let sig = SignalDef::new("Test", 0, 8)
                .with_byte_order(ByteOrder::LittleEndian)
                .with_value_type(ValueType::Unsigned);
            let data = [value, 0, 0, 0, 0, 0, 0, 0];
            let result = sig.extract(&data);
            prop_assert!(result.is_ok());
            let val = result.unwrap();
            prop_assert_eq!(val.raw_value, value as i64);
        }

        /// **Validates: Requirements 2.2.1, 2.2.5**
        /// Property: Factor and offset are correctly applied
        #[test]
        fn prop_extract_factor_offset(
            raw_value in 0u16..1000,
            factor in 0.1f64..10.0,
            offset in -100.0f64..100.0
        ) {
            let sig = SignalDef::new("Test", 0, 16)
                .with_byte_order(ByteOrder::LittleEndian)
                .with_value_type(ValueType::Unsigned)
                .with_factor_offset(factor, offset);

            let data = [
                (raw_value & 0xFF) as u8,
                ((raw_value >> 8) & 0xFF) as u8,
                0, 0, 0, 0, 0, 0
            ];

            let val = sig.extract(&data).unwrap();
            let expected = (raw_value as f64) * factor + offset;
            prop_assert!((val.physical_value - expected).abs() < 0.0001);
        }

        /// **Validates: Requirements 2.2.3, 2.2.4**
        /// Property: Signed values are correctly sign-extended
        #[test]
        fn prop_extract_signed_8bit(value in -128i8..=127) {
            let sig = SignalDef::new("Test", 0, 8)
                .with_byte_order(ByteOrder::LittleEndian)
                .with_value_type(ValueType::Signed);
            let data = [value as u8, 0, 0, 0, 0, 0, 0, 0];
            let val = sig.extract(&data).unwrap();
            prop_assert_eq!(val.raw_value, value as i64);
        }

        /// **Validates: Requirements 2.2.2, 2.2.3**
        /// Property: Little-endian and big-endian extraction are consistent for single bytes
        #[test]
        fn prop_extract_byte_order_consistent_single_byte(value in 0u8..=255) {
            let sig_le = SignalDef::new("Test", 0, 8)
                .with_byte_order(ByteOrder::LittleEndian)
                .with_value_type(ValueType::Unsigned);
            let sig_be = SignalDef::new("Test", 0, 8)
                .with_byte_order(ByteOrder::BigEndian)
                .with_value_type(ValueType::Unsigned);

            let data = [value, 0, 0, 0, 0, 0, 0, 0];
            let val_le = sig_le.extract(&data).unwrap();
            let val_be = sig_be.extract(&data).unwrap();
            prop_assert_eq!(val_le.raw_value, val_be.raw_value);
        }
    }
}
