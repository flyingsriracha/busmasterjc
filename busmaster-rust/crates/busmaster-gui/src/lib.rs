//! BUSMASTER GUI Application
//!
//! A professional automotive bus monitoring and analysis tool built with egui.
//! Designed to rival CANoe, CANape, and ETAS INCA.
//!
//! # Features
//!
//! - Real-time CAN/LIN/Ethernet message monitoring
//! - Signal extraction and visualization
//! - Database browser (DBC, DBF, LDF, ARXML, ODX, A2L)
//! - Signal graphing with time-series plots
//! - Diagnostics panel (UDS, OBD-II)
//! - ECU explorer with auto-detection
//! - Calibration view (measurement and parameter editing)
//! - Logging configuration
//!
//! # Example
//!
//! ```no_run
//! use busmaster_gui::BusmasterApp;
//!
//! fn main() -> eframe::Result<()> {
//!     let options = eframe::NativeOptions::default();
//!     eframe::run_native(
//!         "BUSMASTER",
//!         options,
//!         Box::new(|cc| Ok(Box::new(BusmasterApp::new(cc)))),
//!     )
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod app;
mod panels;
mod state;
#[cfg(test)]
mod tests;
mod theme;
mod widgets;

pub use app::BusmasterApp;
pub use state::AppState;
pub use theme::Theme;
