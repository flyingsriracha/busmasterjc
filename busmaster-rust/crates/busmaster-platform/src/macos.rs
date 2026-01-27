//! macOS platform implementation
//!
//! Provides macOS-specific functionality including:
//! - High-precision timestamps using `Instant`
//! - USB device enumeration using `system_profiler`

use crate::{Platform, UsbDevice};
use busmaster_core::Result;
use std::process::Command;
use std::time::Instant;

/// macOS platform implementation
pub struct MacOsPlatform {
    start_time: Instant,
}

impl MacOsPlatform {
    /// Create a new macOS platform instance
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// Parse USB device information from system_profiler output
    fn parse_usb_devices(output: &str) -> Vec<UsbDevice> {
        let mut devices = Vec::new();
        let mut current_device: Option<UsbDeviceBuilder> = None;
        let mut in_device_section = false;

        for line in output.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for device name (ends with colon, has some indentation)
            if trimmed.ends_with(':') && !trimmed.contains("ID") && line.starts_with("        ") {
                // Save previous device if complete
                if let Some(builder) = current_device.take() {
                    if let Some(device) = builder.build() {
                        devices.push(device);
                    }
                }
                // Start new device
                let name = trimmed.trim_end_matches(':').to_string();
                current_device = Some(UsbDeviceBuilder::new(name));
                in_device_section = true;
            } else if in_device_section {
                // Parse device properties
                if let Some((key, value)) = trimmed.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();

                    if let Some(ref mut builder) = current_device {
                        match key {
                            "Vendor ID" => {
                                // Format: "0x1234 (Vendor Name)" or "0x1234"
                                if let Some(hex) = value.split_whitespace().next() {
                                    if let Some(hex_str) = hex.strip_prefix("0x") {
                                        if let Ok(vid) = u16::from_str_radix(hex_str, 16) {
                                            builder.vendor_id = Some(vid);
                                        }
                                    }
                                }
                            },
                            "Product ID" => {
                                if let Some(hex) = value.split_whitespace().next() {
                                    if let Some(hex_str) = hex.strip_prefix("0x") {
                                        if let Ok(pid) = u16::from_str_radix(hex_str, 16) {
                                            builder.product_id = Some(pid);
                                        }
                                    }
                                }
                            },
                            "Serial Number" => {
                                if !value.is_empty() {
                                    builder.serial = Some(value.to_string());
                                }
                            },
                            "Location ID" => {
                                // Use location ID as path
                                builder.path = Some(value.to_string());
                            },
                            _ => {},
                        }
                    }
                }
            }
        }

        // Don't forget the last device
        if let Some(builder) = current_device {
            if let Some(device) = builder.build() {
                devices.push(device);
            }
        }

        devices
    }
}

/// Builder for USB device
struct UsbDeviceBuilder {
    name: String,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    path: Option<String>,
    serial: Option<String>,
}

impl UsbDeviceBuilder {
    fn new(name: String) -> Self {
        Self {
            name,
            vendor_id: None,
            product_id: None,
            path: None,
            serial: None,
        }
    }

    fn build(self) -> Option<UsbDevice> {
        // Only build if we have vendor and product IDs
        let vendor_id = self.vendor_id?;
        let product_id = self.product_id?;

        Some(UsbDevice {
            vendor_id,
            product_id,
            path: self.path.unwrap_or_else(|| self.name.clone()),
            serial: self.serial,
        })
    }
}

impl Default for MacOsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for MacOsPlatform {
    fn timestamp_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    fn list_usb_devices(&self) -> Result<Vec<UsbDevice>> {
        // Use system_profiler to list USB devices
        let output = Command::new("system_profiler")
            .args(["SPUSBDataType", "-detailLevel", "mini"])
            .output()
            .map_err(busmaster_core::BusmasterError::Io)?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Self::parse_usb_devices(&stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_creation() {
        let platform = MacOsPlatform::new();
        assert!(platform.timestamp_us() < 1_000_000); // Should be less than 1 second
    }

    #[test]
    fn test_timestamp_increases() {
        let platform = MacOsPlatform::new();
        let t1 = platform.timestamp_us();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = platform.timestamp_us();
        assert!(t2 > t1);
        assert!(t2 - t1 >= 10_000); // At least 10ms in microseconds
    }

    #[test]
    fn test_list_usb_devices() {
        let platform = MacOsPlatform::new();
        // This should not panic, even if no devices are found
        let result = platform.list_usb_devices();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_usb_devices_empty() {
        let devices = MacOsPlatform::parse_usb_devices("");
        assert!(devices.is_empty());
    }

    #[test]
    fn test_parse_usb_devices_sample() {
        let sample_output = r#"
USB:

    USB 3.1 Bus:

      Host Controller Driver: AppleT8103USBXHCI

        USB Device:

          Product ID: 0x1234
          Vendor ID: 0x5678 (Some Vendor)
          Serial Number: ABC123
          Location ID: 0x14100000

        Another Device:

          Product ID: 0xABCD
          Vendor ID: 0xEF01
          Location ID: 0x14200000
"#;

        let devices = MacOsPlatform::parse_usb_devices(sample_output);
        assert_eq!(devices.len(), 2);

        assert_eq!(devices[0].vendor_id, 0x5678);
        assert_eq!(devices[0].product_id, 0x1234);
        assert_eq!(devices[0].serial, Some("ABC123".to_string()));

        assert_eq!(devices[1].vendor_id, 0xEF01);
        assert_eq!(devices[1].product_id, 0xABCD);
        assert_eq!(devices[1].serial, None);
    }

    #[test]
    fn test_default_impl() {
        let platform = MacOsPlatform::default();
        assert!(platform.timestamp_us() < 1_000_000);
    }
}
