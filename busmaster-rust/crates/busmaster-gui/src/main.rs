//! BUSMASTER GUI Application
//!
//! Professional automotive bus monitoring and analysis tool.

use busmaster_gui::BusmasterApp;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("BUSMASTER")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "BUSMASTER",
        options,
        Box::new(|cc| Ok(Box::new(BusmasterApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    // Return a simple default icon
    // TODO: Load actual BUSMASTER icon
    egui::IconData::default()
}
