//! About dialog
//!
//! Shows application information, version, and credits.

use egui::{Context, Window, Color32, RichText};

/// About dialog
pub struct AboutDialog;

impl AboutDialog {
    /// Application version
    pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    
    /// Show the about dialog if enabled
    pub fn show(ctx: &Context, show: &mut bool) {
        if !*show {
            return;
        }

        Window::new("About BUSMASTER")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    
                    // Logo/Title
                    ui.label(RichText::new("🚗 BUSMASTER")
                        .size(32.0)
                        .color(Color32::from_rgb(100, 180, 255)));
                    
                    ui.add_space(8.0);
                    
                    ui.label(RichText::new("Automotive Bus Monitoring Tool")
                        .size(16.0));
                    
                    ui.add_space(16.0);
                    
                    // Version info
                    ui.label(format!("Version: {}", Self::VERSION));
                    ui.label("Rust Edition");
                    
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Features
                    ui.label(RichText::new("Features").strong());
                    ui.add_space(4.0);
                    
                    let features = [
                        "• CAN / CAN FD / LIN / J1939 protocols",
                        "• UDS / OBD-II / XCP diagnostics",
                        "• DBC / DBF / LDF / ARXML / A2L parsers",
                        "• ASC / BLF / PCAP logging",
                        "• Real-time signal graphing",
                        "• Cross-platform (macOS, Linux, Windows)",
                    ];
                    
                    for feature in features {
                        ui.label(feature);
                    }
                    
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Credits
                    ui.label(RichText::new("Credits").strong());
                    ui.add_space(4.0);
                    ui.label("Original BUSMASTER by RBEI");
                    ui.label("Rust port by JC");
                    ui.label("Built with egui");
                    
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // License
                    ui.label(RichText::new("License").strong());
                    ui.add_space(4.0);
                    ui.label("LGPL-3.0");
                    ui.hyperlink_to("GitHub Repository", "https://github.com/AeroStun/busmaster");
                    
                    ui.add_space(16.0);
                    
                    // System info
                    ui.collapsing("System Information", |ui| {
                        ui.label(format!("OS: {}", std::env::consts::OS));
                        ui.label(format!("Arch: {}", std::env::consts::ARCH));
                        ui.label(format!("Rust: {}", rustc_version()));
                        #[cfg(debug_assertions)]
                        ui.label("Build: Debug");
                        #[cfg(not(debug_assertions))]
                        ui.label("Build: Release");
                    });
                    
                    ui.add_space(16.0);
                    
                    if ui.button("Close").clicked() {
                        *show = false;
                    }
                });
            });
    }
}

/// Get rustc version (compile-time)
fn rustc_version() -> &'static str {
    // This is set at compile time
    option_env!("RUSTC_VERSION").unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // Version should be set from Cargo.toml
        assert!(!AboutDialog::VERSION.is_empty());
    }
}
