//! UI Tests for BUSMASTER GUI
//!
//! These tests verify the behavior of UI components and state management.

#[cfg(test)]
mod ui_tests {
    use crate::panels::diagnostics::{
        DiagSession, DiagnosticsState, DidResult, DtcEntry, SecurityState, ServiceLogEntry,
    };
    use crate::panels::signal_graph::{GraphState, SignalSeries};
    use crate::state::{
        ActivePanel, AppState, ConnectionStatus, DatabaseType, LoadedDatabase, LoggingStatus,
        ReceivedMessage, WatchedSignal,
    };
    use crate::theme::Theme;
    use crate::widgets::{LogFormat, LogTrigger, LoggingConfig};
    use busmaster_core::CanFrame;
    use egui::Color32;
    use std::path::PathBuf;

    // ==================== AppState Tests ====================

    #[test]
    fn test_app_state_initialization() {
        let state = AppState::new();

        // Verify defaults
        assert_eq!(state.theme, Theme::Dark);
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(state.logging_status, LoggingStatus::Stopped);
        assert_eq!(state.active_panel, ActivePanel::Messages);
        assert!(state.messages.is_empty());
        assert!(state.watched_signals.is_empty());
        assert!(state.loaded_databases.is_empty());
        assert_eq!(state.bitrate, 500_000);
        assert!(!state.show_filter_dialog);
        assert!(!state.show_settings_dialog);
        assert!(!state.show_about_dialog);
        assert!(!state.show_database_dialog);
        assert!(!state.show_logging_dialog);
    }

    #[test]
    fn test_app_state_message_buffer_limit() {
        let mut state = AppState::new();
        state.max_messages = 5;

        // Add more messages than the limit
        for i in 0..10 {
            state.add_message(ReceivedMessage {
                timestamp_us: i as u64,
                channel: 0,
                frame: CanFrame::new_standard(0x100 + i, &[i as u8]).unwrap(),
                is_tx: false,
                name: None,
            });
        }

        // Should only keep max_messages
        assert_eq!(state.messages.len(), 5);
        // Most recent should be first (timestamp 9)
        assert_eq!(state.messages[0].timestamp_us, 9);
        // Oldest should be last (timestamp 5)
        assert_eq!(state.messages[4].timestamp_us, 5);
    }


    #[test]
    fn test_app_state_signal_watch_no_duplicates() {
        let mut state = AppState::new();

        let signal = WatchedSignal {
            name: "Speed".to_string(),
            message_name: "VehicleData".to_string(),
            message_id: 0x200,
            value: 0.0,
            unit: "km/h".to_string(),
            min_value: 0.0,
            max_value: 0.0,
            last_update_us: 0,
        };

        state.add_watched_signal(signal.clone());
        state.add_watched_signal(signal.clone());
        state.add_watched_signal(signal);

        // Should only have one entry
        assert_eq!(state.watched_signals.len(), 1);
    }

    #[test]
    fn test_app_state_signal_update_min_max() {
        let mut state = AppState::new();

        state.add_watched_signal(WatchedSignal {
            name: "Temperature".to_string(),
            message_name: "SensorData".to_string(),
            message_id: 0x300,
            value: 25.0,
            unit: "C".to_string(),
            min_value: 25.0,
            max_value: 25.0,
            last_update_us: 0,
        });

        // Update with lower value
        state.update_signal(0x300, "Temperature", 10.0, 1000);
        assert_eq!(state.watched_signals[0].value, 10.0);
        assert_eq!(state.watched_signals[0].min_value, 10.0);
        assert_eq!(state.watched_signals[0].max_value, 25.0);

        // Update with higher value
        state.update_signal(0x300, "Temperature", 50.0, 2000);
        assert_eq!(state.watched_signals[0].value, 50.0);
        assert_eq!(state.watched_signals[0].min_value, 10.0);
        assert_eq!(state.watched_signals[0].max_value, 50.0);
    }

    // ==================== Theme Tests ====================

    #[test]
    fn test_theme_variants() {
        assert_eq!(Theme::Dark.name(), "Dark");
        assert_eq!(Theme::Light.name(), "Light");
        assert_eq!(Theme::HighContrast.name(), "High Contrast");
    }

    #[test]
    fn test_theme_visuals_differ() {
        let dark = Theme::Dark.visuals();
        let light = Theme::Light.visuals();

        // Dark and light themes should have different backgrounds
        assert_ne!(dark.panel_fill, light.panel_fill);
    }

    // ==================== DatabaseType Tests ====================

    #[test]
    fn test_database_type_extensions() {
        assert_eq!(DatabaseType::Dbc.extension(), "dbc");
        assert_eq!(DatabaseType::Dbf.extension(), "dbf");
        assert_eq!(DatabaseType::Ldf.extension(), "ldf");
        assert_eq!(DatabaseType::Arxml.extension(), "arxml");
        assert_eq!(DatabaseType::Odx.extension(), "odx");
        assert_eq!(DatabaseType::A2l.extension(), "a2l");
    }

