//! BUSMASTER Platform Abstraction Layer
//!
//! This crate provides platform-specific implementations for:
//! - High-precision timestamps
//! - USB device enumeration
//! - Other OS-specific features
//!
//! # Platform Support
//!
//! - **macOS**: Full support via `MacOsPlatform`
//! - **Windows**: Planned (stub implementation)
//! - **Linux**: Planned (stub implementation)
//!
//! # Usage
//!
//! ```rust
//! use busmaster_platform::{current_platform, Platform};
//!
//! let platform = current_platform();
//!
//! // Get high-precision timestamp
//! let timestamp = platform.timestamp_us();
//! println!("Timestamp: {} µs", timestamp);
//!
//! // List USB devices
//! if let Ok(devices) = platform.list_usb_devices() {
//!     for device in devices {
//!         println!("USB Device: {:04x}:{:04x}", device.vendor_id, device.product_id);
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

use busmaster_core::Result;

/// Platform trait for OS-specific operations
///
/// This trait defines the interface for platform-specific functionality
/// that varies between operating systems.
pub trait Platform: Send + Sync {
    /// Get high-precision timestamp in microseconds
    ///
    /// Returns the elapsed time since the platform was created,
    /// measured in microseconds. This is useful for timing CAN
    /// message reception and transmission.
    ///
    /// # Example
    ///
    /// ```rust
    /// use busmaster_platform::{current_platform, Platform};
    ///
    /// let platform = current_platform();
    /// let t1 = platform.timestamp_us();
    /// // ... do some work ...
    /// let t2 = platform.timestamp_us();
    /// println!("Elapsed: {} µs", t2 - t1);
    /// ```
    fn timestamp_us(&self) -> u64;

    /// List USB devices connected to the system
    ///
    /// Returns a list of USB devices with their vendor ID, product ID,
    /// and other identifying information. This is useful for discovering
    /// CAN hardware adapters.
    ///
    /// # Returns
    ///
    /// A vector of `UsbDevice` structs, or an error if enumeration fails.
    fn list_usb_devices(&self) -> Result<Vec<UsbDevice>>;
}

/// USB device information
///
/// Contains identifying information about a USB device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbDevice {
    /// Vendor ID (VID)
    pub vendor_id: u16,
    /// Product ID (PID)
    pub product_id: u16,
    /// Device path or location identifier
    pub path: String,
    /// Serial number (if available)
    pub serial: Option<String>,
}

impl UsbDevice {
    /// Create a new USB device
    pub fn new(vendor_id: u16, product_id: u16, path: impl Into<String>) -> Self {
        Self {
            vendor_id,
            product_id,
            path: path.into(),
            serial: None,
        }
    }

    /// Set the serial number
    pub fn with_serial(mut self, serial: impl Into<String>) -> Self {
        self.serial = Some(serial.into());
        self
    }

    /// Check if this device matches a vendor/product ID pair
    pub fn matches(&self, vendor_id: u16, product_id: u16) -> bool {
        self.vendor_id == vendor_id && self.product_id == product_id
    }
}

// Platform-specific implementations
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::MacOsPlatform;

/// Get the current platform implementation
///
/// Returns a boxed `Platform` implementation appropriate for the
/// current operating system.
///
/// # Example
///
/// ```rust
/// use busmaster_platform::current_platform;
///
/// let platform = current_platform();
/// println!("Timestamp: {} µs", platform.timestamp_us());
/// ```
pub fn current_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOsPlatform::new())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(StubPlatform::new())
    }
}

/// Stub platform for unsupported operating systems
///
/// Provides basic functionality using standard library features.
#[cfg(not(target_os = "macos"))]
pub struct StubPlatform {
    start_time: std::time::Instant,
}

#[cfg(not(target_os = "macos"))]
impl StubPlatform {
    /// Create a new stub platform
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl Default for StubPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "macos"))]
impl Platform for StubPlatform {
    fn timestamp_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    fn list_usb_devices(&self) -> Result<Vec<UsbDevice>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_device_creation() {
        let device = UsbDevice::new(0x1234, 0x5678, "/dev/usb0");
        assert_eq!(device.vendor_id, 0x1234);
        assert_eq!(device.product_id, 0x5678);
        assert_eq!(device.path, "/dev/usb0");
        assert_eq!(device.serial, None);
    }

    #[test]
    fn test_usb_device_with_serial() {
        let device = UsbDevice::new(0x1234, 0x5678, "/dev/usb0").with_serial("ABC123");
        assert_eq!(device.serial, Some("ABC123".to_string()));
    }

    #[test]
    fn test_usb_device_matches() {
        let device = UsbDevice::new(0x1234, 0x5678, "/dev/usb0");
        assert!(device.matches(0x1234, 0x5678));
        assert!(!device.matches(0x1234, 0x9999));
        assert!(!device.matches(0x9999, 0x5678));
    }

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        // Should return a valid timestamp
        let ts = platform.timestamp_us();
        assert!(ts < 1_000_000_000); // Less than 1000 seconds
    }

    #[test]
    fn test_platform_timestamp_increases() {
        let platform = current_platform();
        let t1 = platform.timestamp_us();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let t2 = platform.timestamp_us();
        assert!(t2 > t1);
    }

    #[test]
    fn test_platform_list_usb_devices() {
        let platform = current_platform();
        let result = platform.list_usb_devices();
        assert!(result.is_ok());
    }
}
