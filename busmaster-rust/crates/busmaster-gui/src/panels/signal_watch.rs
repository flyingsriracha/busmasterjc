//! Signal watch panel
//!
//! Real-time display of watched signal values.

use egui::{Ui, RichText, Color32};
use crate::state::{AppState, WatchedSignal};
use crate::theme::SignalColors;

/// Signal watch panel showing live signal values
pub struct SignalWatchPanel;

impl SignalWatchPanel {
    /// Render the signal watch panel
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        ui.heading("👁 Signal Watch");
        ui.separator();

        if state.watched_signals.is_empty() {
            ui.label("No signals being watched");
            ui.label("");
            ui.label(RichText::new("Drag signals from the Database Browser").small().weak());
            ui.label(RichText::new("or right-click a message to add signals").small().weak());
            return;
        }

        // Signal list
        let mut to_remove: Option<(u32, String)> = None;

        for signal in &state.watched_signals {
            ui.horizontal(|ui| {
                // Signal name and message
                ui.vertical(|ui| {
                    ui.label(RichText::new(&signal.name).strong());
                    ui.label(RichText::new(format!("{} (0x{:03X})", signal.message_name, signal.message_id)).small().weak());
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Remove button
                    if ui.small_button("✖").on_hover_text("Remove from watch").clicked() {
                        to_remove = Some((signal.message_id, signal.name.clone()));
                    }

                    // Graph button
                    if ui.small_button("📈").on_hover_text("Add to graph").clicked() {
                        // TODO: Add to graph
                    }

                    // Value with unit
                    let value_text = if signal.unit.is_empty() {
                        format!("{:.2}", signal.value)
                    } else {
                        format!("{:.2} {}", signal.value, signal.unit)
                    };
                    
                    // Color based on value position in range
                    let value_color = Self::value_color(signal);
                    ui.label(RichText::new(value_text).color(value_color).strong());

                    // Min/Max
                    ui.label(RichText::new(format!("[{:.1} - {:.1}]", signal.min_value, signal.max_value)).small().weak());
                });
            });

            ui.separator();
        }

        // Remove signal if requested
        if let Some((msg_id, sig_name)) = to_remove {
            state.remove_watched_signal(msg_id, &sig_name);
        }
    }

    fn value_color(signal: &WatchedSignal) -> Color32 {
        // If min == max, we don't have enough data yet
        if (signal.max_value - signal.min_value).abs() < f64::EPSILON {
            return SignalColors::NORMAL;
        }

        // Calculate position in range (0.0 to 1.0)
        let range = signal.max_value - signal.min_value;
        let position = (signal.value - signal.min_value) / range;

        if position < 0.1 {
            SignalColors::MIN
        } else if position > 0.9 {
            SignalColors::MAX
        } else {
            SignalColors::NORMAL
        }
    }
}
