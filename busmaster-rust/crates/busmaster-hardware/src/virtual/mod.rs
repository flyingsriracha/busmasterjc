//! Virtual CAN Driver
//!
//! A virtual CAN bus implementation that allows multiple processes to communicate
//! through Unix domain sockets. This enables CLI-TUI communication without physical hardware.
//!
//! # Architecture
//!
//! ```text
//! CLI Process          Virtual CAN Bus          TUI Process
//! ┌─────────┐         ┌──────────────┐         ┌─────────┐
//! │ Virtual │────────▶│ Unix Socket  │────────▶│ Virtual │
//! │ Driver  │         │  Broadcast   │         │ Driver  │
//! └─────────┘         └──────────────┘         └─────────┘
//! ```
//!
//! # Features
//!
//! - Multiple processes can connect to the same virtual bus
//! - Messages are broadcast to all connected processes
//! - Simulates real CAN bus behavior
//! - No hardware required
//! - Cross-platform (Unix sockets on macOS/Linux, named pipes on Windows)
//!
//! # Usage
//!
//! ```rust,no_run
//! use busmaster_hardware::VirtualDriver;
//! use busmaster_dil::{CanDriver, ChannelConfig};
//!
//! let mut driver = VirtualDriver::new();
//! let config = ChannelConfig::default();
//! // Note: open_channel requires a running VirtualBus server
//! // driver.open_channel(0, &config).unwrap();
//! ```

mod bus;
mod driver;
mod protocol;

pub use bus::VirtualBus;
pub use driver::VirtualDriver;

/// Default socket path for the virtual CAN bus
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/busmaster-virtual-can.sock";

/// Maximum number of concurrent connections
pub const MAX_CONNECTIONS: usize = 10;

/// Message buffer size per connection
pub const MESSAGE_BUFFER_SIZE: usize = 1000;
