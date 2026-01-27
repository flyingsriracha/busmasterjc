//! Diagnostics panel for UDS/OBD-II operations
//!
//! Provides a comprehensive interface for automotive diagnostics including:
//! - Session management (Default, Extended, Programming)
//! - Security access (seed-key authentication)
//! - DTC reading and clearing
//! - Data identifier read/write
//! - Routine control
//! - ECU reset

use egui::{Color32, ScrollArea, Ui};

/// Diagnostics panel state
#[derive(Debug, Clone, Default)]
pub struct DiagnosticsState {
    /// Current diagnostic session
    pub session: DiagSession,
    /// Security access state
    pub security_state: SecurityState,
    /// Current security level
    pub security_level: u8,
    /// Seed received from ECU
    pub seed: Vec<u8>,
    /// Key to send
    pub key_input: String,
    /// ECU address (request ID)
    pub ecu_request_id: String,
    /// ECU address (response ID)
    pub ecu_response_id: String,
    /// Selected tab
    pub active_tab: DiagTab,
    /// DTC list
    pub dtc_list: Vec<DtcEntry>,
    /// DTC status filter
    pub dtc_status_filter: u8,
    /// DID to read
    pub did_input: String,
    /// DID read results
    pub did_results: Vec<DidResult>,
    /// DID write value (hex)
    pub did_write_value: String,
    /// Routine ID
    pub routine_id: String,
    /// Routine option record (hex)
    pub routine_option: String,
    /// Routine results
    pub routine_results: Vec<RoutineResult>,
    /// Service log
    pub service_log: Vec<ServiceLogEntry>,
    /// Max log entries
    pub max_log_entries: usize,
    /// Is connected to ECU
    pub is_connected: bool,
    /// Last error message
    pub last_error: Option<String>,
}

impl DiagnosticsState {
    /// Create new diagnostics state
    pub fn new() -> Self {
        Self {
            session: DiagSession::Default,
            security_state: SecurityState::Locked,
            security_level: 1,
            seed: Vec::new(),
            key_input: String::new(),
            ecu_request_id: "0x7E0".to_string(),
            ecu_response_id: "0x7E8".to_string(),
            active_tab: DiagTab::Session,
            dtc_list: Vec::new(),
            dtc_status_filter: 0xFF,
            did_input: "F190".to_string(), // VIN by default
            did_results: Vec::new(),
            did_write_value: String::new(),
            routine_id: String::new(),
            routine_option: String::new(),
            routine_results: Vec::new(),
            service_log: Vec::new(),
            max_log_entries: 100,
            is_connected: false,
            last_error: None,
        }
    }

    /// Add entry to service log
    pub fn log_service(&mut self, entry: ServiceLogEntry) {
        self.service_log.push(entry);
        while self.service_log.len() > self.max_log_entries {
            self.service_log.remove(0);
        }
    }

    /// Clear service log
    pub fn clear_log(&mut self) {
        self.service_log.clear();
    }
}

/// Diagnostic session type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagSession {
    #[default]
    Default,
    Extended,
    Programming,
    SafetySystem,
}

impl DiagSession {
    /// Get session name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Default => "Default (0x01)",
            Self::Extended => "Extended (0x03)",
            Self::Programming => "Programming (0x02)",
            Self::SafetySystem => "Safety System (0x04)",
        }
    }

    /// Get session byte value
    pub fn value(&self) -> u8 {
        match self {
            Self::Default => 0x01,
            Self::Extended => 0x03,
            Self::Programming => 0x02,
            Self::SafetySystem => 0x04,
        }
    }
}

/// Security access state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecurityState {
    #[default]
    Locked,
    SeedRequested,
    Unlocked,
}

impl SecurityState {
    /// Get state name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Locked => "Locked",
            Self::SeedRequested => "Seed Requested",
            Self::Unlocked => "Unlocked",
        }
    }

    /// Get state color
    pub fn color(&self) -> Color32 {
        match self {
            Self::Locked => Color32::from_rgb(200, 80, 80),
            Self::SeedRequested => Color32::from_rgb(200, 180, 80),
            Self::Unlocked => Color32::from_rgb(80, 200, 80),
        }
    }
}

