//! # BUSMASTER Core Types
//!
//! This crate provides the fundamental types used throughout the BUSMASTER
//! Rust implementation. It is designed to be safe, efficient, and easy to use.
//!
//! ## Features
//!
//! - **CAN Frames**: [`CanFrame`] for CAN 2.0 and [`CanFdFrame`] for CAN FD
//! - **Signals**: [`SignalDef`] for signal definitions and [`SignalValue`] for extracted values
//! - **Filtering**: [`MessageFilter`] for flexible message filtering
//! - **Errors**: Unified [`BusmasterError`] type with [`Result`] alias
//!
//! ## Safety
//!
//! This crate uses `#![forbid(unsafe_code)]` to ensure memory safety without
//! any unsafe blocks.
//!
//! ## Example
//!
//! ```
//! use busmaster_core::{CanFrame, SignalDef, ByteOrder, MessageFilter, FilterRule, Result};
//!
//! fn process_frame() -> Result<()> {
//!     // Create a CAN frame
//!     let frame = CanFrame::new_standard(0x123, &[0x01, 0x02, 0x03, 0x04])?;
//!     
//!     // Define a signal
//!     let signal = SignalDef::new("Speed", 0, 16)
//!         .with_byte_order(ByteOrder::LittleEndian)
//!         .with_factor_offset(0.01, 0.0)
//!         .with_unit("km/h");
//!     
//!     // Create a filter
//!     let filter = MessageFilter::new()
//!         .add_rule(FilterRule::IdRange { start: 0x100, end: 0x1FF });
//!     
//!     if filter.matches(&frame, 0) {
//!         println!("Frame ID: 0x{:X}, Signal: {}", frame.id(), signal.name);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Crate Structure
//!
//! - [`error`] - Error types and Result alias
//! - [`frame`] - CAN frame types ([`CanFrame`], [`CanFdFrame`])
//! - [`signal`] - Signal definition and value types
//! - [`filter`] - Message filtering

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod filter;
pub mod frame;
pub mod signal;

pub use error::{BusmasterError, Result};
pub use filter::{Direction, FilterMode, FilterRule, MessageFilter};
pub use frame::{
    CanFdFrame, CanFrame, CanXlFrame, CanXlSdt, MAX_CANFD_DATA_LEN, MAX_CAN_DATA_LEN,
    MAX_CANXL_DATA_LEN, MAX_EXTENDED_ID, MAX_STANDARD_ID,
};
pub use signal::{ByteOrder, SignalDef, SignalValue, ValueType};
