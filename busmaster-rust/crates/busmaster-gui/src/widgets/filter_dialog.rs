//! Filter configuration dialog

use egui::{Context, Window};
use crate::state::AppState;

/// Filter configuration dialog
pub struct FilterDialog;

impl FilterDialog {
    /// Show the filter dialog if enabled
    pub fn show(ctx: &Context, state: &mut AppState) {
        if !state.show_filter_dialog {
            return;
        }

        Window::new("🔍 Message Filter")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("ID Filter");
                ui.horizontal(|ui| {
                    ui.label("Start ID:");
                    ui.add(egui::DragValue::new(&mut state.filter_id_start)
                        .hexadecimal(3, false, true)
                        .range(0..=0x1FFFFFFF));
                });
                ui.horizontal(|ui| {
                    ui.label("End ID:");
                    ui.add(egui::DragValue::new(&mut state.filter_id_end)
                        .hexadecimal(3, false, true)
                        .range(0..=0x1FFFFFFF));
                });

                ui.separator();

                ui.heading("Direction Filter");
                ui.checkbox(&mut state.filter_show_tx, "Show TX messages");
                ui.checkbox(&mut state.filter_show_rx, "Show RX messages");

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        // TODO: Apply filter
                        state.show_filter_dialog = false;
                    }
                    if ui.button("Reset").clicked() {
                        state.filter_id_start = 0;
                        state.filter_id_end = 0x7FF;
                        state.filter_show_tx = true;
                        state.filter_show_rx = true;
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_filter_dialog = false;
                    }
                });
            });
    }
}
