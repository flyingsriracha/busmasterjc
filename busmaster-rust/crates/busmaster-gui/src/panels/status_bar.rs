//! Status bar panel

use egui::{Ui, RichText};
use crate::state::{AppState, ConnectionStatus, LoggingStatus};
use crate::theme::MessageColors;

/// Bottom status bar showing connection info and statistics
pub struct StatusBar;

impl StatusBar {
    /// Render the status bar
    pub fn show(ui: &mut Ui, state: &AppState) {
        ui.horizontal(|ui| {
            // Connection status
            let status_icon = match state.connection_status {
                ConnectionStatus::Disconnected => "⚫",
                ConnectionStatus::Connecting => "🟡",
                ConnectionStatus::Connected => "🟢",
                ConnectionStatus::Error => "🔴",
            };
            
            ui.label(RichText::new(status_icon));
            ui.label(state.connection_status.text());

            if state.connection_status == ConnectionStatus::Connected {
                ui.label(format!("| {} @ {} kbps", state.selected_driver, state.bitrate / 1000));
            }

            ui.separator();

            // Logging status
            let log_icon = match state.logging_status {
                LoggingStatus::Stopped => "⏹",
                LoggingStatus::Recording => "🔴",
                LoggingStatus::Paused => "⏸",
            };
            ui.label(log_icon);
            ui.label(match state.logging_status {
                LoggingStatus::Stopped => "Not logging",
                LoggingStatus::Recording => "Recording",
                LoggingStatus::Paused => "Paused",
            });

            ui.separator();

            // Bus load
            let load_color = if state.bus_load > 80.0 {
                MessageColors::ERROR
            } else if state.bus_load > 50.0 {
                MessageColors::WARNING
            } else {
                MessageColors::RX
            };
            ui.label(RichText::new(format!("Bus: {:.1}%", state.bus_load)).color(load_color));

            ui.separator();

            // Message rate
            ui.label(format!("Rate: {:.0} msg/s", state.message_rate));

            ui.separator();

            // Error count
            if state.error_count > 0 {
                ui.label(RichText::new(format!("Errors: {}", state.error_count)).color(MessageColors::ERROR));
            } else {
                ui.label("Errors: 0");
            }

            ui.separator();

            // Message count
            ui.label(format!("Messages: {}", state.messages.len()));

            // Right-aligned items
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Scroll lock indicator
                if state.message_scroll_locked {
                    ui.label("🔒 Auto-scroll");
                } else {
                    ui.label("🔓 Manual scroll");
                }
            });
        });
    }
}