/// Diagnostics tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagTab {
    #[default]
    Session,
    Security,
    Dtc,
    DataId,
    Routine,
    Log,
}

/// DTC entry
#[derive(Debug, Clone)]
pub struct DtcEntry {
    /// DTC code (e.g., P0123)
    pub code: String,
    /// DTC number (24-bit)
    pub number: u32,
    /// Status byte
    pub status: u8,
    /// Description
    pub description: String,
    /// Is confirmed
    pub confirmed: bool,
    /// Is pending
    pub pending: bool,
}

impl DtcEntry {
    /// Create new DTC entry
    pub fn new(number: u32, status: u8) -> Self {
        Self {
            code: Self::format_dtc_code(number),
            number,
            status,
            description: Self::get_description(number),
            confirmed: (status & 0x08) != 0,
            pending: (status & 0x04) != 0,
        }
    }

    /// Format DTC number as standard code (P0123, C0456, etc.)
    fn format_dtc_code(number: u32) -> String {
        let first_char = match (number >> 14) & 0x03 {
            0 => 'P', // Powertrain
            1 => 'C', // Chassis
            2 => 'B', // Body
            3 => 'U', // Network
            _ => '?',
        };
        let second_digit = (number >> 12) & 0x03;
        let remaining = number & 0x0FFF;
        format!("{}{}{:03X}", first_char, second_digit, remaining)
    }

    /// Get DTC description (placeholder - would come from database)
    fn get_description(number: u32) -> String {
        // Common DTCs
        match number {
            0x0100 => "Mass Air Flow Circuit Malfunction".to_string(),
            0x0101 => "Mass Air Flow Circuit Range/Performance".to_string(),
            0x0102 => "Mass Air Flow Circuit Low".to_string(),
            0x0103 => "Mass Air Flow Circuit High".to_string(),
            0x0110 => "Intake Air Temperature Circuit Malfunction".to_string(),
            0x0115 => "Engine Coolant Temperature Circuit Malfunction".to_string(),
            0x0120 => "Throttle Position Sensor Circuit Malfunction".to_string(),
            0x0130 => "O2 Sensor Circuit Malfunction (Bank 1 Sensor 1)".to_string(),
            0x0300 => "Random/Multiple Cylinder Misfire Detected".to_string(),
            0x0301 => "Cylinder 1 Misfire Detected".to_string(),
            0x0302 => "Cylinder 2 Misfire Detected".to_string(),
            0x0303 => "Cylinder 3 Misfire Detected".to_string(),
            0x0304 => "Cylinder 4 Misfire Detected".to_string(),
            0x0420 => "Catalyst System Efficiency Below Threshold".to_string(),
            0x0500 => "Vehicle Speed Sensor Malfunction".to_string(),
            _ => format!("Unknown DTC 0x{:06X}", number),
        }
    }
}

/// DID read result
#[derive(Debug, Clone)]
pub struct DidResult {
    /// DID number
    pub did: u16,
    /// DID name
    pub name: String,
    /// Raw data
    pub data: Vec<u8>,
    /// Interpreted value
    pub value: String,
    /// Timestamp
    pub timestamp: String,
}

impl DidResult {
    /// Create new DID result
    pub fn new(did: u16, data: Vec<u8>) -> Self {
        Self {
            did,
            name: Self::get_did_name(did),
            value: Self::interpret_data(did, &data),
            data,
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        }
    }

    /// Get DID name
    fn get_did_name(did: u16) -> String {
        match did {
            0xF180 => "Boot Software Identification".to_string(),
            0xF181 => "Application Software Identification".to_string(),
            0xF182 => "Application Data Identification".to_string(),
            0xF183 => "Boot Software Fingerprint".to_string(),
            0xF184 => "Application Software Fingerprint".to_string(),
            0xF185 => "Application Data Fingerprint".to_string(),
            0xF186 => "Active Diagnostic Session".to_string(),
            0xF187 => "Vehicle Manufacturer Spare Part Number".to_string(),
            0xF188 => "Vehicle Manufacturer ECU Software Number".to_string(),
            0xF189 => "Vehicle Manufacturer ECU Software Version".to_string(),
            0xF18A => "System Supplier Identifier".to_string(),
            0xF18B => "ECU Manufacturing Date".to_string(),
            0xF18C => "ECU Serial Number".to_string(),
            0xF190 => "VIN (Vehicle Identification Number)".to_string(),
            0xF191 => "Vehicle Manufacturer ECU Hardware Number".to_string(),
            0xF192 => "System Supplier ECU Hardware Number".to_string(),
            0xF193 => "System Supplier ECU Hardware Version".to_string(),
            0xF194 => "System Supplier ECU Software Number".to_string(),
            0xF195 => "System Supplier ECU Software Version".to_string(),
            0xF197 => "System Name or Engine Type".to_string(),
            0xF198 => "Repair Shop Code or Tester Serial Number".to_string(),
            0xF199 => "Programming Date".to_string(),
            0xF19E => "Application Data Identification".to_string(),
            _ => format!("DID 0x{:04X}", did),
        }
    }

