//! Stub CAN driver for testing and development
//!
//! The stub driver provides a virtual CAN interface that can be used for testing
//! without requiring physical hardware. It supports:
//!
//! - Multiple independent channels
//! - Loopback mode (frames sent are received back)
//! - Frame injection for testing
//! - Configurable channel status
//!
//! # Example
//!
//! ```
//! use busmaster_core::CanFrame;
//! use busmaster_dil::{CanDriver, ChannelConfig};
//! use busmaster_hardware::StubDriver;
//!
//! let mut driver = StubDriver::new();
//!
//! // Open channel in loopback mode
//! let config = ChannelConfig::new(500_000).with_loopback(true);
//! driver.open_channel(0, &config).unwrap();
//!
//! // Send a frame
//! let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
//! driver.send(0, &frame).unwrap();
//!
//! // Receive it back
//! let received = driver.receive(0).unwrap().unwrap();
//! assert_eq!(received.id(), frame.id());
//! ```

use busmaster_core::{BusmasterError, CanFrame, Result};
use busmaster_dil::{
    CanDriver, ChannelConfig, ChannelStatus, DeviceInfo, DriverFactory, DriverVersion,
};
use parking_lot::RwLock;
use std::collections::VecDeque;

/// Maximum number of channels supported by the stub driver
const MAX_CHANNELS: usize = 4;

/// Maximum frames in receive buffer per channel
const MAX_BUFFER_SIZE: usize = 1000;

/// Channel state
#[derive(Debug)]
struct Channel {
    /// Channel configuration
    config: ChannelConfig,
    /// Channel status
    status: ChannelStatus,
    /// Receive buffer
    rx_buffer: VecDeque<CanFrame>,
}

impl Channel {
    fn new(config: ChannelConfig) -> Self {
        Self {
            config,
            status: ChannelStatus::Active,
            rx_buffer: VecDeque::with_capacity(MAX_BUFFER_SIZE),
        }
    }

    fn push_frame(&mut self, frame: CanFrame) -> Result<()> {
        if self.rx_buffer.len() >= MAX_BUFFER_SIZE {
            return Err(BusmasterError::BufferFull {
                capacity: MAX_BUFFER_SIZE,
            });
        }
        self.rx_buffer.push_back(frame);
        Ok(())
    }

    fn pop_frame(&mut self) -> Option<CanFrame> {
        self.rx_buffer.pop_front()
    }
}

/// Stub CAN driver for testing
///
/// This driver simulates CAN hardware without requiring physical devices.
/// It's useful for:
///
/// - Unit testing
/// - Integration testing
/// - Development without hardware
/// - CI/CD pipelines
///
/// # Features
///
/// - **Loopback Mode**: Frames sent are automatically received
/// - **Frame Injection**: Inject frames into receive buffer for testing
/// - **Multiple Channels**: Supports up to 4 independent channels
/// - **Status Control**: Manually set channel status for error testing
#[derive(Debug)]
pub struct StubDriver {
    channels: RwLock<[Option<Channel>; MAX_CHANNELS]>,
}

impl StubDriver {
    /// Create a new stub driver
    ///
    /// # Example
    ///
    /// ```
    /// use busmaster_hardware::StubDriver;
    ///
    /// let driver = StubDriver::new();
    /// ```
    pub fn new() -> Self {
        Self {
            channels: RwLock::new([None, None, None, None]),
        }
    }

