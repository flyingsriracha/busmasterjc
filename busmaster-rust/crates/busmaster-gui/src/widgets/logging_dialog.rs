//! Logging configuration dialog
//!
//! Configure log file settings (format, path, triggers).

use egui::{Context, Window, Color32};
use crate::state::AppState;

/// Log file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// ASC (Vector ASCII)
    #[default]
    Asc,
    /// BLF (Vector Binary)
    Blf,
    /// PCAP (Wireshark)
    Pcap,
    /// CSV (Comma-separated)
    Csv,
}

impl LogFormat {
    /// Get format name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Asc => "ASC (Vector ASCII)",
            Self::Blf => "BLF (Vector Binary)",
            Self::Pcap => "PCAP (Wireshark)",
            Self::Csv => "CSV (Spreadsheet)",
        }
    }

    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Blf => "blf",
            Self::Pcap => "pcap",
            Self::Csv => "csv",
        }
    }
}

/// Log trigger mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogTrigger {
    /// Manual start/stop
    #[default]
    Manual,
    /// Start on connect
    OnConnect,
    /// Start on first message
    OnFirstMessage,
    /// Trigger on specific message ID
    OnMessageId,
}

impl LogTrigger {
    /// Get trigger name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::OnConnect => "On Connect",
            Self::OnFirstMessage => "On First Message",
            Self::OnMessageId => "On Message ID",
        }
    }
}

/// Logging configuration state
#[derive(Debug, Clone, Default)]
pub struct LoggingConfig {
    /// Log file format
    pub format: LogFormat,
    /// Log file path
    pub file_path: String,
    /// Auto-generate filename
    pub auto_filename: bool,
    /// Log trigger mode
    pub trigger: LogTrigger,
    /// Trigger message ID (for OnMessageId trigger)
    pub trigger_id: u32,
    /// Maximum file size in MB (0 = unlimited)
    pub max_file_size_mb: u32,
    /// Split files when max size reached
    pub split_files: bool,
    /// Include timestamps
    pub include_timestamps: bool,
    /// Include decoded signals
    pub include_signals: bool,
    /// Log only filtered messages
    pub respect_filter: bool,
}

impl LoggingConfig {
    /// Create new logging config with defaults
    pub fn new() -> Self {
        Self {
            format: LogFormat::Asc,
            file_path: String::new(),
            auto_filename: true,
            trigger: LogTrigger::Manual,
            trigger_id: 0,
            max_file_size_mb: 100,
            split_files: true,
            include_timestamps: true,
            include_signals: true,
            respect_filter: false,
        }
    }

    /// Generate automatic filename
    pub fn generate_filename(&self) -> String {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        format!("busmaster_log_{}.{}", timestamp, self.format.extension())
    }
}

/// Logging configuration dialog
pub struct LoggingDialog;

