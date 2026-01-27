//! Database browser panel
//!
//! Tree view for browsing loaded databases (DBC, DBF, LDF, ARXML, ODX, A2L).

use egui::{Ui, RichText, CollapsingHeader};
use crate::state::{AppState, LoadedDatabase, DatabaseType};
use crate::theme::DatabaseColors;

/// Database browser panel showing loaded databases in a tree view
pub struct DatabaseBrowserPanel;

impl DatabaseBrowserPanel {
    /// Render the database browser
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        ui.heading("📁 Databases");
        ui.separator();

        if state.loaded_databases.is_empty() {
            ui.label("No databases loaded");
            ui.label("");
            if ui.button("📂 Load Database...").clicked() {
                // TODO: Open file dialog
            }
            return;
        }

        // Clone the database list to avoid borrow issues
        let databases = state.loaded_databases.to_vec();
        
        // Show each loaded database
        for db in &databases {
            Self::show_database(ui, db);
        }

        ui.separator();
        if ui.button("📂 Load Database...").clicked() {
            // TODO: Open file dialog
        }
    }

    fn show_database(ui: &mut Ui, db: &LoadedDatabase) {
        let icon = match db.db_type {
            DatabaseType::Dbc => "📊",
            DatabaseType::Dbf => "📋",
            DatabaseType::Ldf => "🔗",
            DatabaseType::Arxml => "📦",
            DatabaseType::Odx => "🔧",
            DatabaseType::A2l => "⚙",
        };

        let filename = db.path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        CollapsingHeader::new(format!("{} {}", icon, filename))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(RichText::new(format!("Type: {}", db.db_type.name())).small());
                ui.label(RichText::new(format!("Messages: {}", db.message_count)).small());
                ui.label(RichText::new(format!("Signals: {}", db.signal_count)).small());

                // TODO: Show actual database contents as tree
                // For now, show placeholder
                ui.label("");
                ui.label(RichText::new("Messages:").color(DatabaseColors::MESSAGE));
                
                // Placeholder messages
                Self::show_placeholder_message(ui, "EngineData", 0x100, &["EngineSpeed", "EngineTemp"]);
                Self::show_placeholder_message(ui, "VehicleSpeed", 0x200, &["Speed", "Odometer"]);
            });
    }

    fn show_placeholder_message(ui: &mut Ui, name: &str, id: u32, signals: &[&str]) {
        CollapsingHeader::new(RichText::new(format!("📨 {} (0x{:03X})", name, id)).color(DatabaseColors::MESSAGE))
            .default_open(false)
            .show(ui, |ui| {
                for signal in signals {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("  📊 {}", signal)).color(DatabaseColors::SIGNAL));
                        if ui.small_button("👁").on_hover_text("Add to Watch").clicked() {
                            // TODO: Add to signal watch
                        }
                        if ui.small_button("📈").on_hover_text("Add to Graph").clicked() {
                            // TODO: Add to graph
                        }
                    });
                }
            });
    }
}