    /// Inject a frame into a channel's receive buffer
    ///
    /// This is useful for testing frame reception without sending.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number
    /// * `frame` - Frame to inject
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The channel is not open
    /// - The receive buffer is full
    ///
    /// # Example
    ///
    /// ```
    /// use busmaster_core::CanFrame;
    /// use busmaster_dil::{CanDriver, ChannelConfig};
    /// use busmaster_hardware::StubDriver;
    ///
    /// let mut driver = StubDriver::new();
    /// driver.open_channel(0, &ChannelConfig::new(500_000)).unwrap();
    ///
    /// // Inject a frame
    /// let frame = CanFrame::new_standard(0x456, &[5, 6, 7, 8]).unwrap();
    /// driver.inject_frame(0, frame.clone()).unwrap();
    ///
    /// // Receive it
    /// let received = driver.receive(0).unwrap().unwrap();
    /// assert_eq!(received.id(), frame.id());
    /// ```
    pub fn inject_frame(&mut self, channel: u8, frame: CanFrame) -> Result<()> {
        let mut channels = self.channels.write();
        let ch = channels
            .get_mut(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?
            .as_mut()
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        ch.push_frame(frame)
    }

    /// Set the status of a channel
    ///
    /// This allows testing error conditions.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number
    /// * `status` - New status
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is not open.
    ///
    /// # Example
    ///
    /// ```
    /// use busmaster_dil::{CanDriver, ChannelConfig, ChannelStatus};
    /// use busmaster_hardware::StubDriver;
    ///
    /// let mut driver = StubDriver::new();
    /// driver.open_channel(0, &ChannelConfig::new(500_000)).unwrap();
    ///
    /// // Simulate bus-off condition
    /// driver.set_channel_status(0, ChannelStatus::BusOff).unwrap();
    /// assert_eq!(driver.channel_status(0).unwrap(), ChannelStatus::BusOff);
    /// ```
    pub fn set_channel_status(&mut self, channel: u8, status: ChannelStatus) -> Result<()> {
        let mut channels = self.channels.write();
        let ch = channels
            .get_mut(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?
            .as_mut()
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        ch.status = status;
        Ok(())
    }

    /// Get the number of frames in the receive buffer
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number
    ///
    /// # Returns
    ///
    /// The number of frames waiting in the buffer, or 0 if channel is closed.
    pub fn buffer_count(&self, channel: u8) -> usize {
        let channels = self.channels.read();
        channels
            .get(channel as usize)
            .and_then(|ch| ch.as_ref())
            .map_or(0, |ch| ch.rx_buffer.len())
    }

    /// Clear the receive buffer for a channel
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel number
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is not open.
    pub fn clear_buffer(&mut self, channel: u8) -> Result<()> {
        let mut channels = self.channels.write();
        let ch = channels
            .get_mut(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?
            .as_mut()
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        ch.rx_buffer.clear();
        Ok(())
    }
}

impl Default for StubDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl CanDriver for StubDriver {
    fn name(&self) -> &str {
        "Stub Driver"
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        // Return a single virtual device
        Ok(vec![DeviceInfo::new(
            "Virtual CAN Device",
            "STUB-0000",
            MAX_CHANNELS as u8,
        )
        .with_hardware_version("1.0.0")
        .with_firmware_version("1.0.0")])
    }

    fn open_channel(&mut self, channel: u8, config: &ChannelConfig) -> Result<()> {
        if channel as usize >= MAX_CHANNELS {
            return Err(BusmasterError::ChannelNotFound { channel });
        }

        let mut channels = self.channels.write();
        if channels[channel as usize].is_some() {
            return Err(BusmasterError::Config {
                message: format!("Channel {} is already open", channel),
            });
        }

        channels[channel as usize] = Some(Channel::new(config.clone()));
        Ok(())
    }

    fn close_channel(&mut self, channel: u8) -> Result<()> {
        let mut channels = self.channels.write();
        let ch = channels
            .get_mut(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        if ch.is_none() {
            return Err(BusmasterError::Config {
                message: format!("Channel {} is not open", channel),
            });
        }

        *ch = None;
        Ok(())
    }

    fn send(&mut self, channel: u8, frame: &CanFrame) -> Result<()> {
        let mut channels = self.channels.write();
        let ch = channels
            .get_mut(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?
            .as_mut()
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        // Check if channel is operational
        if !ch.status.is_operational() {
            return Err(BusmasterError::Hardware {
                vendor: "Stub".to_string(),
                message: format!("Channel {} is not operational: {:?}", channel, ch.status),
            });
        }

        // If loopback is enabled, add frame to receive buffer
        if ch.config.loopback {
            ch.push_frame(frame.clone())?;
        }

        Ok(())
    }

    fn receive(&mut self, channel: u8) -> Result<Option<CanFrame>> {
        let mut channels = self.channels.write();
        let ch = channels
            .get_mut(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?
            .as_mut()
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        Ok(ch.pop_frame())
    }

    fn channel_status(&self, channel: u8) -> Result<ChannelStatus> {
        let channels = self.channels.read();
        let ch = channels
            .get(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        Ok(ch.as_ref().map_or(ChannelStatus::Closed, |c| c.status))
    }

    fn reset_channel(&mut self, channel: u8) -> Result<()> {
        let mut channels = self.channels.write();
        let ch = channels
            .get_mut(channel as usize)
            .ok_or(BusmasterError::ChannelNotFound { channel })?
            .as_mut()
            .ok_or(BusmasterError::ChannelNotFound { channel })?;

        ch.status = ChannelStatus::Active;
        ch.rx_buffer.clear();
        Ok(())
    }

    fn version(&self) -> Option<DriverVersion> {
        Some(DriverVersion::new("1.0.0"))
    }
}

/// Factory for creating stub driver instances
#[allow(dead_code)] // Will be used when implementing driver discovery
pub struct StubDriverFactory;

impl DriverFactory for StubDriverFactory {
    fn create(&self) -> Result<Box<dyn CanDriver>> {
        Ok(Box::new(StubDriver::new()))
    }

    fn name(&self) -> &str {
        "Stub Driver"
    }

    fn vendor(&self) -> &str {
        "BUSMASTER"
    }

    fn description(&self) -> Option<&str> {
        Some("Virtual CAN driver for testing and development")
    }

    fn is_available(&self) -> bool {
        true // Always available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_driver_creation() {
        let driver = StubDriver::new();
        assert_eq!(driver.name(), "Stub Driver");
    }

    #[test]
    fn test_list_devices() {
        let driver = StubDriver::new();
        let devices = driver.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Virtual CAN Device");
        assert_eq!(devices[0].channel_count, MAX_CHANNELS as u8);
    }

    #[test]
    fn test_open_close_channel() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);

        // Open channel
        driver.open_channel(0, &config).unwrap();
        assert_eq!(driver.channel_status(0).unwrap(), ChannelStatus::Active);

        // Close channel
        driver.close_channel(0).unwrap();
        assert_eq!(driver.channel_status(0).unwrap(), ChannelStatus::Closed);
    }

    #[test]
    fn test_open_already_open_channel() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);

        driver.open_channel(0, &config).unwrap();
        let result = driver.open_channel(0, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_close_not_open_channel() {
        let mut driver = StubDriver::new();
        let result = driver.close_channel(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_channel() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);
        let result = driver.open_channel(MAX_CHANNELS as u8, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_loopback_mode() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000).with_loopback(true);
        driver.open_channel(0, &config).unwrap();

        // Send a frame
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        driver.send(0, &frame).unwrap();

        // Receive it back
        let received = driver.receive(0).unwrap().unwrap();
        assert_eq!(received.id(), frame.id());
        assert_eq!(received.data(), frame.data());
    }

    #[test]
    fn test_no_loopback() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000).with_loopback(false);
        driver.open_channel(0, &config).unwrap();

        // Send a frame
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        driver.send(0, &frame).unwrap();

        // Should not receive anything
        let received = driver.receive(0).unwrap();
        assert!(received.is_none());
    }

    #[test]
    fn test_inject_frame() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);
        driver.open_channel(0, &config).unwrap();

        // Inject a frame
        let frame = CanFrame::new_standard(0x456, &[5, 6, 7, 8]).unwrap();
        driver.inject_frame(0, frame.clone()).unwrap();

        // Receive it
        let received = driver.receive(0).unwrap().unwrap();
        assert_eq!(received.id(), frame.id());
        assert_eq!(received.data(), frame.data());
    }

    #[test]
    fn test_multiple_channels() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000).with_loopback(true);

