//! Database configuration dialog
//!
//! Allows loading and managing database files (DBC, DBF, LDF, ARXML, ODX, A2L).

use egui::{Context, Window, ScrollArea, Color32};
use crate::state::{AppState, LoadedDatabase, DatabaseType};
use std::path::PathBuf;

/// Database configuration dialog
pub struct DatabaseDialog;

impl DatabaseDialog {
    /// Show the database dialog if enabled
    pub fn show(ctx: &Context, state: &mut AppState) {
        if !state.show_database_dialog {
            return;
        }

        Window::new("📁 Database Manager")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.heading("Loaded Databases");
                ui.add_space(8.0);

                // Load database buttons
                ui.horizontal(|ui| {
                    if ui.button("📂 Load DBC...").clicked() {
                        // In a real app, this would open a file dialog
                        // For now, simulate loading a database
                        state.loaded_databases.push(LoadedDatabase {
                            path: PathBuf::from("example.dbc"),
                            db_type: DatabaseType::Dbc,
                            message_count: 42,
                            signal_count: 156,
                        });
                    }
                    if ui.button("📂 Load DBF...").clicked() {
                        state.loaded_databases.push(LoadedDatabase {
                            path: PathBuf::from("example.dbf"),
                            db_type: DatabaseType::Dbf,
                            message_count: 28,
                            signal_count: 89,
                        });
                    }
                    if ui.button("📂 Load A2L...").clicked() {
                        state.loaded_databases.push(LoadedDatabase {
                            path: PathBuf::from("example.a2l"),
                            db_type: DatabaseType::A2l,
                            message_count: 0,
                            signal_count: 512,
                        });
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("📂 Load LDF...").clicked() {
                        state.loaded_databases.push(LoadedDatabase {
                            path: PathBuf::from("example.ldf"),
                            db_type: DatabaseType::Ldf,
                            message_count: 15,
                            signal_count: 45,
                        });
                    }
                    if ui.button("📂 Load ARXML...").clicked() {
                        state.loaded_databases.push(LoadedDatabase {
                            path: PathBuf::from("example.arxml"),
                            db_type: DatabaseType::Arxml,
                            message_count: 120,
                            signal_count: 480,
                        });
                    }
                    if ui.button("📂 Load ODX...").clicked() {
                        state.loaded_databases.push(LoadedDatabase {
                            path: PathBuf::from("example.odx"),
                            db_type: DatabaseType::Odx,
                            message_count: 0,
                            signal_count: 0,
                        });
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                // Database list
                if state.loaded_databases.is_empty() {
                    ui.label("No databases loaded.");
                    ui.label("Load a database file to decode CAN messages and signals.");
                } else {
                    ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            let mut to_remove = None;
                            
                            egui::Grid::new("database_grid")
                                .num_columns(5)
                                .striped(true)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    // Header
                                    ui.strong("Type");
                                    ui.strong("File");
                                    ui.strong("Messages");
                                    ui.strong("Signals");
                                    ui.strong("Actions");
                                    ui.end_row();

                                    for (idx, db) in state.loaded_databases.iter().enumerate() {
                                        // Type with color
                                        let type_color = match db.db_type {
                                            DatabaseType::Dbc => Color32::from_rgb(100, 180, 255),
                                            DatabaseType::Dbf => Color32::from_rgb(255, 180, 100),
                                            DatabaseType::Ldf => Color32::from_rgb(180, 255, 100),
                                            DatabaseType::Arxml => Color32::from_rgb(255, 100, 180),
                                            DatabaseType::Odx => Color32::from_rgb(180, 100, 255),
                                            DatabaseType::A2l => Color32::from_rgb(100, 255, 180),
                                        };
                                        ui.colored_label(type_color, db.db_type.name());

                                        // File path
                                        ui.label(db.path.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "Unknown".to_string()));

                                        // Message count
                                        ui.label(format!("{}", db.message_count));

                                        // Signal count
                                        ui.label(format!("{}", db.signal_count));

                                        // Remove button
                                        if ui.small_button("🗑 Remove").clicked() {
                                            to_remove = Some(idx);
                                        }

                                        ui.end_row();
                                    }
                                });

                            if let Some(idx) = to_remove {
                                state.loaded_databases.remove(idx);
                            }
                        });
                }

                ui.add_space(8.0);
                ui.separator();

                // Summary
                let total_messages: usize = state.loaded_databases.iter().map(|d| d.message_count).sum();
                let total_signals: usize = state.loaded_databases.iter().map(|d| d.signal_count).sum();
                ui.horizontal(|ui| {
                    ui.label(format!("Total: {} databases, {} messages, {} signals",
                        state.loaded_databases.len(),
                        total_messages,
                        total_signals));
                });

                ui.add_space(8.0);

                // Close button
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        state.show_database_dialog = false;
                    }
                    if ui.button("Unload All").clicked() {
                        state.loaded_databases.clear();
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_colors() {
        // Just verify the types exist and have names
        assert_eq!(DatabaseType::Dbc.name(), "DBC (Vector)");
        assert_eq!(DatabaseType::Dbf.name(), "DBF (BUSMASTER)");
        assert_eq!(DatabaseType::A2l.name(), "A2L (XCP)");
    }
}
