//! Message view panel
//!
//! Real-time display of CAN/LIN/Ethernet messages with virtual scrolling.

use egui::{Ui, RichText};
use egui_extras::{TableBuilder, Column};
use crate::state::{AppState, ReceivedMessage};
use crate::theme::MessageColors;

/// Message view panel showing real-time traffic
pub struct MessageViewPanel;

impl MessageViewPanel {
    /// Render the message view
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        // Header with controls
        ui.horizontal(|ui| {
            ui.heading("📨 Messages");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Scroll lock toggle
                if ui.selectable_label(state.message_scroll_locked, "🔒 Auto-scroll").clicked() {
                    state.message_scroll_locked = !state.message_scroll_locked;
                }

                // Quick filter
                ui.label("Filter:");
                // TODO: Add quick filter text input
            });
        });

        ui.separator();

        if state.messages.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No messages received yet");
            });
            return;
        }

        // Message table with virtual scrolling
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(80.0)) // Timestamp
            .column(Column::auto().at_least(30.0)) // Ch
            .column(Column::auto().at_least(30.0)) // Dir
            .column(Column::auto().at_least(80.0)) // ID
            .column(Column::auto().at_least(120.0)) // Name
            .column(Column::auto().at_least(30.0)) // DLC
            .column(Column::remainder().at_least(200.0)) // Data
            .header(20.0, |mut header| {
                header.col(|ui| { ui.strong("Time"); });
                header.col(|ui| { ui.strong("Ch"); });
                header.col(|ui| { ui.strong("Dir"); });
                header.col(|ui| { ui.strong("ID"); });
                header.col(|ui| { ui.strong("Name"); });
                header.col(|ui| { ui.strong("DLC"); });
                header.col(|ui| { ui.strong("Data"); });
            })
            .body(|body| {
                body.rows(text_height, state.messages.len(), |mut row| {
                    let idx = row.index();
                    if let Some(msg) = state.messages.get(idx) {
                        Self::render_message_row(&mut row, msg);
                    }
                });
            });
    }

    fn render_message_row(row: &mut egui_extras::TableRow, msg: &ReceivedMessage) {
        let dir_color = if msg.is_tx { MessageColors::TX } else { MessageColors::RX };

        // Timestamp (relative, in seconds)
        row.col(|ui| {
            let time_s = msg.timestamp_us as f64 / 1_000_000.0;
            ui.label(format!("{:.6}", time_s));
        });

        // Channel
        row.col(|ui| {
            ui.label(format!("{}", msg.channel));
        });

        // Direction
        row.col(|ui| {
            let dir_text = if msg.is_tx { "TX" } else { "RX" };
            ui.label(RichText::new(dir_text).color(dir_color));
        });

        // ID
        row.col(|ui| {
            let id_text = if msg.frame.is_extended() {
                format!("0x{:08X}", msg.frame.id())
            } else {
                format!("0x{:03X}", msg.frame.id())
            };
            ui.label(id_text);
        });

        // Name
        row.col(|ui| {
            if let Some(name) = &msg.name {
                ui.label(name);
            } else {
                ui.label(RichText::new("-").weak());
            }
        });

        // DLC
        row.col(|ui| {
            ui.label(format!("{}", msg.frame.dlc()));
        });

        // Data
        row.col(|ui| {
            let data_str: String = msg.frame.data()
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            ui.label(data_str);
        });
    }
}
