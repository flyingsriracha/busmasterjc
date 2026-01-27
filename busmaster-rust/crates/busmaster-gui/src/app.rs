//! Main application struct

use eframe::Frame;
use egui::{Context, CentralPanel, SidePanel, TopBottomPanel};

use crate::panels::{DatabaseBrowserPanel, DiagnosticsPanel, MessageViewPanel, SignalGraphPanel, SignalWatchPanel, StatusBar, Toolbar};
use crate::state::{AppState, ActivePanel};
use crate::widgets::{AboutDialog, DatabaseDialog, FilterDialog, LoggingDialog, SettingsDialog};

/// Main BUSMASTER application
pub struct BusmasterApp {
    /// Application state
    state: AppState,
}

impl BusmasterApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up custom fonts if needed
        // cc.egui_ctx.set_fonts(...);

        // Load persisted state if available
        let state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::new()
        };

        // Apply theme
        cc.egui_ctx.set_visuals(state.theme.visuals());

        Self { state }
    }
}

impl eframe::App for BusmasterApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Persist state (but not messages)
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Apply theme changes
        ctx.set_visuals(self.state.theme.visuals());

        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);

        // Top toolbar
        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            Toolbar::show(ui, &mut self.state);
        });

        // Bottom status bar
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            StatusBar::show(ui, &self.state);
        });

        // Left panel: Database browser
        SidePanel::left("database_browser")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                DatabaseBrowserPanel::show(ui, &mut self.state);
            });

        // Right panel: Signal watch
        SidePanel::right("signal_watch")
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                SignalWatchPanel::show(ui, &mut self.state);
            });

        // Central panel: Main content area
        CentralPanel::default().show(ctx, |ui| {
            match self.state.active_panel {
                ActivePanel::Messages => MessageViewPanel::show(ui, &mut self.state),
                ActivePanel::SignalGraph => SignalGraphPanel::show(ui, &mut self.state),
                ActivePanel::Diagnostics => DiagnosticsPanel::show(ui, &mut self.state.diagnostics_state),
                ActivePanel::Calibration => self.show_calibration(ui),
            }
        });

        // Dialogs
        FilterDialog::show(ctx, &mut self.state);
        SettingsDialog::show(ctx, &mut self.state);
        DatabaseDialog::show(ctx, &mut self.state);
        LoggingDialog::show(ctx, &mut self.state);
        AboutDialog::show(ctx, &mut self.state.show_about_dialog);

        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}

impl BusmasterApp {
    fn handle_shortcuts(&mut self, ctx: &Context) {
        // Ctrl+Space: Connect/Disconnect
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Space)) {
            // Toggle connection
            use crate::state::ConnectionStatus;
            self.state.connection_status = match self.state.connection_status {
                ConnectionStatus::Disconnected => ConnectionStatus::Connecting,
                ConnectionStatus::Connected => ConnectionStatus::Disconnected,
                other => other,
            };
        }

        // Ctrl+L: Toggle logging
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::L)) {
            use crate::state::LoggingStatus;
            self.state.logging_status = match self.state.logging_status {
                LoggingStatus::Stopped => LoggingStatus::Recording,
                LoggingStatus::Recording => LoggingStatus::Stopped,
                LoggingStatus::Paused => LoggingStatus::Recording,
            };
        }

        // Ctrl+F: Open filter dialog
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F)) {
            self.state.show_filter_dialog = true;
        }

        // Ctrl+G: Switch to graph view
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G)) {
            self.state.active_panel = ActivePanel::SignalGraph;
        }

        // Ctrl+D: Switch to diagnostics view
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) {
            self.state.active_panel = ActivePanel::Diagnostics;
        }

        // Escape: Close dialogs
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.state.show_filter_dialog = false;
            self.state.show_settings_dialog = false;
            self.state.show_about_dialog = false;
            self.state.show_database_dialog = false;
            self.state.show_logging_dialog = false;
        }

        // Ctrl+O: Open database dialog
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            self.state.show_database_dialog = true;
        }
    }

    fn show_calibration(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙ Calibration");
        ui.separator();

        ui.label("XCP measurement and calibration panel");
        ui.label("");
        ui.label("Features planned:");
        ui.label("• Measurement display");
        ui.label("• Parameter editing");
        ui.label("• Curve/Map editors");
        ui.label("• Memory page management");
        ui.label("• Flash programming");
    }
}

// Implement Default for persistence
impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// Implement serde for persistence
impl serde::Serialize for AppState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppState", 8)?;
        state.serialize_field("theme", &(self.theme as u8))?;
        state.serialize_field("selected_driver", &self.selected_driver)?;
        state.serialize_field("selected_channel", &self.selected_channel)?;
        state.serialize_field("bitrate", &self.bitrate)?;
        state.serialize_field("max_messages", &self.max_messages)?;
        state.serialize_field("filter_id_start", &self.filter_id_start)?;
        state.serialize_field("filter_id_end", &self.filter_id_end)?;
        state.serialize_field("filter_show_tx", &self.filter_show_tx)?;
        state.serialize_field("filter_show_rx", &self.filter_show_rx)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for AppState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct AppStateData {
            theme: u8,
            selected_driver: String,
            selected_channel: u8,
            bitrate: u32,
            max_messages: usize,
            filter_id_start: u32,
            filter_id_end: u32,
            filter_show_tx: bool,
            filter_show_rx: bool,
        }

        let data = AppStateData::deserialize(deserializer)?;
        let mut state = AppState::new();
        state.theme = match data.theme {
            0 => crate::theme::Theme::Dark,
            1 => crate::theme::Theme::Light,
            2 => crate::theme::Theme::HighContrast,
            _ => crate::theme::Theme::Dark,
        };
        state.selected_driver = data.selected_driver;
        state.selected_channel = data.selected_channel;
        state.bitrate = data.bitrate;
        state.max_messages = data.max_messages;
        state.filter_id_start = data.filter_id_start;
        state.filter_id_end = data.filter_id_end;
        state.filter_show_tx = data.filter_show_tx;
        state.filter_show_rx = data.filter_show_rx;
        Ok(state)
    }
}