impl LoggingDialog {
    /// Show the logging dialog if enabled
    pub fn show(ctx: &Context, state: &mut AppState) {
        if !state.show_logging_dialog {
            return;
        }

        Window::new("📝 Logging Configuration")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("File Settings");
                ui.add_space(8.0);

                // Format selection
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    egui::ComboBox::from_id_salt("log_format")
                        .selected_text(state.logging_config.format.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.logging_config.format, LogFormat::Asc, "ASC (Vector ASCII)");
                            ui.selectable_value(&mut state.logging_config.format, LogFormat::Blf, "BLF (Vector Binary)");
                            ui.selectable_value(&mut state.logging_config.format, LogFormat::Pcap, "PCAP (Wireshark)");
                            ui.selectable_value(&mut state.logging_config.format, LogFormat::Csv, "CSV (Spreadsheet)");
                        });
                });

                // Auto filename
                ui.checkbox(&mut state.logging_config.auto_filename, "Auto-generate filename");

                if state.logging_config.auto_filename {
                    let filename = state.logging_config.generate_filename();
                    ui.horizontal(|ui| {
                        ui.label("Filename:");
                        ui.monospace(&filename);
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("File path:");
                        ui.add(egui::TextEdit::singleline(&mut state.logging_config.file_path)
                            .desired_width(250.0));
                    });
                }

                ui.add_space(8.0);
                ui.separator();

                // Trigger settings
                ui.heading("Trigger Settings");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Start trigger:");
                    egui::ComboBox::from_id_salt("log_trigger")
                        .selected_text(state.logging_config.trigger.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.logging_config.trigger, LogTrigger::Manual, "Manual");
                            ui.selectable_value(&mut state.logging_config.trigger, LogTrigger::OnConnect, "On Connect");
                            ui.selectable_value(&mut state.logging_config.trigger, LogTrigger::OnFirstMessage, "On First Message");
                            ui.selectable_value(&mut state.logging_config.trigger, LogTrigger::OnMessageId, "On Message ID");
                        });
                });

                if state.logging_config.trigger == LogTrigger::OnMessageId {
                    ui.horizontal(|ui| {
                        ui.label("Trigger ID:");
                        ui.add(egui::DragValue::new(&mut state.logging_config.trigger_id)
                            .hexadecimal(3, false, true)
                            .range(0..=0x1FFFFFFF));
                    });
                }

                ui.add_space(8.0);
                ui.separator();

                // File size settings
                ui.heading("Size Limits");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Max file size (MB):");
                    ui.add(egui::DragValue::new(&mut state.logging_config.max_file_size_mb)
                        .range(0..=10000));
                    if state.logging_config.max_file_size_mb == 0 {
                        ui.label("(unlimited)");
                    }
                });

                if state.logging_config.max_file_size_mb > 0 {
                    ui.checkbox(&mut state.logging_config.split_files, "Split into multiple files");
                }

                ui.add_space(8.0);
                ui.separator();

                // Content settings
                ui.heading("Content Options");
                ui.add_space(8.0);

                ui.checkbox(&mut state.logging_config.include_timestamps, "Include timestamps");
                ui.checkbox(&mut state.logging_config.include_signals, "Include decoded signals");
                ui.checkbox(&mut state.logging_config.respect_filter, "Log only filtered messages");

                ui.add_space(16.0);
                ui.separator();

                // Status indicator
                ui.horizontal(|ui| {
                    let status_color = match state.logging_status {
                        crate::state::LoggingStatus::Stopped => Color32::from_rgb(150, 150, 150),
                        crate::state::LoggingStatus::Recording => Color32::from_rgb(255, 80, 80),
                        crate::state::LoggingStatus::Paused => Color32::from_rgb(255, 200, 80),
                    };
                    ui.colored_label(status_color, format!("● Status: {:?}", state.logging_status));
                });

                ui.add_space(8.0);

                // Buttons
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        state.show_logging_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_logging_dialog = false;
                    }
                    if ui.button("Reset Defaults").clicked() {
                        state.logging_config = LoggingConfig::new();
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format() {
        assert_eq!(LogFormat::Asc.extension(), "asc");
        assert_eq!(LogFormat::Blf.extension(), "blf");
        assert_eq!(LogFormat::Pcap.extension(), "pcap");
        assert_eq!(LogFormat::Csv.extension(), "csv");
    }

    #[test]
    fn test_log_trigger() {
        assert_eq!(LogTrigger::Manual.name(), "Manual");
        assert_eq!(LogTrigger::OnConnect.name(), "On Connect");
    }

    #[test]
    fn test_logging_config_new() {
        let config = LoggingConfig::new();
        assert_eq!(config.format, LogFormat::Asc);
        assert!(config.auto_filename);
        assert_eq!(config.max_file_size_mb, 100);
    }

    #[test]
    fn test_generate_filename() {
        let config = LoggingConfig::new();
        let filename = config.generate_filename();
        assert!(filename.starts_with("busmaster_log_"));
        assert!(filename.ends_with(".asc"));
    }
}