    #[test]
    fn test_database_type_names() {
        assert!(DatabaseType::Dbc.name().contains("Vector"));
        assert!(DatabaseType::Dbf.name().contains("BUSMASTER"));
        assert!(DatabaseType::Ldf.name().contains("LIN"));
        assert!(DatabaseType::Arxml.name().contains("AUTOSAR"));
        assert!(DatabaseType::Odx.name().contains("Diagnostics"));
        assert!(DatabaseType::A2l.name().contains("XCP"));
    }


    // ==================== Diagnostics Tests ====================

    #[test]
    fn test_diagnostics_session_values() {
        assert_eq!(DiagSession::Default.value(), 0x01);
        assert_eq!(DiagSession::Programming.value(), 0x02);
        assert_eq!(DiagSession::Extended.value(), 0x03);
        assert_eq!(DiagSession::SafetySystem.value(), 0x04);
    }

    #[test]
    fn test_diagnostics_security_state_colors() {
        // Just verify colors are different for different states
        let locked_color = SecurityState::Locked.color();
        let unlocked_color = SecurityState::Unlocked.color();
        assert_ne!(locked_color, unlocked_color);
    }

    #[test]
    fn test_dtc_code_formatting() {
        // Powertrain codes (P)
        let dtc = DtcEntry::new(0x0123, 0x08);
        assert!(dtc.code.starts_with('P'));

        // Chassis codes (C) - bit 14 set
        let dtc = DtcEntry::new(0x4123, 0x08);
        assert!(dtc.code.starts_with('C'));

        // Body codes (B) - bit 15 set
        let dtc = DtcEntry::new(0x8123, 0x08);
        assert!(dtc.code.starts_with('B'));

        // Network codes (U) - bits 14 and 15 set
        let dtc = DtcEntry::new(0xC123, 0x08);
        assert!(dtc.code.starts_with('U'));
    }

    #[test]
    fn test_dtc_status_flags() {
        // Confirmed DTC (bit 3)
        let dtc = DtcEntry::new(0x0100, 0x08);
        assert!(dtc.confirmed);
        assert!(!dtc.pending);

        // Pending DTC (bit 2)
        let dtc = DtcEntry::new(0x0100, 0x04);
        assert!(!dtc.confirmed);
        assert!(dtc.pending);

        // Both confirmed and pending
        let dtc = DtcEntry::new(0x0100, 0x0C);
        assert!(dtc.confirmed);
        assert!(dtc.pending);
    }

    #[test]
    fn test_did_result_vin_interpretation() {
        let vin_data = b"WVWZZZ3CZWE123456".to_vec();
        let result = DidResult::new(0xF190, vin_data);

        assert_eq!(result.did, 0xF190);
        assert!(result.name.contains("VIN"));
        assert_eq!(result.value, "WVWZZZ3CZWE123456");
    }

    #[test]
    fn test_service_log_entry() {
        let tx_entry = ServiceLogEntry::tx("DiagnosticSessionControl", vec![0x10, 0x03]);
        assert_eq!(tx_entry.direction, "TX");
        assert!(!tx_entry.is_error);
        assert_eq!(tx_entry.bytes, vec![0x10, 0x03]);

        let rx_entry = ServiceLogEntry::rx("NegativeResponse", vec![0x7F, 0x10, 0x22], true);
        assert_eq!(rx_entry.direction, "RX");
        assert!(rx_entry.is_error);
    }


    // ==================== Signal Graph Tests ====================

    #[test]
    fn test_graph_state_signal_management() {
        let mut state = GraphState::default();

        state.add_signal("Signal1", 0x100);
        state.add_signal("Signal2", 0x200);

        assert_eq!(state.series.len(), 2);

        // Add duplicate - should not increase count
        state.add_signal("Signal1", 0x100);
        assert_eq!(state.series.len(), 2);
    }

    #[test]
    fn test_graph_state_data_points() {
        let mut state = GraphState::default();
        state.add_signal("TestSignal", 0x100);

        // Add data points
        for i in 0..100 {
            state.add_data_point("TestSignal", 0x100, i as u64 * 100_000, i as f64);
        }

        // Verify data was added
        let key = "TestSignal_100";
        let series = state.series.get(key).unwrap();
        assert_eq!(series.points.len(), 100);
    }

    #[test]
    fn test_signal_series_ring_buffer() {
        let mut series = SignalSeries::new("Test".to_string(), 0x100, Color32::RED);

        // Add points (under the MAX_DATA_POINTS limit of 10000)
        for i in 0..100 {
            series.add_point(i as f64, i as f64);
        }

        // Should have all 100 points
        assert_eq!(series.points.len(), 100);
        // First point should be 0
        assert_eq!(series.points[0].value, 0.0);
        // Last point should be 99
        assert_eq!(series.points[99].value, 99.0);
    }

