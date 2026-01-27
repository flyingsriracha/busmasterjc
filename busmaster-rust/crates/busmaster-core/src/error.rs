//! Error types for BUSMASTER
//!
//! This module provides the unified error type [`BusmasterError`] used throughout
//! the BUSMASTER Rust implementation. All errors are designed to be informative
//! and actionable.
//!
//! # Example
//!
//! ```
//! use busmaster_core::{BusmasterError, Result};
//!
//! fn validate_id(id: u32) -> Result<()> {
//!     if id > 0x7FF {
//!         return Err(BusmasterError::InvalidCanId { id, max: 0x7FF });
//!     }
//!     Ok(())
//! }
//! ```

use thiserror::Error;

/// Result type alias using [`BusmasterError`]
pub type Result<T> = std::result::Result<T, BusmasterError>;

/// Main error type for BUSMASTER operations
///
/// This enum covers all error conditions that can occur in BUSMASTER,
/// from CAN frame validation to hardware communication errors.
#[derive(Error, Debug)]
pub enum BusmasterError {
    /// Invalid CAN frame ID
    ///
    /// Returned when a CAN ID exceeds the maximum allowed value
    /// (0x7FF for standard, 0x1FFFFFFF for extended).
    #[error("Invalid CAN ID: 0x{id:X} (max: 0x{max:X})")]
    InvalidCanId {
        /// The invalid ID value
        id: u32,
        /// Maximum allowed value
        max: u32,
    },

    /// Invalid data length
    ///
    /// Returned when frame data exceeds the maximum allowed length
    /// (8 bytes for CAN 2.0, 64 bytes for CAN FD).
    #[error("Invalid data length: {len} bytes (max: {max})")]
    InvalidDataLength {
        /// The invalid length
        len: usize,
        /// Maximum allowed length
        max: usize,
    },

    /// Invalid signal extraction parameters
    #[error("Invalid signal: {message}")]
    InvalidSignal {
        /// Error message describing the issue
        message: String,
    },

    /// Protocol error during communication
    #[error("Protocol error: {message}")]
    Protocol {
        /// Error message
        message: String,
    },

    /// Hardware/driver error with vendor information
    #[error("Hardware error ({vendor}): {message}")]
    Hardware {
        /// Error message
        message: String,
        /// Hardware vendor name
        vendor: String,
    },

    /// I/O error (file operations, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Database parse error with line information
    #[error("Parse error at line {line}: {message}")]
    DatabaseParse {
        /// Error message
        message: String,
        /// Line number where error occurred
        line: usize,
    },

    /// General parse error
    #[error("Parse error: {message}")]
    Parse {
        /// Error message
        message: String,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Config {
        /// Error message
        message: String,
    },

    /// Channel not found
    #[error("Channel {channel} not found")]
    ChannelNotFound {
        /// Channel number that was not found
        channel: u8,
    },

    /// Operation timeout
    #[error("Timeout after {timeout_ms}ms")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u32,
    },

    /// Buffer full condition
    #[error("Buffer full (capacity: {capacity})")]
    BufferFull {
        /// Buffer capacity
        capacity: usize,
    },

    /// Network error (for Ethernet protocols)
    #[error("Network error: {message}")]
    Network {
        /// Error message
        message: String,
    },

    /// AI/ML integration error
    #[error("AI error: {message}")]
    Ai {
        /// Error message
        message: String,
    },
}

impl BusmasterError {
    /// Create a hardware error with vendor information
    pub fn hardware(vendor: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Hardware {
            vendor: vendor.into(),
            message: message.into(),
        }
    }

    /// Create a protocol error
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    /// Create a parse error
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    /// Create a database parse error with line number
    pub fn database_parse(line: usize, message: impl Into<String>) -> Self {
        Self::DatabaseParse {
            line,
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a timeout error
    #[must_use]
    pub fn timeout(timeout_ms: u32) -> Self {
        Self::Timeout { timeout_ms }
    }

    /// Check if this is a recoverable error
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. } | Self::BufferFull { .. } | Self::Network { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BusmasterError::InvalidCanId {
            id: 0x800,
            max: 0x7FF,
        };
        assert!(err.to_string().contains("0x800"));
        assert!(err.to_string().contains("0x7FF"));
    }

    #[test]
    fn test_hardware_error_helper() {
        let err = BusmasterError::hardware("PEAK", "Device not found");
        match err {
            BusmasterError::Hardware { vendor, message } => {
                assert_eq!(vendor, "PEAK");
                assert_eq!(message, "Device not found");
            },
            _ => panic!("Expected Hardware error"),
        }
    }

    #[test]
    fn test_is_recoverable() {
        assert!(BusmasterError::timeout(1000).is_recoverable());
        assert!(BusmasterError::BufferFull { capacity: 100 }.is_recoverable());
        assert!(!BusmasterError::InvalidCanId { id: 0, max: 0 }.is_recoverable());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: BusmasterError = io_err.into();
        assert!(matches!(err, BusmasterError::Io(_)));
    }
}
