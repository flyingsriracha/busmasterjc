//! Signal graphing panel
//!
//! Real-time time-series plotting of signal values using egui_plot.

use std::collections::{HashMap, VecDeque};
use egui::{Ui, RichText, Color32};
use egui_plot::{Plot, Line, PlotPoints, Legend, Corner};

use crate::state::AppState;

/// Maximum number of data points to keep per signal
const MAX_DATA_POINTS: usize = 10000;

/// Default time window in seconds
const DEFAULT_TIME_WINDOW: f64 = 10.0;

/// Signal data point
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    /// Timestamp in seconds (relative to start)
    pub time: f64,
    /// Signal value
    pub value: f64,
}

/// Data series for a single signal
#[derive(Debug, Clone)]
pub struct SignalSeries {
    /// Signal name
    pub name: String,
    /// Message ID this signal belongs to
    pub message_id: u32,
    /// Data points (time, value)
    pub points: VecDeque<DataPoint>,
    /// Color for this series
    pub color: Color32,
    /// Whether this series is visible
    pub visible: bool,
    /// Y-axis: true = right, false = left
    pub use_right_axis: bool,
}

impl SignalSeries {
    /// Create a new signal series
    pub fn new(name: String, message_id: u32, color: Color32) -> Self {
        Self {
            name,
            message_id,
            points: VecDeque::with_capacity(MAX_DATA_POINTS),
            color,
            visible: true,
            use_right_axis: false,
        }
    }

    /// Add a data point
    pub fn add_point(&mut self, time: f64, value: f64) {
        self.points.push_back(DataPoint { time, value });
        while self.points.len() > MAX_DATA_POINTS {
            self.points.pop_front();
        }
    }

    /// Get the plot points for egui_plot
    pub fn plot_points(&self) -> PlotPoints {
        PlotPoints::from_iter(
            self.points.iter().map(|p| [p.time, p.value])
        )
    }

    /// Get min/max values
    pub fn value_range(&self) -> Option<(f64, f64)> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for p in &self.points {
            min = min.min(p.value);
            max = max.max(p.value);
        }
        Some((min, max))
    }
}

/// Graph state
#[derive(Debug, Clone)]
pub struct GraphState {
    /// Signal series data
    pub series: HashMap<String, SignalSeries>,
    /// Start time (for relative timestamps)
    pub start_time_us: u64,
    /// Time window to display (seconds)
    pub time_window: f64,
    /// Auto-scroll (follow latest data)
    pub auto_scroll: bool,
    /// Show legend
    pub show_legend: bool,
    /// Show grid
    pub show_grid: bool,
    /// Paused
    pub paused: bool,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            series: HashMap::new(),
            start_time_us: 0,
            time_window: DEFAULT_TIME_WINDOW,
            auto_scroll: true,
            show_legend: true,
            show_grid: true,
            paused: false,
        }
    }
}

impl GraphState {
    /// Add or update a signal series
    pub fn add_signal(&mut self, name: &str, message_id: u32) {
        let key = format!("{}_{:X}", name, message_id);
        if !self.series.contains_key(&key) {
            // Assign a color based on the number of series
            let color = series_color(self.series.len());
            self.series.insert(
                key,
                SignalSeries::new(name.to_string(), message_id, color),
            );
        }
    }

    /// Remove a signal series
    pub fn remove_signal(&mut self, name: &str, message_id: u32) {
        let key = format!("{}_{:X}", name, message_id);
        self.series.remove(&key);
    }