        // Open multiple channels
        driver.open_channel(0, &config).unwrap();
        driver.open_channel(1, &config).unwrap();

        // Send on channel 0
        let frame0 = CanFrame::new_standard(0x100, &[1, 2]).unwrap();
        driver.send(0, &frame0).unwrap();

        // Send on channel 1
        let frame1 = CanFrame::new_standard(0x200, &[3, 4]).unwrap();
        driver.send(1, &frame1).unwrap();

        // Receive from channel 0
        let received0 = driver.receive(0).unwrap().unwrap();
        assert_eq!(received0.id(), 0x100);

        // Receive from channel 1
        let received1 = driver.receive(1).unwrap().unwrap();
        assert_eq!(received1.id(), 0x200);

        // Channels should be independent
        assert!(driver.receive(0).unwrap().is_none());
        assert!(driver.receive(1).unwrap().is_none());
    }

    #[test]
    fn test_set_channel_status() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);
        driver.open_channel(0, &config).unwrap();

        // Set to bus-off
        driver.set_channel_status(0, ChannelStatus::BusOff).unwrap();
        assert_eq!(driver.channel_status(0).unwrap(), ChannelStatus::BusOff);

        // Try to send - should fail
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        let result = driver.send(0, &frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_count() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);
        driver.open_channel(0, &config).unwrap();

        assert_eq!(driver.buffer_count(0), 0);

        // Inject frames
        for i in 0..5 {
            let frame = CanFrame::new_standard(0x100 + i, &[i as u8]).unwrap();
            driver.inject_frame(0, frame).unwrap();
        }

        assert_eq!(driver.buffer_count(0), 5);

        // Receive one
        driver.receive(0).unwrap();
        assert_eq!(driver.buffer_count(0), 4);
    }

    #[test]
    fn test_clear_buffer() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);
        driver.open_channel(0, &config).unwrap();

        // Inject frames
        for i in 0..5 {
            let frame = CanFrame::new_standard(0x100 + i, &[i as u8]).unwrap();
            driver.inject_frame(0, frame).unwrap();
        }

        assert_eq!(driver.buffer_count(0), 5);

        // Clear buffer
        driver.clear_buffer(0).unwrap();
        assert_eq!(driver.buffer_count(0), 0);
    }

    #[test]
    fn test_reset_channel() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);
        driver.open_channel(0, &config).unwrap();

        // Set error status and add frames
        driver.set_channel_status(0, ChannelStatus::BusOff).unwrap();
        let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
        driver.inject_frame(0, frame).unwrap();

        // Reset
        driver.reset_channel(0).unwrap();

        // Status should be active and buffer cleared
        assert_eq!(driver.channel_status(0).unwrap(), ChannelStatus::Active);
        assert_eq!(driver.buffer_count(0), 0);
    }

    #[test]
    fn test_buffer_full() {
        let mut driver = StubDriver::new();
        let config = ChannelConfig::new(500_000);
        driver.open_channel(0, &config).unwrap();

        // Fill buffer
        for i in 0..MAX_BUFFER_SIZE {
            let frame = CanFrame::new_standard(0x100, &[i as u8]).unwrap();
            driver.inject_frame(0, frame).unwrap();
        }

        // Next injection should fail
        let frame = CanFrame::new_standard(0x100, &[0]).unwrap();
        let result = driver.inject_frame(0, frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_driver_version() {
        let driver = StubDriver::new();
        let version = driver.version().unwrap();
        assert_eq!(version.driver, "1.0.0");
    }

    #[test]
    fn test_factory() {
        let factory = StubDriverFactory;
        assert_eq!(factory.name(), "Stub Driver");
        assert_eq!(factory.vendor(), "BUSMASTER");
        assert!(factory.is_available());
        assert!(factory.description().is_some());

        let driver = factory.create().unwrap();
        assert_eq!(driver.name(), "Stub Driver");
    }
}