    // ==================== Logging Config Tests ====================

    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::new();

        assert_eq!(config.format, LogFormat::Asc);
        assert!(config.auto_filename);
        assert_eq!(config.trigger, LogTrigger::Manual);
        assert_eq!(config.max_file_size_mb, 100);
        assert!(config.include_timestamps);
        assert!(config.include_signals);
    }

    #[test]
    fn test_logging_config_filename_generation() {
        let config = LoggingConfig::new();
        let filename = config.generate_filename();

        assert!(filename.starts_with("busmaster_log_"));
        assert!(filename.ends_with(".asc"));
    }

    #[test]
    fn test_log_format_extensions() {
        assert_eq!(LogFormat::Asc.extension(), "asc");
        assert_eq!(LogFormat::Blf.extension(), "blf");
        assert_eq!(LogFormat::Pcap.extension(), "pcap");
        assert_eq!(LogFormat::Csv.extension(), "csv");
    }


    // ==================== Connection Status Tests ====================

    #[test]
    fn test_connection_status_text() {
        assert_eq!(ConnectionStatus::Disconnected.text(), "Disconnected");
        assert_eq!(ConnectionStatus::Connecting.text(), "Connecting...");
        assert_eq!(ConnectionStatus::Connected.text(), "Connected");
        assert_eq!(ConnectionStatus::Error.text(), "Error");
    }

    // ==================== Active Panel Tests ====================

    #[test]
    fn test_active_panel_default() {
        let panel = ActivePanel::default();
        assert_eq!(panel, ActivePanel::Messages);
    }

    // ==================== DiagnosticsState Tests ====================

    #[test]
    fn test_diagnostics_state_new() {
        let state = DiagnosticsState::new();
        assert_eq!(state.session, DiagSession::Default);
        assert_eq!(state.security_state, SecurityState::Locked);
        assert_eq!(state.security_level, 1);
        assert!(state.dtc_list.is_empty());
        assert!(state.did_results.is_empty());
        assert!(state.service_log.is_empty());
        assert!(!state.is_connected);
    }

    #[test]
    fn test_diagnostics_state_log_service() {
        let mut state = DiagnosticsState::new();
        state.max_log_entries = 3;

        // Add entries
        for i in 0..5 {
            state.log_service(ServiceLogEntry::tx(&format!("Service{}", i), vec![i as u8]));
        }

        // Should only keep max_log_entries
        assert_eq!(state.service_log.len(), 3);
        // Should have the most recent entries
        assert_eq!(state.service_log[0].service, "Service2");
        assert_eq!(state.service_log[2].service, "Service4");
    }

    #[test]
    fn test_diagnostics_state_clear_log() {
        let mut state = DiagnosticsState::new();
        state.log_service(ServiceLogEntry::tx("Test", vec![0x10]));
        state.log_service(ServiceLogEntry::rx("Response", vec![0x50], false));

        assert_eq!(state.service_log.len(), 2);

        state.clear_log();
        assert!(state.service_log.is_empty());
    }


    // ==================== Integration Tests ====================

    #[test]
    fn test_full_workflow_simulation() {
        let mut state = AppState::new();

        // 1. Load a database
        state.loaded_databases.push(LoadedDatabase {
            path: PathBuf::from("test.dbc"),
            db_type: DatabaseType::Dbc,
            message_count: 10,
            signal_count: 50,
        });
        assert_eq!(state.loaded_databases.len(), 1);

        // 2. Add signals to watch
        state.add_watched_signal(WatchedSignal {
            name: "EngineRPM".to_string(),
            message_name: "EngineData".to_string(),
            message_id: 0x100,
            value: 0.0,
            unit: "rpm".to_string(),
            min_value: 0.0,
            max_value: 0.0,
            last_update_us: 0,
        });

        // 3. Simulate connection
        state.connection_status = ConnectionStatus::Connected;

        // 4. Receive messages
        for i in 0..100 {
            state.add_message(ReceivedMessage {
                timestamp_us: i * 1000,
                channel: 0,
                frame: CanFrame::new_standard(0x100, &[0, 0, (i % 256) as u8, 0, 0, 0, 0, 0])
                    .unwrap(),
                is_tx: false,
                name: Some("EngineData".to_string()),
            });

            // Update signal value
            state.update_signal(0x100, "EngineRPM", (i * 100) as f64, i * 1000);
        }

        // 5. Start logging
        state.logging_status = LoggingStatus::Recording;

        // Verify state
        assert_eq!(state.connection_status, ConnectionStatus::Connected);
        assert_eq!(state.logging_status, LoggingStatus::Recording);
        assert_eq!(state.messages.len(), 100);
        assert_eq!(state.watched_signals[0].value, 9900.0);
        assert_eq!(state.watched_signals[0].max_value, 9900.0);
    }
}
