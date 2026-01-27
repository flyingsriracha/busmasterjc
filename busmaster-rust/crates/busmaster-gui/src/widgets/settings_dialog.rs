//! Settings dialog

use egui::{Context, Window};
use crate::state::AppState;
use crate::theme::Theme;

/// Settings dialog
pub struct SettingsDialog;

impl SettingsDialog {
    /// Show the settings dialog if enabled
    pub fn show(ctx: &Context, state: &mut AppState) {
        if !state.show_settings_dialog {
            return;
        }

        Window::new("⚙ Settings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Appearance");
                
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    egui::ComboBox::from_id_salt("theme_selector")
                        .selected_text(state.theme.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.theme, Theme::Dark, "Dark");
                            ui.selectable_value(&mut state.theme, Theme::Light, "Light");
                            ui.selectable_value(&mut state.theme, Theme::HighContrast, "High Contrast");
                        });
                });

                ui.separator();

                ui.heading("Connection");
                
                ui.horizontal(|ui| {
                    ui.label("Driver:");
                    egui::ComboBox::from_id_salt("driver_selector")
                        .selected_text(&state.selected_driver)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.selected_driver, "Stub".to_string(), "Stub (Virtual)");
                            ui.selectable_value(&mut state.selected_driver, "Virtual".to_string(), "Virtual Bus");
                            ui.selectable_value(&mut state.selected_driver, "PEAK".to_string(), "PEAK USB");
                            ui.selectable_value(&mut state.selected_driver, "Vector".to_string(), "Vector XL");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Channel:");
                    ui.add(egui::DragValue::new(&mut state.selected_channel).range(0..=15));
                });

                ui.horizontal(|ui| {
                    ui.label("Bitrate:");
                    egui::ComboBox::from_id_salt("bitrate_selector")
                        .selected_text(format!("{} kbps", state.bitrate / 1000))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.bitrate, 125_000, "125 kbps");
                            ui.selectable_value(&mut state.bitrate, 250_000, "250 kbps");
                            ui.selectable_value(&mut state.bitrate, 500_000, "500 kbps");
                            ui.selectable_value(&mut state.bitrate, 1_000_000, "1000 kbps");
                        });
                });

                ui.separator();

                ui.heading("Message Buffer");
                
                ui.horizontal(|ui| {
                    ui.label("Max messages:");
                    ui.add(egui::DragValue::new(&mut state.max_messages).range(100..=100000));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        state.show_settings_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_settings_dialog = false;
                    }
                });
            });
    }
}