    /// Interpret DID data
    fn interpret_data(did: u16, data: &[u8]) -> String {
        match did {
            // VIN - ASCII string
            0xF190 => {
                if let Ok(s) = std::str::from_utf8(data) {
                    s.trim_end_matches('\0').to_string()
                } else {
                    Self::hex_string(data)
                }
            }
            // Active session - special handling
            0xF186 => {
                if let Some(&session) = data.first() {
                    match session {
                        0x01 => "Default Session".to_string(),
                        0x02 => "Programming Session".to_string(),
                        0x03 => "Extended Session".to_string(),
                        0x04 => "Safety System Session".to_string(),
                        _ => format!("Session 0x{:02X}", session),
                    }
                } else {
                    "No data".to_string()
                }
            }
            // Software/hardware IDs - usually ASCII
            0xF180..=0xF189 | 0xF191..=0xF199 => {
                if let Ok(s) = std::str::from_utf8(data) {
                    s.trim_end_matches('\0').to_string()
                } else {
                    Self::hex_string(data)
                }
            }
            // Default: hex dump
            _ => Self::hex_string(data),
        }
    }

    /// Format data as hex string
    fn hex_string(data: &[u8]) -> String {
        data.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Routine control result
#[derive(Debug, Clone)]
pub struct RoutineResult {
    /// Routine ID
    pub routine_id: u16,
    /// Control type
    pub control_type: String,
    /// Status
    pub status: String,
    /// Result data
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: String,
}

/// Service log entry
#[derive(Debug, Clone)]
pub struct ServiceLogEntry {
    /// Timestamp
    pub timestamp: String,
    /// Direction (TX/RX)
    pub direction: String,
    /// Service name
    pub service: String,
    /// Raw bytes
    pub bytes: Vec<u8>,
    /// Is error
    pub is_error: bool,
}

impl ServiceLogEntry {
    /// Create TX entry
    pub fn tx(service: &str, bytes: Vec<u8>) -> Self {
        Self {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction: "TX".to_string(),
            service: service.to_string(),
            bytes,
            is_error: false,
        }
    }

    /// Create RX entry
    pub fn rx(service: &str, bytes: Vec<u8>, is_error: bool) -> Self {
        Self {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction: "RX".to_string(),
            service: service.to_string(),
            bytes,
            is_error,
        }
    }
}

/// Diagnostics panel
pub struct DiagnosticsPanel;

impl DiagnosticsPanel {
    /// Show the diagnostics panel
    pub fn show(ui: &mut Ui, state: &mut DiagnosticsState) {
        ui.heading("🔧 Diagnostics");
        ui.separator();

        // ECU Address configuration
        ui.horizontal(|ui| {
            ui.label("ECU Request ID:");
            ui.add(egui::TextEdit::singleline(&mut state.ecu_request_id).desired_width(80.0));
            ui.label("Response ID:");
            ui.add(egui::TextEdit::singleline(&mut state.ecu_response_id).desired_width(80.0));
            
            // Connection status indicator
            ui.separator();
            if state.is_connected {
                ui.colored_label(Color32::from_rgb(80, 200, 80), "● Connected");
            } else {
                ui.colored_label(Color32::from_rgb(200, 80, 80), "● Disconnected");
            }
        });

        ui.separator();

        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.active_tab, DiagTab::Session, "📋 Session");
            ui.selectable_value(&mut state.active_tab, DiagTab::Security, "🔐 Security");
            ui.selectable_value(&mut state.active_tab, DiagTab::Dtc, "⚠ DTCs");
            ui.selectable_value(&mut state.active_tab, DiagTab::DataId, "📊 Data IDs");
            ui.selectable_value(&mut state.active_tab, DiagTab::Routine, "⚙ Routines");
            ui.selectable_value(&mut state.active_tab, DiagTab::Log, "📜 Log");
        });

