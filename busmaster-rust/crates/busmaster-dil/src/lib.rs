//! BUSMASTER Driver Interface Layer (DIL)
//!
//! This crate defines the traits and types for hardware driver abstraction,
//! allowing different CAN hardware vendors to be supported through a common
//! interface. The DIL provides a vendor-neutral API for CAN communication.
//!
//! # Architecture
//!
//! The DIL consists of three main components:
//!
//! 1. **[`CanDriver`]** - The main trait that hardware drivers implement
//! 2. **[`DriverFactory`]** - Factory trait for creating driver instances
//! 3. **Configuration Types** - [`ChannelConfig`], [`DeviceInfo`], etc.
//!
//! # Example
//!
//! ```
//! use busmaster_core::CanFrame;
//! use busmaster_dil::{CanDriver, ChannelConfig, ChannelStatus};
//!
//! fn use_driver(driver: &mut dyn CanDriver) -> busmaster_core::Result<()> {
//!     // List available devices
//!     let devices = driver.list_devices()?;
//!     println!("Found {} devices", devices.len());
//!
//!     // Configure and open a channel
//!     let config = ChannelConfig::new(500_000); // 500 kbps
//!     driver.open_channel(0, &config)?;
//!
//!     // Send a frame
//!     let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03, 0x04])?;
//!     driver.send(0, &frame)?;
//!
//!     // Receive frames
//!     if let Some(received) = driver.receive(0)? {
//!         println!("Received frame: {:?}", received);
//!     }
//!
//!     // Check status
//!     let status = driver.channel_status(0)?;
//!     assert_eq!(status, ChannelStatus::Active);
//!
//!     // Close channel
//!     driver.close_channel(0)?;
//!     Ok(())
//! }
//! ```
//!
//! # Implementing a Driver
//!
//! To implement a new hardware driver:
//!
//! ```
//! use busmaster_core::{CanFrame, Result, BusmasterError};
//! use busmaster_dil::{CanDriver, ChannelConfig, ChannelStatus, DeviceInfo};
//!
//! struct MyDriver {
//!     // Driver state
//! }
//!
//! impl CanDriver for MyDriver {
//!     fn name(&self) -> &str {
//!         "MyDriver"
//!     }
//!
//!     fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
//!         // Enumerate hardware devices
//!         Ok(vec![])
//!     }
//!
//!     fn open_channel(&mut self, channel: u8, config: &ChannelConfig) -> Result<()> {
//!         // Open and configure the channel
//!         Ok(())
//!     }
//!
//!     fn close_channel(&mut self, channel: u8) -> Result<()> {
//!         // Close the channel
//!         Ok(())
//!     }
//!
//!     fn send(&mut self, channel: u8, frame: &CanFrame) -> Result<()> {
//!         // Transmit the frame
//!         Ok(())
//!     }
//!
//!     fn receive(&mut self, channel: u8) -> Result<Option<CanFrame>> {
//!         // Receive a frame (non-blocking)
//!         Ok(None)
//!     }
//!
//!     fn channel_status(&self, channel: u8) -> Result<ChannelStatus> {
//!         // Get channel status
//!         Ok(ChannelStatus::Closed)
//!     }
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]

use busmaster_core::{CanFrame, Result};
use serde::{Deserialize, Serialize};

/// CAN driver trait - implemented by hardware drivers
///
/// This trait defines the interface that all CAN hardware drivers must implement.
/// It provides methods for device enumeration, channel management, and frame
/// transmission/reception.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow use across threads.
///
/// # Example
///
/// See the crate-level documentation for a complete example.
pub trait CanDriver: Send + Sync {
    /// Get the driver name
    ///
    /// Returns a human-readable name for this driver (e.g., "PEAK USB", "Vector XL").
    fn name(&self) -> &str;

    /// List available hardware devices
    ///
    /// Enumerates all connected devices that this driver can control.
    ///
    /// # Returns
    ///
    /// A vector of [`DeviceInfo`] describing each available device.
    ///
    /// # Errors
    ///
    /// Returns an error if device enumeration fails (e.g., USB communication error).
    fn list_devices(&self) -> Result<Vec<DeviceInfo>>;

    /// Open a CAN channel with the specified configuration
    ///
    /// Opens and configures a CAN channel for communication. The channel must
    /// be opened before sending or receiving frames.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number (0-based)
    /// * `config` - Channel configuration (baudrate, mode, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The channel is already open
    /// - The channel number is invalid
    /// - The hardware configuration fails
    fn open_channel(&mut self, channel: u8, config: &ChannelConfig) -> Result<()>;

    /// Close a CAN channel
    ///
    /// Closes a previously opened channel and releases associated resources.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number to close
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is not open or the operation fails.
    fn close_channel(&mut self, channel: u8) -> Result<()>;

    /// Send a CAN frame on the specified channel
    ///
    /// Transmits a CAN frame. This operation may block briefly if the transmit
    /// buffer is full.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number to transmit on
    /// * `frame` - The CAN frame to send
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The channel is not open
    /// - The transmit buffer is full
    /// - The hardware reports an error
    fn send(&mut self, channel: u8, frame: &CanFrame) -> Result<()>;

