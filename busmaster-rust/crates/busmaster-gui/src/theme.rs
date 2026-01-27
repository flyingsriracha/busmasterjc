//! Theme configuration for BUSMASTER GUI
//!
//! Provides dark mode (default), light mode, and high contrast themes.

use egui::{Color32, Visuals};

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Dark theme (default) - easy on the eyes for long sessions
    #[default]
    Dark,
    /// Light theme
    Light,
    /// High contrast for accessibility
    HighContrast,
}

impl Theme {
    /// Get the egui visuals for this theme
    pub fn visuals(&self) -> Visuals {
        match self {
            Theme::Dark => {
                let mut visuals = Visuals::dark();
                // Customize for automotive professional look
                visuals.panel_fill = Color32::from_rgb(30, 30, 35);
                visuals.window_fill = Color32::from_rgb(35, 35, 40);
                visuals.extreme_bg_color = Color32::from_rgb(20, 20, 25);
                visuals
            }
            Theme::Light => Visuals::light(),
            Theme::HighContrast => {
                let mut visuals = Visuals::dark();
                visuals.panel_fill = Color32::BLACK;
                visuals.window_fill = Color32::BLACK;
                visuals.override_text_color = Some(Color32::WHITE);
                visuals
            }
        }
    }

    /// Get the name of this theme
    pub fn name(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::HighContrast => "High Contrast",
        }
    }
}

/// Color palette for message types
pub struct MessageColors;

impl MessageColors {
    /// TX message color (blue)
    pub const TX: Color32 = Color32::from_rgb(100, 149, 237);
    /// RX message color (green)
    pub const RX: Color32 = Color32::from_rgb(144, 238, 144);
    /// Error message color (red)
    pub const ERROR: Color32 = Color32::from_rgb(255, 99, 71);
    /// Warning color (yellow)
    pub const WARNING: Color32 = Color32::from_rgb(255, 215, 0);
    /// Info color (cyan)
    #[allow(dead_code)]
    pub const INFO: Color32 = Color32::from_rgb(0, 191, 255);
}

/// Color palette for signal values
pub struct SignalColors;

impl SignalColors {
    /// Signal value changed recently
    #[allow(dead_code)]
    pub const CHANGED: Color32 = Color32::from_rgb(255, 255, 100);
    /// Signal value normal
    pub const NORMAL: Color32 = Color32::from_rgb(200, 200, 200);
    /// Signal value at minimum
    pub const MIN: Color32 = Color32::from_rgb(100, 149, 237);
    /// Signal value at maximum
    pub const MAX: Color32 = Color32::from_rgb(255, 99, 71);
}

/// Color palette for database browser icons
pub struct DatabaseColors;

impl DatabaseColors {
    /// Package/folder color
    #[allow(dead_code)]
    pub const PACKAGE: Color32 = Color32::from_rgb(255, 193, 7);
    /// Message/frame color
    pub const MESSAGE: Color32 = Color32::from_rgb(33, 150, 243);
    /// Signal color
    pub const SIGNAL: Color32 = Color32::from_rgb(76, 175, 80);
    /// Parameter color
    #[allow(dead_code)]
    pub const PARAMETER: Color32 = Color32::from_rgb(156, 39, 176);
    /// Node/ECU color
    #[allow(dead_code)]
    pub const NODE: Color32 = Color32::from_rgb(255, 87, 34);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        assert_eq!(Theme::default(), Theme::Dark);
    }

    #[test]
    fn test_theme_names() {
        assert_eq!(Theme::Dark.name(), "Dark");
        assert_eq!(Theme::Light.name(), "Light");
        assert_eq!(Theme::HighContrast.name(), "High Contrast");
    }

    #[test]
    fn test_theme_visuals() {
        // Just verify they don't panic
        let _ = Theme::Dark.visuals();
        let _ = Theme::Light.visuals();
        let _ = Theme::HighContrast.visuals();
    }
}