        ui.separator();

        // Tab content
        match state.active_tab {
            DiagTab::Session => Self::show_session_tab(ui, state),
            DiagTab::Security => Self::show_security_tab(ui, state),
            DiagTab::Dtc => Self::show_dtc_tab(ui, state),
            DiagTab::DataId => Self::show_data_id_tab(ui, state),
            DiagTab::Routine => Self::show_routine_tab(ui, state),
            DiagTab::Log => Self::show_log_tab(ui, state),
        }

        // Error display
        if let Some(error) = &state.last_error {
            ui.separator();
            ui.colored_label(Color32::from_rgb(255, 100, 100), format!("⚠ {}", error));
        }
    }

    /// Show session management tab
    fn show_session_tab(ui: &mut Ui, state: &mut DiagnosticsState) {
        ui.heading("Diagnostic Session Control");
        ui.add_space(8.0);

        // Current session display
        ui.horizontal(|ui| {
            ui.label("Current Session:");
            ui.colored_label(Color32::from_rgb(100, 180, 255), state.session.name());
        });

        ui.add_space(16.0);

        // Session selection buttons
        ui.label("Change Session:");
        ui.horizontal(|ui| {
            if ui.button("Default (0x01)").clicked() {
                state.session = DiagSession::Default;
                state.log_service(ServiceLogEntry::tx(
                    "DiagnosticSessionControl",
                    vec![0x10, 0x01],
                ));
            }
            if ui.button("Extended (0x03)").clicked() {
                state.session = DiagSession::Extended;
                state.log_service(ServiceLogEntry::tx(
                    "DiagnosticSessionControl",
                    vec![0x10, 0x03],
                ));
            }
            if ui.button("Programming (0x02)").clicked() {
                state.session = DiagSession::Programming;
                state.log_service(ServiceLogEntry::tx(
                    "DiagnosticSessionControl",
                    vec![0x10, 0x02],
                ));
            }
        });

        ui.add_space(16.0);
        ui.separator();

        // ECU Reset
        ui.heading("ECU Reset");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Hard Reset").clicked() {
                state.log_service(ServiceLogEntry::tx("EcuReset", vec![0x11, 0x01]));
            }
            if ui.button("Key Off/On").clicked() {
                state.log_service(ServiceLogEntry::tx("EcuReset", vec![0x11, 0x02]));
            }
            if ui.button("Soft Reset").clicked() {
                state.log_service(ServiceLogEntry::tx("EcuReset", vec![0x11, 0x03]));
            }
        });

        ui.add_space(16.0);
        ui.separator();

        // Tester Present
        ui.heading("Tester Present");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Send Tester Present").clicked() {
                state.log_service(ServiceLogEntry::tx("TesterPresent", vec![0x3E, 0x00]));
            }
            if ui.button("Send (Suppress Response)").clicked() {
                state.log_service(ServiceLogEntry::tx("TesterPresent", vec![0x3E, 0x80]));
            }
        });
    }

    /// Show security access tab
    fn show_security_tab(ui: &mut Ui, state: &mut DiagnosticsState) {
        ui.heading("Security Access");
        ui.add_space(8.0);

        // Security state display
        ui.horizontal(|ui| {
            ui.label("Security State:");
            ui.colored_label(state.security_state.color(), state.security_state.name());
        });

        ui.add_space(8.0);

        // Security level selection
        ui.horizontal(|ui| {
            ui.label("Security Level:");
            ui.add(egui::DragValue::new(&mut state.security_level).range(1..=127));
            ui.label("(odd = request seed, even = send key)");
        });

        ui.add_space(16.0);

        // Request Seed
        ui.horizontal(|ui| {
            if ui.button("Request Seed").clicked() {
                let level = if state.security_level % 2 == 0 {
                    state.security_level - 1
                } else {
                    state.security_level
                };
                state.security_state = SecurityState::SeedRequested;
                state.log_service(ServiceLogEntry::tx(
                    "SecurityAccess (Request Seed)",
                    vec![0x27, level],
                ));
                // Simulate seed response
                state.seed = vec![0x12, 0x34, 0x56, 0x78];
            }
        });

        // Display seed if received
        if !state.seed.is_empty() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Received Seed:");
                let seed_hex: String = state
                    .seed
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                ui.monospace(seed_hex);
            });
        }

        ui.add_space(16.0);

        // Send Key
        ui.horizontal(|ui| {
            ui.label("Key (hex):");
            ui.add(egui::TextEdit::singleline(&mut state.key_input).desired_width(200.0));
        });

        ui.horizontal(|ui| {
            if ui.button("Send Key").clicked() {
                let level = if state.security_level % 2 == 0 {
                    state.security_level
                } else {
                    state.security_level + 1
                };
                // Parse key from hex
                let key_bytes: Vec<u8> = state
                    .key_input
                    .split_whitespace()
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    .collect();
                let mut request = vec![0x27, level];
                request.extend(&key_bytes);
                state.log_service(ServiceLogEntry::tx("SecurityAccess (Send Key)", request));
                // Simulate unlock
                state.security_state = SecurityState::Unlocked;
                state.seed.clear();
            }
        });

        ui.add_space(16.0);
        ui.separator();

        // Common seed-key algorithms info
        ui.collapsing("ℹ Seed-Key Algorithm Info", |ui| {
            ui.label("Common algorithms:");
            ui.label("• XOR with constant");
            ui.label("• Bit rotation + XOR");
            ui.label("• CRC-based");
            ui.label("• Manufacturer-specific");
            ui.add_space(4.0);
            ui.label("Note: Actual algorithm depends on ECU manufacturer.");
        });
    }

    /// Show DTC tab
    fn show_dtc_tab(ui: &mut Ui, state: &mut DiagnosticsState) {
        ui.heading("Diagnostic Trouble Codes");
        ui.add_space(8.0);

        // DTC controls
        ui.horizontal(|ui| {
            if ui.button("📖 Read DTCs").clicked() {
                state.log_service(ServiceLogEntry::tx(
                    "ReadDTCInformation",
                    vec![0x19, 0x02, state.dtc_status_filter],
                ));
                // Simulate some DTCs
                state.dtc_list = vec![
                    DtcEntry::new(0x0100, 0x09), // Confirmed
                    DtcEntry::new(0x0300, 0x04), // Pending
                    DtcEntry::new(0x0420, 0x08), // Confirmed
                ];
            }
            if ui.button("🗑 Clear DTCs").clicked() {
                state.log_service(ServiceLogEntry::tx(
                    "ClearDiagnosticInformation",
                    vec![0x14, 0xFF, 0xFF, 0xFF],
                ));
                state.dtc_list.clear();
            }
            ui.separator();
            ui.label("Status Filter:");
            ui.add(
                egui::DragValue::new(&mut state.dtc_status_filter)
                    .hexadecimal(2, false, true)
                    .prefix("0x"),
            );
        });

        ui.add_space(8.0);

        // Status filter checkboxes
        ui.collapsing("Status Filter Options", |ui| {
            ui.horizontal(|ui| {
                let mut test_failed = (state.dtc_status_filter & 0x01) != 0;
                let mut pending = (state.dtc_status_filter & 0x04) != 0;
                let mut confirmed = (state.dtc_status_filter & 0x08) != 0;
                let mut warning = (state.dtc_status_filter & 0x80) != 0;

                if ui.checkbox(&mut test_failed, "Test Failed").changed() {
                    state.dtc_status_filter =
                        (state.dtc_status_filter & !0x01) | if test_failed { 0x01 } else { 0 };
                }
                if ui.checkbox(&mut pending, "Pending").changed() {
                    state.dtc_status_filter =
                        (state.dtc_status_filter & !0x04) | if pending { 0x04 } else { 0 };
                }
                if ui.checkbox(&mut confirmed, "Confirmed").changed() {
                    state.dtc_status_filter =
                        (state.dtc_status_filter & !0x08) | if confirmed { 0x08 } else { 0 };
                }
                if ui.checkbox(&mut warning, "Warning Indicator").changed() {
                    state.dtc_status_filter =
                        (state.dtc_status_filter & !0x80) | if warning { 0x80 } else { 0 };
                }
            });
        });

        ui.separator();

        // DTC list
        ui.label(format!("DTCs Found: {}", state.dtc_list.len()));
        ui.add_space(4.0);

        ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                egui::Grid::new("dtc_grid")
                    .num_columns(5)
                    .striped(true)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.strong("Code");
                        ui.strong("Status");
                        ui.strong("State");
                        ui.strong("Description");
                        ui.strong("Raw");
                        ui.end_row();

                        for dtc in &state.dtc_list {
                            // Code
                            ui.monospace(&dtc.code);

                            // Status byte
                            ui.monospace(format!("0x{:02X}", dtc.status));

                            // State (confirmed/pending)
                            if dtc.confirmed {
                                ui.colored_label(Color32::from_rgb(255, 100, 100), "Confirmed");
                            } else if dtc.pending {
                                ui.colored_label(Color32::from_rgb(255, 200, 100), "Pending");
                            } else {
                                ui.label("-");
                            }

                            // Description
                            ui.label(&dtc.description);

                            // Raw number
                            ui.monospace(format!("0x{:06X}", dtc.number));

                            ui.end_row();
                        }
                    });
            });
    }


    /// Show Data ID tab
    fn show_data_id_tab(ui: &mut Ui, state: &mut DiagnosticsState) {
        ui.heading("Read/Write Data By Identifier");
        ui.add_space(8.0);

        // Read DID
        ui.horizontal(|ui| {
            ui.label("DID (hex):");
            ui.add(egui::TextEdit::singleline(&mut state.did_input).desired_width(80.0));
            if ui.button("📖 Read").clicked() {
                if let Ok(did) = u16::from_str_radix(&state.did_input, 16) {
                    state.log_service(ServiceLogEntry::tx(
                        "ReadDataByIdentifier",
                        vec![0x22, (did >> 8) as u8, (did & 0xFF) as u8],
                    ));
                    // Simulate response
                    let data = match did {
                        0xF190 => b"WVWZZZ3CZWE123456".to_vec(), // VIN
                        0xF186 => vec![0x03],                    // Extended session
                        0xF188 => b"SW_V1.2.3".to_vec(),         // Software version
                        _ => vec![0x00, 0x01, 0x02, 0x03],
                    };
                    state.did_results.push(DidResult::new(did, data));
                }
            }
        });

        // Common DIDs quick buttons
        ui.add_space(8.0);
        ui.label("Common DIDs:");
        ui.horizontal(|ui| {
            if ui.small_button("VIN (F190)").clicked() {
                state.did_input = "F190".to_string();
            }
            if ui.small_button("Session (F186)").clicked() {
                state.did_input = "F186".to_string();
            }
            if ui.small_button("SW Ver (F188)").clicked() {
                state.did_input = "F188".to_string();
            }
            if ui.small_button("HW Ver (F191)").clicked() {
                state.did_input = "F191".to_string();
            }
            if ui.small_button("Serial (F18C)").clicked() {
                state.did_input = "F18C".to_string();
            }
        });

        ui.add_space(16.0);
        ui.separator();

        // Write DID
        ui.heading("Write Data By Identifier");
        ui.horizontal(|ui| {
            ui.label("Value (hex):");
            ui.add(egui::TextEdit::singleline(&mut state.did_write_value).desired_width(200.0));
            if ui.button("✏ Write").clicked() {
                if let Ok(did) = u16::from_str_radix(&state.did_input, 16) {
                    let value_bytes: Vec<u8> = state
                        .did_write_value
                        .split_whitespace()
                        .filter_map(|s| u8::from_str_radix(s, 16).ok())
                        .collect();
                    let mut request = vec![0x2E, (did >> 8) as u8, (did & 0xFF) as u8];
                    request.extend(&value_bytes);
                    state.log_service(ServiceLogEntry::tx("WriteDataByIdentifier", request));
                }
            }
        });

        ui.add_space(16.0);
        ui.separator();

        // Results
        ui.heading("Read Results");
        if ui.button("Clear Results").clicked() {
            state.did_results.clear();
        }

        ui.add_space(4.0);

        ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                egui::Grid::new("did_grid")
                    .num_columns(5)
                    .striped(true)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.strong("Time");
                        ui.strong("DID");
                        ui.strong("Name");
                        ui.strong("Value");
                        ui.strong("Raw");
                        ui.end_row();

                        for result in state.did_results.iter().rev() {
                            ui.monospace(&result.timestamp);
                            ui.monospace(format!("0x{:04X}", result.did));
                            ui.label(&result.name);
                            ui.label(&result.value);
                            ui.monospace(
                                result
                                    .data
                                    .iter()
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            );
                            ui.end_row();
                        }
                    });
            });
    }

    /// Show Routine tab
    fn show_routine_tab(ui: &mut Ui, state: &mut DiagnosticsState) {
        ui.heading("Routine Control");
        ui.add_space(8.0);

        // Routine ID input
        ui.horizontal(|ui| {
            ui.label("Routine ID (hex):");
            ui.add(egui::TextEdit::singleline(&mut state.routine_id).desired_width(80.0));
        });

        ui.horizontal(|ui| {
            ui.label("Option Record (hex):");
            ui.add(egui::TextEdit::singleline(&mut state.routine_option).desired_width(200.0));
        });

        ui.add_space(8.0);

        // Routine control buttons
        ui.horizontal(|ui| {
            if ui.button("▶ Start Routine").clicked() {
                if let Ok(routine_id) = u16::from_str_radix(&state.routine_id, 16) {
                    let option_bytes: Vec<u8> = state
                        .routine_option
                        .split_whitespace()
                        .filter_map(|s| u8::from_str_radix(s, 16).ok())
                        .collect();
                    let mut request = vec![
                        0x31,
                        0x01, // Start
                        (routine_id >> 8) as u8,
                        (routine_id & 0xFF) as u8,
                    ];
                    request.extend(&option_bytes);
                    state.log_service(ServiceLogEntry::tx("RoutineControl (Start)", request));
                    state.routine_results.push(RoutineResult {
                        routine_id,
                        control_type: "Start".to_string(),
                        status: "Started".to_string(),
                        data: vec![],
                        timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
                    });
                }
            }
            if ui.button("⏹ Stop Routine").clicked() {
                if let Ok(routine_id) = u16::from_str_radix(&state.routine_id, 16) {
                    let request = vec![
                        0x31,
                        0x02, // Stop
                        (routine_id >> 8) as u8,
                        (routine_id & 0xFF) as u8,
                    ];
                    state.log_service(ServiceLogEntry::tx("RoutineControl (Stop)", request));
                }
            }
            if ui.button("📊 Request Results").clicked() {
                if let Ok(routine_id) = u16::from_str_radix(&state.routine_id, 16) {
                    let request = vec![
                        0x31,
                        0x03, // Request Results
                        (routine_id >> 8) as u8,
                        (routine_id & 0xFF) as u8,
                    ];
                    state.log_service(ServiceLogEntry::tx(
                        "RoutineControl (Request Results)",
                        request,
                    ));
                }
            }
        });

        ui.add_space(16.0);

        // Common routines
        ui.label("Common Routines:");
        ui.horizontal(|ui| {
            if ui.small_button("Erase Memory (FF00)").clicked() {
                state.routine_id = "FF00".to_string();
            }
            if ui.small_button("Check Prog Deps (FF01)").clicked() {
                state.routine_id = "FF01".to_string();
            }
            if ui.small_button("Check Memory (0202)").clicked() {
                state.routine_id = "0202".to_string();
            }
        });

        ui.add_space(16.0);
        ui.separator();

        // Results
        ui.heading("Routine Results");
        if ui.button("Clear Results").clicked() {
            state.routine_results.clear();
        }

        ui.add_space(4.0);

        ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                egui::Grid::new("routine_grid")
                    .num_columns(5)
                    .striped(true)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.strong("Time");
                        ui.strong("Routine ID");
                        ui.strong("Type");
                        ui.strong("Status");
                        ui.strong("Data");
                        ui.end_row();

                        for result in state.routine_results.iter().rev() {
                            ui.monospace(&result.timestamp);
                            ui.monospace(format!("0x{:04X}", result.routine_id));
                            ui.label(&result.control_type);
                            ui.label(&result.status);
                            ui.monospace(
                                result
                                    .data
                                    .iter()
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            );
                            ui.end_row();
                        }
                    });
            });
    }

    /// Show service log tab
    fn show_log_tab(ui: &mut Ui, state: &mut DiagnosticsState) {
        ui.heading("Service Log");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Clear Log").clicked() {
                state.clear_log();
            }
            ui.label(format!("Entries: {}", state.service_log.len()));
        });

        ui.add_space(8.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("log_grid")
                    .num_columns(4)
                    .striped(true)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.strong("Time");
                        ui.strong("Dir");
                        ui.strong("Service");
                        ui.strong("Data");
                        ui.end_row();

                        for entry in state.service_log.iter().rev() {
                            ui.monospace(&entry.timestamp);

                            // Direction with color
                            let dir_color = if entry.direction == "TX" {
                                Color32::from_rgb(100, 180, 255)
                            } else if entry.is_error {
                                Color32::from_rgb(255, 100, 100)
                            } else {
                                Color32::from_rgb(100, 255, 100)
                            };
                            ui.colored_label(dir_color, &entry.direction);

                            // Service name
                            if entry.is_error {
                                ui.colored_label(Color32::from_rgb(255, 100, 100), &entry.service);
                            } else {
                                ui.label(&entry.service);
                            }

                            // Data bytes
                            ui.monospace(
                                entry
                                    .bytes
                                    .iter()
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            );

                            ui.end_row();
                        }
                    });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diag_session() {
        assert_eq!(DiagSession::Default.value(), 0x01);
        assert_eq!(DiagSession::Extended.value(), 0x03);
        assert_eq!(DiagSession::Programming.value(), 0x02);
    }

    #[test]
    fn test_security_state() {
        assert_eq!(SecurityState::Locked.name(), "Locked");
        assert_eq!(SecurityState::Unlocked.name(), "Unlocked");
    }

    #[test]
    fn test_dtc_entry() {
        let dtc = DtcEntry::new(0x0100, 0x09);
        assert_eq!(dtc.code, "P0100");
        assert!(dtc.confirmed);
        assert!(!dtc.pending);
    }

    #[test]
    fn test_dtc_code_format() {
        // Powertrain
        let dtc = DtcEntry::new(0x0123, 0x00);
        assert_eq!(dtc.code, "P0123");

        // Chassis (0x4000 = C)
        let dtc = DtcEntry::new(0x4123, 0x00);
        assert_eq!(dtc.code, "C0123");

        // Body (0x8000 = B)
        let dtc = DtcEntry::new(0x8123, 0x00);
        assert_eq!(dtc.code, "B0123");

        // Network (0xC000 = U)
        let dtc = DtcEntry::new(0xC123, 0x00);
        assert_eq!(dtc.code, "U0123");
    }

    #[test]
    fn test_did_result() {
        let result = DidResult::new(0xF190, b"WVWZZZ3CZWE123456".to_vec());
        assert_eq!(result.did, 0xF190);
        assert_eq!(result.name, "VIN (Vehicle Identification Number)");
        assert_eq!(result.value, "WVWZZZ3CZWE123456");
    }

    #[test]
    fn test_diagnostics_state_new() {
        let state = DiagnosticsState::new();
        assert_eq!(state.session, DiagSession::Default);
        assert_eq!(state.security_state, SecurityState::Locked);
        assert!(state.dtc_list.is_empty());
    }

    #[test]
    fn test_service_log() {
        let mut state = DiagnosticsState::new();
        state.log_service(ServiceLogEntry::tx("Test", vec![0x10, 0x01]));
        assert_eq!(state.service_log.len(), 1);
        assert_eq!(state.service_log[0].direction, "TX");
    }

    #[test]
    fn test_service_log_max_entries() {
        let mut state = DiagnosticsState::new();
        state.max_log_entries = 5;
        for i in 0..10 {
            state.log_service(ServiceLogEntry::tx(&format!("Test{}", i), vec![i as u8]));
        }
        assert_eq!(state.service_log.len(), 5);
    }
}