    /// Receive a CAN frame from the specified channel (non-blocking)
    ///
    /// Attempts to receive a frame from the channel's receive buffer.
    /// Returns `None` if no frame is available.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number to receive from
    ///
    /// # Returns
    ///
    /// * `Ok(Some(frame))` - A frame was received
    /// * `Ok(None)` - No frame available
    /// * `Err(e)` - An error occurred
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is not open or a hardware error occurs.
    fn receive(&mut self, channel: u8) -> Result<Option<CanFrame>>;

    /// Get the current status of a channel
    ///
    /// Returns the operational status of the specified channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number to query
    ///
    /// # Returns
    ///
    /// The current [`ChannelStatus`]
    ///
    /// # Errors
    ///
    /// Returns an error if the channel number is invalid.
    fn channel_status(&self, channel: u8) -> Result<ChannelStatus>;

    /// Reset a channel (optional)
    ///
    /// Resets the channel, clearing error states and buffers. Not all drivers
    /// support this operation.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number to reset
    ///
    /// # Errors
    ///
    /// Returns an error if the operation is not supported or fails.
    fn reset_channel(&mut self, channel: u8) -> Result<()> {
        // Default implementation: close and reopen
        let _ = self.close_channel(channel);
        Ok(())
    }

    /// Get driver version information (optional)
    ///
    /// Returns version information for the driver and/or hardware firmware.
    fn version(&self) -> Option<DriverVersion> {
        None
    }
}

/// Device information
///
/// Describes a CAN hardware device discovered by a driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device name (e.g., "PCAN-USB", "VN1610")
    pub name: String,
    /// Serial number or unique identifier
    pub serial: String,
    /// Number of CAN channels on this device
    pub channel_count: u8,
    /// Hardware version (optional)
    pub hardware_version: Option<String>,
    /// Firmware version (optional)
    pub firmware_version: Option<String>,
}

impl DeviceInfo {
    /// Create a new device info
    pub fn new(name: impl Into<String>, serial: impl Into<String>, channel_count: u8) -> Self {
        Self {
            name: name.into(),
            serial: serial.into(),
            channel_count,
            hardware_version: None,
            firmware_version: None,
        }
    }

    /// Set hardware version
    pub fn with_hardware_version(mut self, version: impl Into<String>) -> Self {
        self.hardware_version = Some(version.into());
        self
    }

    /// Set firmware version
    pub fn with_firmware_version(mut self, version: impl Into<String>) -> Self {
        self.firmware_version = Some(version.into());
        self
    }
}

/// Channel configuration
///
/// Specifies how a CAN channel should be configured when opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Baudrate in bits per second (e.g., 500_000 for 500 kbps)
    pub baudrate: u32,
    /// Enable silent mode (listen-only, no ACK)
    pub silent: bool,
    /// Enable loopback mode (for testing)
    pub loopback: bool,
    /// Enable CAN FD mode (if supported)
    pub fd_enabled: bool,
    /// CAN FD data baudrate (if fd_enabled is true)
    pub fd_baudrate: Option<u32>,
}

impl ChannelConfig {
    /// Create a new channel configuration with the specified baudrate
    ///
    /// # Arguments
    ///
    /// * `baudrate` - Baudrate in bits per second
    ///
    /// # Example
    ///
    /// ```
    /// use busmaster_dil::ChannelConfig;
    ///
    /// let config = ChannelConfig::new(500_000); // 500 kbps
    /// ```
    pub fn new(baudrate: u32) -> Self {
        Self {
            baudrate,
            silent: false,
            loopback: false,
            fd_enabled: false,
            fd_baudrate: None,
        }
    }

    /// Enable silent (listen-only) mode
    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// Enable loopback mode
    pub fn with_loopback(mut self, loopback: bool) -> Self {
        self.loopback = loopback;
        self
    }

    /// Enable CAN FD mode with specified data baudrate
    pub fn with_fd(mut self, fd_baudrate: u32) -> Self {
        self.fd_enabled = true;
        self.fd_baudrate = Some(fd_baudrate);
        self
    }

    /// Common baudrate: 125 kbps
    pub const BAUDRATE_125K: u32 = 125_000;
    /// Common baudrate: 250 kbps
    pub const BAUDRATE_250K: u32 = 250_000;
    /// Common baudrate: 500 kbps
    pub const BAUDRATE_500K: u32 = 500_000;
    /// Common baudrate: 1 Mbps
    pub const BAUDRATE_1M: u32 = 1_000_000;
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self::new(Self::BAUDRATE_500K)
    }
}

/// Channel status
///
/// Represents the operational state of a CAN channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelStatus {
    /// Channel is closed (not initialized)
    Closed,
    /// Channel is open and operating normally
    Active,
    /// Channel is in bus-off state (too many errors)
    BusOff,
    /// Channel has warnings (error counters elevated)
    Warning,
    /// Channel has errors
    Error,
    /// Channel is in passive error state
    Passive,
}

