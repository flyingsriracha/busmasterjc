//! # BUSMASTER Engine
//!
//! The main orchestration engine that coordinates all BUSMASTER components:
//! - Driver management
//! - Database loading
//! - Message reception and processing
//! - Signal extraction
//! - Filtering
//! - Logging
//! - Message subscription (pub/sub)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      BUSMASTER Engine                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌─────────┐ │
//! │  │ Driver   │──▶│ Filter   │──▶│ Signal   │──▶│ Logger  │ │
//! │  │ Manager  │   │ Pipeline │   │ Extract  │   │         │ │
//! │  └──────────┘   └──────────┘   └──────────┘   └─────────┘ │
//! │       │              │               │              │       │
//! │       └──────────────┴───────────────┴──────────────┘       │
//! │                          │                                  │
//! │                    ┌─────▼─────┐                           │
//! │                    │  Pub/Sub  │                           │
//! │                    │ Broadcast │                           │
//! │                    └───────────┘                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Example
//!
//! ```no_run
//! use busmaster_engine::{Engine, EngineConfig};
//! use busmaster_hardware::StubDriver;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create engine with stub driver
//!     let driver = Box::new(StubDriver::new());
//!     let config = EngineConfig::default();
//!     let mut engine = Engine::new(driver, config)?;
//!     
//!     // Start the engine
//!     engine.start().await?;
//!     
//!     // Subscribe to messages
//!     let mut rx = engine.subscribe();
//!     
//!     // Process messages
//!     while let Ok(frame) = rx.recv().await {
//!         println!("Received: {:?}", frame);
//!     }
//!     
//!     // Stop the engine
//!     engine.stop().await?;
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]

mod engine;
mod error;
mod subscription;

pub use engine::{Engine, EngineConfig, EngineState};
pub use error::{EngineError, Result};
pub use subscription::{MessageEvent, Subscriber};
