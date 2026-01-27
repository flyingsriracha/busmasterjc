//! Application state management
//!
//! Centralized state for the BUSMASTER GUI application.

use std::path::PathBuf;

use busmaster_core::CanFrame;

use crate::panels::diagnostics::DiagnosticsState;
use crate::panels::signal_graph::GraphState;
use crate::theme::Theme;
use crate::widgets::LoggingConfig;

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    /// Not connected to any hardware
    #[default]
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and ready
    Connected,
    /// Connection error
    Error,
}

impl ConnectionStatus {
    /// Get display text for status
    pub fn text(&self) -> &'static str {
        match self {
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Error => "Error",
        }
    }
}

/// Logging status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoggingStatus {
    /// Not logging
    #[default]
    Stopped,
    /// Currently logging
    Recording,
    /// Paused
    Paused,
}

/// A received CAN message with metadata
#[derive(Debug, Clone)]
pub struct ReceivedMessage {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Channel number
    pub channel: u8,
    /// The CAN frame
    pub frame: CanFrame,
    /// Direction (true = TX, false = RX)
    pub is_tx: bool,
    /// Decoded message name (if database loaded)
    pub name: Option<String>,
}

/// Signal watch entry
#[derive(Debug, Clone)]
pub struct WatchedSignal {
    /// Signal name
    pub name: String,
    /// Message name
    pub message_name: String,
    /// Message ID
    pub message_id: u32,
    /// Current value
    pub value: f64,
    /// Unit
    pub unit: String,
    /// Minimum seen value
    pub min_value: f64,
    /// Maximum seen value
    pub max_value: f64,
    /// Last update timestamp
    pub last_update_us: u64,
}

/// Loaded database info
#[derive(Debug, Clone)]
pub struct LoadedDatabase {
    /// File path
    pub path: PathBuf,
    /// Database type
    pub db_type: DatabaseType,
    /// Number of messages
    pub message_count: usize,
    /// Number of signals
    pub signal_count: usize,
}

/// Database type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// DBC (Vector)
    Dbc,
    /// DBF (BUSMASTER native)
    Dbf,
    /// LDF (LIN)
    Ldf,
    /// ARXML (AUTOSAR)
    Arxml,
    /// ODX (Diagnostics)
    Odx,
    /// A2L (XCP/Calibration)
    A2l,
}

impl DatabaseType {
    /// Get file extension for this type
    pub fn extension(&self) -> &'static str {
        match self {
            DatabaseType::Dbc => "dbc",
            DatabaseType::Dbf => "dbf",
            DatabaseType::Ldf => "ldf",
            DatabaseType::Arxml => "arxml",
            DatabaseType::Odx => "odx",
            DatabaseType::A2l => "a2l",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            DatabaseType::Dbc => "DBC (Vector)",
            DatabaseType::Dbf => "DBF (BUSMASTER)",
            DatabaseType::Ldf => "LDF (LIN)",
            DatabaseType::Arxml => "ARXML (AUTOSAR)",
            DatabaseType::Odx => "ODX (Diagnostics)",
            DatabaseType::A2l => "A2L (XCP)",
        }
    }
}

/// Which panel is currently active in the main area
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePanel {
    /// Message view (default)
    #[default]
    Messages,
    /// Signal graph
    SignalGraph,
    /// Diagnostics
    Diagnostics,
    /// Calibration
    Calibration,
}

/// Application state
pub struct AppState {
    /// Current theme
    pub theme: Theme,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Logging status
    pub logging_status: LoggingStatus,
    /// Active main panel
    pub active_panel: ActivePanel,

    /// Received messages (ring buffer, most recent first)
    pub messages: Vec<ReceivedMessage>,
    /// Maximum messages to keep
    pub max_messages: usize,

    /// Watched signals
    pub watched_signals: Vec<WatchedSignal>,

    /// Loaded databases
    pub loaded_databases: Vec<LoadedDatabase>,

    /// Signal graph state
    pub graph_state: GraphState,

    /// Diagnostics state
    pub diagnostics_state: DiagnosticsState,

    /// Logging configuration
    pub logging_config: LoggingConfig,

    /// Bus load percentage (0-100)
    pub bus_load: f32,
    /// Messages per second
    pub message_rate: f32,
    /// Error count
    pub error_count: u32,

    /// Selected driver name
    pub selected_driver: String,
    /// Selected channel
    pub selected_channel: u8,
    /// Bitrate
    pub bitrate: u32,

    /// Show filter dialog
    pub show_filter_dialog: bool,
    /// Show settings dialog
    pub show_settings_dialog: bool,
    /// Show about dialog
    pub show_about_dialog: bool,
    /// Show database dialog
    pub show_database_dialog: bool,
    /// Show logging dialog
    pub show_logging_dialog: bool,