impl ChannelStatus {
    /// Check if the channel is operational (Active or Warning)
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Active | Self::Warning)
    }

    /// Check if the channel has errors
    pub fn has_errors(&self) -> bool {
        matches!(self, Self::BusOff | Self::Error | Self::Passive)
    }
}

/// Driver version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverVersion {
    /// Driver software version
    pub driver: String,
    /// Hardware/firmware version (if applicable)
    pub hardware: Option<String>,
    /// API version
    pub api: Option<String>,
}

impl DriverVersion {
    /// Create a new driver version
    pub fn new(driver: impl Into<String>) -> Self {
        Self {
            driver: driver.into(),
            hardware: None,
            api: None,
        }
    }
}

/// Driver factory trait for creating driver instances
///
/// This trait is implemented by driver plugins to allow dynamic driver loading.
///
/// # Example
///
/// ```
/// use busmaster_core::Result;
/// use busmaster_dil::{CanDriver, DriverFactory};
///
/// struct MyDriverFactory;
///
/// impl DriverFactory for MyDriverFactory {
///     fn create(&self) -> Result<Box<dyn CanDriver>> {
///         // Create and return a new driver instance
///         # unimplemented!()
///     }
///
///     fn name(&self) -> &str {
///         "MyDriver"
///     }
///
///     fn vendor(&self) -> &str {
///         "MyCompany"
///     }
/// }
/// ```
pub trait DriverFactory: Send + Sync {
    /// Create a new driver instance
    ///
    /// # Returns
    ///
    /// A boxed driver instance
    ///
    /// # Errors
    ///
    /// Returns an error if driver initialization fails.
    fn create(&self) -> Result<Box<dyn CanDriver>>;

    /// Get the driver name
    fn name(&self) -> &str;

    /// Get the driver vendor name
    fn vendor(&self) -> &str;

    /// Get driver description (optional)
    fn description(&self) -> Option<&str> {
        None
    }

    /// Check if the driver is available on this platform
    fn is_available(&self) -> bool {
        true
    }
}

/// Channel handle for managing open channels
///
/// Provides a type-safe way to reference open channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelHandle {
    /// Channel number
    pub channel: u8,
    /// Driver instance ID (for multi-driver scenarios)
    pub driver_id: u32,
}

impl ChannelHandle {
    /// Create a new channel handle
    pub fn new(channel: u8, driver_id: u32) -> Self {
        Self { channel, driver_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_builder() {
        let info = DeviceInfo::new("PCAN-USB", "12345", 1)
            .with_hardware_version("1.0")
            .with_firmware_version("2.0");

        assert_eq!(info.name, "PCAN-USB");
        assert_eq!(info.serial, "12345");
        assert_eq!(info.channel_count, 1);
        assert_eq!(info.hardware_version, Some("1.0".to_string()));
        assert_eq!(info.firmware_version, Some("2.0".to_string()));
    }

    #[test]
    fn test_channel_config_builder() {
        let config = ChannelConfig::new(500_000)
            .with_silent(true)
            .with_loopback(false);

        assert_eq!(config.baudrate, 500_000);
        assert!(config.silent);
        assert!(!config.loopback);
        assert!(!config.fd_enabled);
    }

    #[test]
    fn test_channel_config_fd() {
        let config = ChannelConfig::new(500_000).with_fd(2_000_000);

        assert!(config.fd_enabled);
        assert_eq!(config.fd_baudrate, Some(2_000_000));
    }

    #[test]
    fn test_channel_config_default() {
        let config = ChannelConfig::default();
        assert_eq!(config.baudrate, ChannelConfig::BAUDRATE_500K);
        assert!(!config.silent);
        assert!(!config.loopback);
    }

    #[test]
    fn test_channel_config_constants() {
        assert_eq!(ChannelConfig::BAUDRATE_125K, 125_000);
        assert_eq!(ChannelConfig::BAUDRATE_250K, 250_000);
        assert_eq!(ChannelConfig::BAUDRATE_500K, 500_000);
        assert_eq!(ChannelConfig::BAUDRATE_1M, 1_000_000);
    }

    #[test]
    fn test_channel_status_operational() {
        assert!(ChannelStatus::Active.is_operational());
        assert!(ChannelStatus::Warning.is_operational());
        assert!(!ChannelStatus::Closed.is_operational());
        assert!(!ChannelStatus::BusOff.is_operational());
    }

    #[test]
    fn test_channel_status_errors() {
        assert!(ChannelStatus::BusOff.has_errors());
        assert!(ChannelStatus::Error.has_errors());
        assert!(ChannelStatus::Passive.has_errors());
        assert!(!ChannelStatus::Active.has_errors());
    }

    #[test]
    fn test_channel_handle() {
        let handle = ChannelHandle::new(0, 1);
        assert_eq!(handle.channel, 0);
        assert_eq!(handle.driver_id, 1);
    }

    #[test]
    fn test_driver_version() {
        let version = DriverVersion::new("1.0.0");
        assert_eq!(version.driver, "1.0.0");
        assert_eq!(version.hardware, None);
        assert_eq!(version.api, None);
    }
}
