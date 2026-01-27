//! Error types for the BUSMASTER engine

use busmaster_core::BusmasterError;
use std::io;
use thiserror::Error;

/// Result type for engine operations
pub type Result<T> = std::result::Result<T, EngineError>;

/// Errors that can occur in the BUSMASTER engine
#[derive(Error, Debug)]
pub enum EngineError {
    /// Driver error
    #[error("Driver error: {0}")]
    Driver(#[from] BusmasterError),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Logger error
    #[error("Logger error: {0}")]
    Logger(#[from] io::Error),

    /// Engine is not running
    #[error("Engine is not running")]
    NotRunning,

    /// Engine is already running
    #[error("Engine is already running")]
    AlreadyRunning,

    /// Channel error
    #[error("Channel error: {0}")]
    Channel(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    Subscription(String),
}

impl EngineError {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            EngineError::NotRunning | EngineError::AlreadyRunning | EngineError::Channel(_)
        )
    }
}