    /// Message scroll locked (auto-scroll)
    pub message_scroll_locked: bool,

    /// Filter: ID range start
    pub filter_id_start: u32,
    /// Filter: ID range end
    pub filter_id_end: u32,
    /// Filter: show TX
    pub filter_show_tx: bool,
    /// Filter: show RX
    pub filter_show_rx: bool,
}

impl AppState {
    /// Create new application state with defaults
    pub fn new() -> Self {
        Self {
            theme: Theme::Dark,
            connection_status: ConnectionStatus::Disconnected,
            logging_status: LoggingStatus::Stopped,
            active_panel: ActivePanel::Messages,
            messages: Vec::new(),
            max_messages: 10000,
            watched_signals: Vec::new(),
            loaded_databases: Vec::new(),
            graph_state: GraphState::default(),
            diagnostics_state: DiagnosticsState::new(),
            logging_config: LoggingConfig::new(),
            bus_load: 0.0,
            message_rate: 0.0,
            error_count: 0,
            selected_driver: String::new(),
            selected_channel: 0,
            bitrate: 500_000,
            show_filter_dialog: false,
            show_settings_dialog: false,
            show_about_dialog: false,
            show_database_dialog: false,
            show_logging_dialog: false,
            message_scroll_locked: true,
            filter_id_start: 0,
            filter_id_end: 0x7FF,
            filter_show_tx: true,
            filter_show_rx: true,
        }
    }

    /// Add a received message
    pub fn add_message(&mut self, msg: ReceivedMessage) {
        self.messages.insert(0, msg);
        if self.messages.len() > self.max_messages {
            self.messages.pop();
        }
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Update a watched signal value
    pub fn update_signal(&mut self, message_id: u32, signal_name: &str, value: f64, timestamp_us: u64) {
        for signal in &mut self.watched_signals {
            if signal.message_id == message_id && signal.name == signal_name {
                signal.value = value;
                signal.min_value = signal.min_value.min(value);
                signal.max_value = signal.max_value.max(value);
                signal.last_update_us = timestamp_us;
                break;
            }
        }
    }

    /// Add a signal to watch
    pub fn add_watched_signal(&mut self, signal: WatchedSignal) {
        // Don't add duplicates
        if !self.watched_signals.iter().any(|s| 
            s.message_id == signal.message_id && s.name == signal.name
        ) {
            self.watched_signals.push(signal);
        }
    }

    /// Remove a watched signal
    pub fn remove_watched_signal(&mut self, message_id: u32, signal_name: &str) {
        self.watched_signals.retain(|s| 
            !(s.message_id == message_id && s.name == signal_name)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status_text() {
        assert_eq!(ConnectionStatus::Disconnected.text(), "Disconnected");
        assert_eq!(ConnectionStatus::Connected.text(), "Connected");
    }

    #[test]
    fn test_database_type_extension() {
        assert_eq!(DatabaseType::Dbc.extension(), "dbc");
        assert_eq!(DatabaseType::A2l.extension(), "a2l");
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.theme, Theme::Dark);
        assert_eq!(state.max_messages, 10000);
        assert_eq!(state.bitrate, 500_000);
    }

    #[test]
    fn test_add_message() {
        let mut state = AppState::new();
        state.max_messages = 3;

        for i in 0..5 {
            state.add_message(ReceivedMessage {
                timestamp_us: i as u64,
                channel: 0,
                frame: CanFrame::new_standard(0x100, &[]).unwrap(),
                is_tx: false,
                name: None,
            });
        }

        // Should only keep 3 messages
        assert_eq!(state.messages.len(), 3);
        // Most recent should be first
        assert_eq!(state.messages[0].timestamp_us, 4);
    }

    #[test]
    fn test_watched_signal() {
        let mut state = AppState::new();
        
        let signal = WatchedSignal {
            name: "EngineSpeed".to_string(),
            message_name: "EngineData".to_string(),
            message_id: 0x100,
            value: 1000.0,
            unit: "rpm".to_string(),
            min_value: 1000.0,
            max_value: 1000.0,
            last_update_us: 0,
        };

        state.add_watched_signal(signal.clone());
        assert_eq!(state.watched_signals.len(), 1);

        // Don't add duplicate
        state.add_watched_signal(signal);
        assert_eq!(state.watched_signals.len(), 1);

        // Update value
        state.update_signal(0x100, "EngineSpeed", 2000.0, 1000);
        assert_eq!(state.watched_signals[0].value, 2000.0);
        assert_eq!(state.watched_signals[0].max_value, 2000.0);

        // Remove
        state.remove_watched_signal(0x100, "EngineSpeed");
        assert!(state.watched_signals.is_empty());
    }
}
