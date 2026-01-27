//! Main toolbar panel

use egui::{Ui, RichText, Color32};
use crate::state::{AppState, ConnectionStatus, LoggingStatus, ActivePanel};

/// Main toolbar with quick actions
pub struct Toolbar;

impl Toolbar {
    /// Render the toolbar
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            // Connection button
            let connect_text = match state.connection_status {
                ConnectionStatus::Disconnected => "🔌 Connect",
                ConnectionStatus::Connecting => "⏳ Connecting...",
                ConnectionStatus::Connected => "🔗 Disconnect",
                ConnectionStatus::Error => "❌ Retry",
            };
            
            let connect_color = match state.connection_status {
                ConnectionStatus::Connected => Color32::from_rgb(76, 175, 80),
                ConnectionStatus::Error => Color32::from_rgb(244, 67, 54),
                _ => Color32::from_rgb(158, 158, 158),
            };

            if ui.button(RichText::new(connect_text).color(connect_color)).clicked() {
                match state.connection_status {
                    ConnectionStatus::Disconnected | ConnectionStatus::Error => {
                        state.connection_status = ConnectionStatus::Connecting;
                        // TODO: Actually connect
                    }
                    ConnectionStatus::Connected => {
                        state.connection_status = ConnectionStatus::Disconnected;
                        // TODO: Actually disconnect
                    }
                    _ => {}
                }
            }

            ui.separator();

            // Logging button
            let log_text = match state.logging_status {
                LoggingStatus::Stopped => "⏺ Start Log",
                LoggingStatus::Recording => "⏹ Stop Log",
                LoggingStatus::Paused => "▶ Resume Log",
            };

            let log_color = match state.logging_status {
                LoggingStatus::Recording => Color32::from_rgb(244, 67, 54),
                LoggingStatus::Paused => Color32::from_rgb(255, 193, 7),
                _ => Color32::from_rgb(158, 158, 158),
            };

            if ui.button(RichText::new(log_text).color(log_color)).clicked() {
                state.logging_status = match state.logging_status {
                    LoggingStatus::Stopped => LoggingStatus::Recording,
                    LoggingStatus::Recording => LoggingStatus::Stopped,
                    LoggingStatus::Paused => LoggingStatus::Recording,
                };
            }

            ui.separator();

            // Clear messages
            if ui.button("🗑 Clear").clicked() {
                state.clear_messages();
            }

            ui.separator();

            // Panel selection
            ui.label("View:");
            
            if ui.selectable_label(state.active_panel == ActivePanel::Messages, "📨 Messages").clicked() {
                state.active_panel = ActivePanel::Messages;
            }
            if ui.selectable_label(state.active_panel == ActivePanel::SignalGraph, "📈 Graph").clicked() {
                state.active_panel = ActivePanel::SignalGraph;
            }
            if ui.selectable_label(state.active_panel == ActivePanel::Diagnostics, "🔧 Diagnostics").clicked() {
                state.active_panel = ActivePanel::Diagnostics;
            }
            if ui.selectable_label(state.active_panel == ActivePanel::Calibration, "⚙ Calibration").clicked() {
                state.active_panel = ActivePanel::Calibration;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Settings button
                if ui.button("⚙").on_hover_text("Settings").clicked() {
                    state.show_settings_dialog = true;
                }

                // Filter button
                if ui.button("🔍").on_hover_text("Filter").clicked() {
                    state.show_filter_dialog = true;
                }
            });
        });
    }
}
