//! BUSMASTER Hardware Drivers
//!
//! This crate provides hardware driver implementations for various CAN interfaces.
//! It includes both real hardware drivers and virtual drivers for testing.
//!
//! # Available Drivers
//!
//! - **[`StubDriver`]** - A simple loopback driver for testing (single process)
//! - **[`VirtualDriver`]** - A virtual CAN bus for multi-process communication
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
//! // Configure and open a channel
//! let config = ChannelConfig::new(500_000);
//! driver.open_channel(0, &config).unwrap();
//!
//! // Send a frame
//! let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
//! driver.send(0, &frame).unwrap();
//!
//! // In loopback mode, we can receive it back
//! if let Some(received) = driver.receive(0).unwrap() {
//!     assert_eq!(received.id(), frame.id());
//! }
//!
//! driver.close_channel(0).unwrap();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::unnecessary_literal_bound)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::type_complexity)]
#![allow(clippy::trivially_copy_pass_by_ref)]

mod stub;
mod r#virtual;

pub use r#virtual::{VirtualBus, VirtualDriver};
pub use stub::StubDriver;
