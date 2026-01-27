//! GUI Panels
//!
//! Each panel represents a major UI component in the BUSMASTER application.

mod database_browser;
pub mod diagnostics;
mod message_view;
pub mod signal_graph;
mod signal_watch;
mod status_bar;
mod toolbar;

pub use database_browser::DatabaseBrowserPanel;
pub use diagnostics::DiagnosticsPanel;
pub use message_view::MessageViewPanel;
pub use signal_graph::SignalGraphPanel;
pub use signal_watch::SignalWatchPanel;
pub use status_bar::StatusBar;
pub use toolbar::Toolbar;