    /// Add a data point to a signal
    pub fn add_data_point(&mut self, name: &str, message_id: u32, timestamp_us: u64, value: f64) {
        if self.paused {
            return;
        }

        // Initialize start time on first data point
        if self.start_time_us == 0 {
            self.start_time_us = timestamp_us;
        }

        let key = format!("{}_{:X}", name, message_id);
        if let Some(series) = self.series.get_mut(&key) {
            let time = (timestamp_us - self.start_time_us) as f64 / 1_000_000.0;
            series.add_point(time, value);
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        for series in self.series.values_mut() {
            series.points.clear();
        }
        self.start_time_us = 0;
    }

    /// Get the current time range for display
    pub fn time_range(&self) -> (f64, f64) {
        if self.series.is_empty() {
            return (0.0, self.time_window);
        }

        // Find the latest time across all series
        let mut max_time = 0.0_f64;
        for series in self.series.values() {
            if let Some(last) = series.points.back() {
                max_time = max_time.max(last.time);
            }
        }

        if self.auto_scroll {
            // Show the last time_window seconds
            let end = max_time;
            let start = (end - self.time_window).max(0.0);
            (start, end)
        } else {
            // Fixed window from 0
            (0.0, self.time_window)
        }
    }
}

/// Get a color for a series based on its index
fn series_color(index: usize) -> Color32 {
    const COLORS: &[Color32] = &[
        Color32::from_rgb(66, 133, 244),   // Blue
        Color32::from_rgb(234, 67, 53),    // Red
        Color32::from_rgb(251, 188, 5),    // Yellow
        Color32::from_rgb(52, 168, 83),    // Green
        Color32::from_rgb(255, 109, 0),    // Orange
        Color32::from_rgb(156, 39, 176),   // Purple
        Color32::from_rgb(0, 188, 212),    // Cyan
        Color32::from_rgb(255, 87, 34),    // Deep Orange
        Color32::from_rgb(103, 58, 183),   // Deep Purple
        Color32::from_rgb(0, 150, 136),    // Teal
    ];
    COLORS[index % COLORS.len()]
}

/// Signal graph panel
pub struct SignalGraphPanel;

impl SignalGraphPanel {
    /// Render the signal graph panel
    pub fn show(ui: &mut Ui, state: &mut AppState) {
        // Header with controls
        ui.horizontal(|ui| {
            ui.heading("📈 Signal Graph");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Pause/Resume button
                let pause_text = if state.graph_state.paused { "▶ Resume" } else { "⏸ Pause" };
                if ui.button(pause_text).clicked() {
                    state.graph_state.paused = !state.graph_state.paused;
                }

                // Clear button
                if ui.button("🗑 Clear").clicked() {
                    state.graph_state.clear();
                }

                // Auto-scroll toggle
                ui.checkbox(&mut state.graph_state.auto_scroll, "Auto-scroll");

                // Time window selector
                ui.label("Window:");
                egui::ComboBox::from_id_salt("time_window")
                    .selected_text(format!("{:.0}s", state.graph_state.time_window))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.graph_state.time_window, 5.0, "5s");
                        ui.selectable_value(&mut state.graph_state.time_window, 10.0, "10s");
                        ui.selectable_value(&mut state.graph_state.time_window, 30.0, "30s");
                        ui.selectable_value(&mut state.graph_state.time_window, 60.0, "60s");
                        ui.selectable_value(&mut state.graph_state.time_window, 300.0, "5min");
                    });
            });
        });

        ui.separator();

        // Check if we have any signals to graph
        if state.graph_state.series.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(RichText::new("No signals being graphed").size(18.0));
                    ui.add_space(10.0);
                    ui.label("Add signals from the Signal Watch panel");
                    ui.label("or drag signals from the Database Browser");
                });
            });
            return;
        }

        // Signal legend/controls on the left
        ui.horizontal(|ui| {
            // Left side: signal list
            ui.vertical(|ui| {
                ui.set_min_width(150.0);
                ui.set_max_width(200.0);
                
                ui.label(RichText::new("Signals").strong());
                ui.separator();

                let series_keys: Vec<_> = state.graph_state.series.keys().cloned().collect();
                for key in series_keys {
                    if let Some(series) = state.graph_state.series.get_mut(&key) {
                        ui.horizontal(|ui| {
                            // Color indicator
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(12.0, 12.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 2.0, series.color);

                            // Visibility checkbox
                            ui.checkbox(&mut series.visible, "");

                            // Signal name
                            ui.label(&series.name);

                            // Current value
                            if let Some(last) = series.points.back() {
                                ui.label(RichText::new(format!("{:.2}", last.value)).weak());
                            }
                        });
                    }
                }
            });

            ui.separator();

            // Right side: the plot
            ui.vertical(|ui| {
                let (time_start, time_end) = state.graph_state.time_range();

                let plot = Plot::new("signal_plot")
                    .legend(Legend::default().position(Corner::RightTop))
                    .x_axis_label("Time (s)")
                    .y_axis_label("Value")
                    .show_axes(true)
                    .show_grid(state.graph_state.show_grid)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .allow_scroll(true)
                    .include_x(time_start)
                    .include_x(time_end);

                plot.show(ui, |plot_ui| {
                    for series in state.graph_state.series.values() {
                        if series.visible && !series.points.is_empty() {
                            let line = Line::new(series.plot_points())
                                .name(&series.name)
                                .color(series.color)
                                .width(1.5);
                            plot_ui.line(line);
                        }
                    }
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_series_new() {
        let series = SignalSeries::new("TestSignal".to_string(), 0x100, Color32::RED);
        assert_eq!(series.name, "TestSignal");
        assert_eq!(series.message_id, 0x100);
        assert!(series.points.is_empty());
        assert!(series.visible);
    }

    #[test]
    fn test_signal_series_add_point() {
        let mut series = SignalSeries::new("Test".to_string(), 0x100, Color32::RED);
        series.add_point(0.0, 10.0);
        series.add_point(1.0, 20.0);
        series.add_point(2.0, 30.0);

        assert_eq!(series.points.len(), 3);
        assert_eq!(series.points[0].value, 10.0);
        assert_eq!(series.points[2].value, 30.0);
    }

    #[test]
    fn test_signal_series_value_range() {
        let mut series = SignalSeries::new("Test".to_string(), 0x100, Color32::RED);
        assert!(series.value_range().is_none());

        series.add_point(0.0, 10.0);
        series.add_point(1.0, 50.0);
        series.add_point(2.0, 30.0);

        let (min, max) = series.value_range().unwrap();
        assert_eq!(min, 10.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_graph_state_add_signal() {
        let mut state = GraphState::default();
        state.add_signal("EngineSpeed", 0x100);
        state.add_signal("VehicleSpeed", 0x200);

        assert_eq!(state.series.len(), 2);
        assert!(state.series.contains_key("EngineSpeed_100"));
        assert!(state.series.contains_key("VehicleSpeed_200"));
    }

    #[test]
    fn test_graph_state_add_data_point() {
        let mut state = GraphState::default();
        state.add_signal("EngineSpeed", 0x100);
        
        state.add_data_point("EngineSpeed", 0x100, 1000000, 1500.0);
        state.add_data_point("EngineSpeed", 0x100, 2000000, 2000.0);

        let series = state.series.get("EngineSpeed_100").unwrap();
        assert_eq!(series.points.len(), 2);
        assert_eq!(series.points[0].time, 0.0);
        assert_eq!(series.points[1].time, 1.0);
    }

    #[test]
    fn test_graph_state_time_range() {
        let mut state = GraphState {
            time_window: 10.0,
            auto_scroll: true,
            ..Default::default()
        };

        state.add_signal("Test", 0x100);
        
        // Add points spanning 15 seconds (0 to 15)
        // First point sets start_time_us
        for i in 0..16 {
            state.add_data_point("Test", 0x100, i * 1_000_000, i as f64);
        }

        let (start, end) = state.time_range();
        // Last point is at time 15.0 (15_000_000 - 0) / 1_000_000
        // With auto_scroll and 10s window, should show 5.0 to 15.0
        assert!(end >= 14.0, "end should be >= 14.0, got {}", end);
        assert!(start >= 4.0, "start should be >= 4.0, got {}", start);
    }

    #[test]
    fn test_graph_state_paused() {
        let mut state = GraphState::default();
        state.add_signal("Test", 0x100);
        state.paused = true;

        state.add_data_point("Test", 0x100, 1000000, 100.0);

        let series = state.series.get("Test_100").unwrap();
        assert!(series.points.is_empty()); // Should not add when paused
    }

    #[test]
    fn test_series_color() {
        // Should cycle through colors
        let c0 = series_color(0);
        let c1 = series_color(1);
        let c10 = series_color(10);

        assert_ne!(c0, c1);
        assert_eq!(c0, c10); // Should wrap around
    }
}
