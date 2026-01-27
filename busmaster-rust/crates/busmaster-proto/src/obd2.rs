//! OBD-II (On-Board Diagnostics) Protocol Implementation
//!
//! OBD-II is a standardized diagnostic protocol mandated for all vehicles sold
//! in the US since 1996. It provides access to emissions-related data and DTCs.
//!
//! # Key Concepts
//!
//! ## Modes (Services)
//! OBD-II defines 10 modes (01-0A) for different diagnostic functions.
//!
//! ## PIDs (Parameter IDs)
//! Each mode uses PIDs to identify specific data parameters.
//!
//! ## CAN IDs
//! - Request: 0x7DF (broadcast) or 0x7E0-0x7E7 (specific ECU)
//! - Response: 0x7E8-0x7EF
//!
//! # Example
//!
//! ```
//! use busmaster_proto::obd2::{Obd2Mode, Obd2Pid, Obd2Request};
//!
//! // Request engine RPM
//! let request = Obd2Request::new(Obd2Mode::CurrentData, Obd2Pid::ENGINE_RPM);
//! let bytes = request.encode();
//! assert_eq!(bytes, vec![0x01, 0x0C]);
//! ```

use serde::{Deserialize, Serialize};

/// OBD-II broadcast request CAN ID
pub const OBD2_REQUEST_BROADCAST: u32 = 0x7DF;

/// OBD-II ECU request CAN ID range start
pub const OBD2_REQUEST_ECU_START: u32 = 0x7E0;

/// OBD-II ECU response CAN ID range start
pub const OBD2_RESPONSE_ECU_START: u32 = 0x7E8;

/// OBD-II Modes (Services)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Obd2Mode {
    /// Mode 01: Show current data
    CurrentData = 0x01,
    /// Mode 02: Show freeze frame data
    FreezeFrameData = 0x02,
    /// Mode 03: Show stored DTCs
    StoredDtcs = 0x03,
    /// Mode 04: Clear DTCs and stored values
    ClearDtcs = 0x04,
    /// Mode 05: Test results, oxygen sensor monitoring (non-CAN)
    OxygenSensorMonitoring = 0x05,
    /// Mode 06: Test results, other component/system monitoring
    OnBoardMonitoringTest = 0x06,
    /// Mode 07: Show pending DTCs
    PendingDtcs = 0x07,
    /// Mode 08: Control operation of on-board component/system
    ControlOperation = 0x08,
    /// Mode 09: Request vehicle information
    VehicleInformation = 0x09,
    /// Mode 0A: Permanent DTCs (cleared DTCs)
    PermanentDtcs = 0x0A,
}

impl Obd2Mode {
    /// Get the positive response mode (mode + 0x40)
    #[must_use]
    pub fn positive_response(self) -> u8 {
        (self as u8) + 0x40
    }

    /// Try to create from a raw byte
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::CurrentData),
            0x02 => Some(Self::FreezeFrameData),
            0x03 => Some(Self::StoredDtcs),
            0x04 => Some(Self::ClearDtcs),
            0x05 => Some(Self::OxygenSensorMonitoring),
            0x06 => Some(Self::OnBoardMonitoringTest),
            0x07 => Some(Self::PendingDtcs),
            0x08 => Some(Self::ControlOperation),
            0x09 => Some(Self::VehicleInformation),
            0x0A => Some(Self::PermanentDtcs),
            _ => None,
        }
    }
}

/// Common OBD-II PIDs for Mode 01 (Current Data)
pub struct Obd2Pid;

impl Obd2Pid {
    /// PIDs supported [01-20]
    pub const PIDS_SUPPORTED_01_20: u8 = 0x00;
    /// Monitor status since DTCs cleared
    pub const MONITOR_STATUS: u8 = 0x01;
    /// Freeze DTC
    pub const FREEZE_DTC: u8 = 0x02;
    /// Fuel system status
    pub const FUEL_SYSTEM_STATUS: u8 = 0x03;
    /// Calculated engine load
    pub const ENGINE_LOAD: u8 = 0x04;
    /// Engine coolant temperature
    pub const COOLANT_TEMP: u8 = 0x05;
    /// Short term fuel trim - Bank 1
    pub const SHORT_TERM_FUEL_TRIM_B1: u8 = 0x06;
    /// Long term fuel trim - Bank 1
    pub const LONG_TERM_FUEL_TRIM_B1: u8 = 0x07;
    /// Short term fuel trim - Bank 2
    pub const SHORT_TERM_FUEL_TRIM_B2: u8 = 0x08;
    /// Long term fuel trim - Bank 2
    pub const LONG_TERM_FUEL_TRIM_B2: u8 = 0x09;
    /// Fuel pressure
    pub const FUEL_PRESSURE: u8 = 0x0A;
    /// Intake manifold absolute pressure
    pub const INTAKE_MAP: u8 = 0x0B;
    /// Engine RPM
    pub const ENGINE_RPM: u8 = 0x0C;
    /// Vehicle speed
    pub const VEHICLE_SPEED: u8 = 0x0D;
    /// Timing advance
    pub const TIMING_ADVANCE: u8 = 0x0E;
    /// Intake air temperature
    pub const INTAKE_AIR_TEMP: u8 = 0x0F;
    /// MAF air flow rate
    pub const MAF_FLOW_RATE: u8 = 0x10;
    /// Throttle position
    pub const THROTTLE_POSITION: u8 = 0x11;
    /// Commanded secondary air status
    pub const SECONDARY_AIR_STATUS: u8 = 0x12;
    /// Oxygen sensors present (2 banks)
    pub const O2_SENSORS_PRESENT: u8 = 0x13;
    /// Oxygen sensor 1 - Bank 1
    pub const O2_SENSOR_1_B1: u8 = 0x14;
    /// Oxygen sensor 2 - Bank 1
    pub const O2_SENSOR_2_B1: u8 = 0x15;
    /// Oxygen sensor 3 - Bank 1
    pub const O2_SENSOR_3_B1: u8 = 0x16;
    /// Oxygen sensor 4 - Bank 1
    pub const O2_SENSOR_4_B1: u8 = 0x17;
    /// Oxygen sensor 1 - Bank 2
    pub const O2_SENSOR_1_B2: u8 = 0x18;
    /// Oxygen sensor 2 - Bank 2
    pub const O2_SENSOR_2_B2: u8 = 0x19;
    /// Oxygen sensor 3 - Bank 2
    pub const O2_SENSOR_3_B2: u8 = 0x1A;
    /// Oxygen sensor 4 - Bank 2
    pub const O2_SENSOR_4_B2: u8 = 0x1B;
    /// OBD standards this vehicle conforms to
    pub const OBD_STANDARDS: u8 = 0x1C;
    /// Oxygen sensors present (4 banks)
    pub const O2_SENSORS_PRESENT_4B: u8 = 0x1D;
    /// Auxiliary input status
    pub const AUX_INPUT_STATUS: u8 = 0x1E;
    /// Run time since engine start
    pub const RUN_TIME: u8 = 0x1F;
    /// PIDs supported [21-40]
    pub const PIDS_SUPPORTED_21_40: u8 = 0x20;
    /// Distance traveled with MIL on
    pub const DISTANCE_WITH_MIL: u8 = 0x21;
    /// Fuel rail pressure (relative to manifold vacuum)
    pub const FUEL_RAIL_PRESSURE_VAC: u8 = 0x22;
    /// Fuel rail gauge pressure (diesel)
    pub const FUEL_RAIL_PRESSURE_DIRECT: u8 = 0x23;
    /// Commanded EGR
    pub const COMMANDED_EGR: u8 = 0x2C;
    /// EGR error
    pub const EGR_ERROR: u8 = 0x2D;
    /// Commanded evaporative purge
    pub const COMMANDED_EVAP_PURGE: u8 = 0x2E;
    /// Fuel tank level input
    pub const FUEL_TANK_LEVEL: u8 = 0x2F;
    /// Warm-ups since codes cleared
    pub const WARMUPS_SINCE_CLEAR: u8 = 0x30;
    /// Distance traveled since codes cleared
    pub const DISTANCE_SINCE_CLEAR: u8 = 0x31;
    /// Evap system vapor pressure
    pub const EVAP_VAPOR_PRESSURE: u8 = 0x32;
    /// Absolute barometric pressure
    pub const BAROMETRIC_PRESSURE: u8 = 0x33;
    /// Catalyst temperature Bank 1, Sensor 1
    pub const CATALYST_TEMP_B1S1: u8 = 0x3C;
    /// Catalyst temperature Bank 2, Sensor 1
    pub const CATALYST_TEMP_B2S1: u8 = 0x3D;
    /// Catalyst temperature Bank 1, Sensor 2
    pub const CATALYST_TEMP_B1S2: u8 = 0x3E;
    /// Catalyst temperature Bank 2, Sensor 2
    pub const CATALYST_TEMP_B2S2: u8 = 0x3F;
    /// PIDs supported [41-60]
    pub const PIDS_SUPPORTED_41_60: u8 = 0x40;
    /// Monitor status this drive cycle
    pub const MONITOR_STATUS_DRIVE_CYCLE: u8 = 0x41;
    /// Control module voltage
    pub const CONTROL_MODULE_VOLTAGE: u8 = 0x42;
    /// Absolute load value
    pub const ABSOLUTE_LOAD: u8 = 0x43;
    /// Commanded air-fuel equivalence ratio
    pub const COMMANDED_AFR: u8 = 0x44;
    /// Relative throttle position
    pub const RELATIVE_THROTTLE: u8 = 0x45;
    /// Ambient air temperature
    pub const AMBIENT_AIR_TEMP: u8 = 0x46;
    /// Absolute throttle position B
    pub const THROTTLE_POSITION_B: u8 = 0x47;
    /// Absolute throttle position C
    pub const THROTTLE_POSITION_C: u8 = 0x48;
    /// Accelerator pedal position D
    pub const ACCELERATOR_POSITION_D: u8 = 0x49;
    /// Accelerator pedal position E
    pub const ACCELERATOR_POSITION_E: u8 = 0x4A;
    /// Accelerator pedal position F
    pub const ACCELERATOR_POSITION_F: u8 = 0x4B;
    /// Commanded throttle actuator
    pub const COMMANDED_THROTTLE: u8 = 0x4C;
    /// Time run with MIL on
    pub const TIME_WITH_MIL: u8 = 0x4D;
    /// Time since trouble codes cleared
    pub const TIME_SINCE_CLEAR: u8 = 0x4E;
    /// Fuel type
    pub const FUEL_TYPE: u8 = 0x51;
    /// Ethanol fuel percentage
    pub const ETHANOL_PERCENT: u8 = 0x52;
    /// Engine oil temperature
    pub const ENGINE_OIL_TEMP: u8 = 0x5C;
    /// Fuel injection timing
    pub const FUEL_INJECTION_TIMING: u8 = 0x5D;
    /// Engine fuel rate
    pub const ENGINE_FUEL_RATE: u8 = 0x5E;
    /// Odometer
    pub const ODOMETER: u8 = 0xA6;
}

/// Mode 09 PIDs (Vehicle Information)
pub struct VehicleInfoPid;

impl VehicleInfoPid {
    /// VIN message count
    pub const VIN_MESSAGE_COUNT: u8 = 0x01;
    /// Vehicle Identification Number (VIN)
    pub const VIN: u8 = 0x02;
    /// Calibration ID message count
    pub const CALIBRATION_ID_COUNT: u8 = 0x03;
    /// Calibration ID
    pub const CALIBRATION_ID: u8 = 0x04;
    /// CVN message count
    pub const CVN_COUNT: u8 = 0x05;
    /// Calibration Verification Numbers (CVN)
    pub const CVN: u8 = 0x06;
    /// In-use performance tracking message count
    pub const PERFORMANCE_TRACKING_COUNT: u8 = 0x07;
    /// In-use performance tracking
    pub const PERFORMANCE_TRACKING: u8 = 0x08;
    /// ECU name message count
    pub const ECU_NAME_COUNT: u8 = 0x09;
    /// ECU name
    pub const ECU_NAME: u8 = 0x0A;
}

/// OBD-II DTC (Diagnostic Trouble Code)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Obd2Dtc {
    /// First byte (contains category and first digit)
    byte1: u8,
    /// Second byte (contains remaining digits)
    byte2: u8,
}

impl Obd2Dtc {
    /// Create a new DTC from raw bytes
    #[must_use]
    pub fn new(byte1: u8, byte2: u8) -> Self {
        Self { byte1, byte2 }
    }

    /// Get the DTC category (P, C, B, U)
    #[must_use]
    pub fn category(&self) -> char {
        match (self.byte1 >> 6) & 0x03 {
            0 => 'P', // Powertrain
            1 => 'C', // Chassis
            2 => 'B', // Body
            3 => 'U', // Network
            _ => unreachable!(),
        }
    }

    /// Get the first digit (0-3)
    #[must_use]
    pub fn digit1(&self) -> u8 {
        (self.byte1 >> 4) & 0x03
    }

    /// Get the second digit (0-F)
    #[must_use]
    pub fn digit2(&self) -> u8 {
        self.byte1 & 0x0F
    }

    /// Get the third digit (0-F)
    #[must_use]
    pub fn digit3(&self) -> u8 {
        (self.byte2 >> 4) & 0x0F
    }

    /// Get the fourth digit (0-F)
    #[must_use]
    pub fn digit4(&self) -> u8 {
        self.byte2 & 0x0F
    }

    /// Format as standard DTC string (e.g., "P0420")
    #[must_use]
    pub fn to_string(&self) -> String {
        format!(
            "{}{:01X}{:01X}{:01X}{:01X}",
            self.category(),
            self.digit1(),
            self.digit2(),
            self.digit3(),
            self.digit4()
        )
    }

    /// Parse from a standard DTC string (e.g., "P0420")
    #[must_use]
    pub fn from_string(s: &str) -> Option<Self> {
        if s.len() != 5 {
            return None;
        }

        let chars: Vec<char> = s.chars().collect();
        let category = match chars[0].to_ascii_uppercase() {
            'P' => 0u8,
            'C' => 1,
            'B' => 2,
            'U' => 3,
            _ => return None,
        };

        let d1 = chars[1].to_digit(16)? as u8;
        let d2 = chars[2].to_digit(16)? as u8;
        let d3 = chars[3].to_digit(16)? as u8;
        let d4 = chars[4].to_digit(16)? as u8;

        if d1 > 3 {
            return None; // First digit must be 0-3
        }

        let byte1 = (category << 6) | (d1 << 4) | d2;
        let byte2 = (d3 << 4) | d4;

        Some(Self { byte1, byte2 })
    }

    /// Check if this is a generic (SAE) code (second digit 0)
    #[must_use]
    pub fn is_generic(&self) -> bool {
        self.digit1() == 0
    }

    /// Check if this is a manufacturer-specific code (second digit 1-3)
    #[must_use]
    pub fn is_manufacturer_specific(&self) -> bool {
        self.digit1() > 0
    }
}

/// OBD-II Request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Obd2Request {
    /// Mode (service)
    mode: Obd2Mode,
    /// PID (parameter ID)
    pid: Option<u8>,
    /// Additional data
    data: Vec<u8>,
}

impl Obd2Request {
    /// Create a new OBD-II request
    #[must_use]
    pub fn new(mode: Obd2Mode, pid: u8) -> Self {
        Self {
            mode,
            pid: Some(pid),
            data: vec![],
        }
    }

    /// Create a request without a PID (e.g., Mode 03, 04)
    #[must_use]
    pub fn new_no_pid(mode: Obd2Mode) -> Self {
        Self {
            mode,
            pid: None,
            data: vec![],
        }
    }

    /// Create a request with additional data
    #[must_use]
    pub fn new_with_data(mode: Obd2Mode, pid: u8, data: Vec<u8>) -> Self {
        Self {
            mode,
            pid: Some(pid),
            data,
        }
    }

    /// Request supported PIDs (Mode 01, PID 00)
    #[must_use]
    pub fn supported_pids() -> Self {
        Self::new(Obd2Mode::CurrentData, Obd2Pid::PIDS_SUPPORTED_01_20)
    }

    /// Request engine RPM
    #[must_use]
    pub fn engine_rpm() -> Self {
        Self::new(Obd2Mode::CurrentData, Obd2Pid::ENGINE_RPM)
    }

    /// Request vehicle speed
    #[must_use]
    pub fn vehicle_speed() -> Self {
        Self::new(Obd2Mode::CurrentData, Obd2Pid::VEHICLE_SPEED)
    }

    /// Request coolant temperature
    #[must_use]
    pub fn coolant_temp() -> Self {
        Self::new(Obd2Mode::CurrentData, Obd2Pid::COOLANT_TEMP)
    }

    /// Request stored DTCs
    #[must_use]
    pub fn stored_dtcs() -> Self {
        Self::new_no_pid(Obd2Mode::StoredDtcs)
    }

    /// Request pending DTCs
    #[must_use]
    pub fn pending_dtcs() -> Self {
        Self::new_no_pid(Obd2Mode::PendingDtcs)
    }

    /// Clear DTCs
    #[must_use]
    pub fn clear_dtcs() -> Self {
        Self::new_no_pid(Obd2Mode::ClearDtcs)
    }

    /// Request VIN
    #[must_use]
    pub fn vin() -> Self {
        Self::new(Obd2Mode::VehicleInformation, VehicleInfoPid::VIN)
    }

    /// Get the mode
    #[must_use]
    pub fn mode(&self) -> Obd2Mode {
        self.mode
    }

    /// Get the PID
    #[must_use]
    pub fn pid(&self) -> Option<u8> {
        self.pid
    }

    /// Encode the request to bytes
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + self.data.len());
        bytes.push(self.mode as u8);
        if let Some(pid) = self.pid {
            bytes.push(pid);
        }
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

/// OBD-II Response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Obd2Response {
    /// Current data response (Mode 01)
    CurrentData {
        /// PID
        pid: u8,
        /// Data bytes
        data: Vec<u8>,
    },
    /// Freeze frame data response (Mode 02)
    FreezeFrameData {
        /// PID
        pid: u8,
        /// Frame number
        frame: u8,
        /// Data bytes
        data: Vec<u8>,
    },
    /// DTC response (Mode 03, 07, 0A)
    Dtcs {
        /// List of DTCs
        dtcs: Vec<Obd2Dtc>,
    },
    /// Clear DTCs response (Mode 04)
    ClearDtcsAck,
    /// Vehicle information response (Mode 09)
    VehicleInfo {
        /// Info PID
        pid: u8,
        /// Data bytes
        data: Vec<u8>,
    },
    /// Negative response
    Negative {
        /// Mode that was rejected
        mode: Obd2Mode,
    },
}

impl Obd2Response {
    /// Parse an OBD-II response from bytes
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let response_mode = bytes[0];

        // Check for negative response (0x7F)
        if response_mode == 0x7F && bytes.len() >= 2 {
            let mode = Obd2Mode::from_byte(bytes[1])?;
            return Some(Self::Negative { mode });
        }

        // Positive response (mode + 0x40)
        if response_mode < 0x40 {
            return None;
        }

        let mode = Obd2Mode::from_byte(response_mode - 0x40)?;

        match mode {
            Obd2Mode::CurrentData => {
                if bytes.len() < 2 {
                    return None;
                }
                Some(Self::CurrentData {
                    pid: bytes[1],
                    data: bytes[2..].to_vec(),
                })
            },
            Obd2Mode::FreezeFrameData => {
                if bytes.len() < 3 {
                    return None;
                }
                Some(Self::FreezeFrameData {
                    pid: bytes[1],
                    frame: bytes[2],
                    data: bytes[3..].to_vec(),
                })
            },
            Obd2Mode::StoredDtcs | Obd2Mode::PendingDtcs | Obd2Mode::PermanentDtcs => {
                // First byte after mode is number of DTCs (or padding)
                let dtc_bytes = if bytes.len() > 1 { &bytes[1..] } else { &[] };
                let mut dtcs = Vec::new();

                // DTCs are 2 bytes each
                for chunk in dtc_bytes.chunks(2) {
                    if chunk.len() == 2 && (chunk[0] != 0 || chunk[1] != 0) {
                        dtcs.push(Obd2Dtc::new(chunk[0], chunk[1]));
                    }
                }

                Some(Self::Dtcs { dtcs })
            },
            Obd2Mode::ClearDtcs => Some(Self::ClearDtcsAck),
            Obd2Mode::VehicleInformation => {
                if bytes.len() < 2 {
                    return None;
                }
                Some(Self::VehicleInfo {
                    pid: bytes[1],
                    data: bytes[2..].to_vec(),
                })
            },
            _ => None,
        }
    }
}

/// Decode engine RPM from OBD-II response data
///
/// Formula: ((A * 256) + B) / 4
#[must_use]
pub fn decode_engine_rpm(data: &[u8]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    Some((((data[0] as u16) * 256 + (data[1] as u16)) as f64) / 4.0)
}

/// Decode vehicle speed from OBD-II response data (km/h)
///
/// Formula: A
#[must_use]
pub fn decode_vehicle_speed(data: &[u8]) -> Option<u8> {
    data.first().copied()
}

/// Decode coolant temperature from OBD-II response data (°C)
///
/// Formula: A - 40
#[must_use]
pub fn decode_coolant_temp(data: &[u8]) -> Option<i16> {
    data.first().map(|&a| (a as i16) - 40)
}

/// Decode engine load from OBD-II response data (%)
///
/// Formula: A * 100 / 255
#[must_use]
pub fn decode_engine_load(data: &[u8]) -> Option<f64> {
    data.first().map(|&a| (a as f64) * 100.0 / 255.0)
}

/// Decode throttle position from OBD-II response data (%)
///
/// Formula: A * 100 / 255
#[must_use]
pub fn decode_throttle_position(data: &[u8]) -> Option<f64> {
    data.first().map(|&a| (a as f64) * 100.0 / 255.0)
}

/// Decode intake air temperature from OBD-II response data (°C)
///
/// Formula: A - 40
#[must_use]
pub fn decode_intake_air_temp(data: &[u8]) -> Option<i16> {
    data.first().map(|&a| (a as i16) - 40)
}

/// Decode MAF air flow rate from OBD-II response data (g/s)
///
/// Formula: ((A * 256) + B) / 100
#[must_use]
pub fn decode_maf_flow_rate(data: &[u8]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    Some((((data[0] as u16) * 256 + (data[1] as u16)) as f64) / 100.0)
}

/// Decode fuel tank level from OBD-II response data (%)
///
/// Formula: A * 100 / 255
#[must_use]
pub fn decode_fuel_tank_level(data: &[u8]) -> Option<f64> {
    data.first().map(|&a| (a as f64) * 100.0 / 255.0)
}

/// Decode timing advance from OBD-II response data (degrees before TDC)
///
/// Formula: (A / 2) - 64
#[must_use]
pub fn decode_timing_advance(data: &[u8]) -> Option<f64> {
    data.first().map(|&a| ((a as f64) / 2.0) - 64.0)
}

/// Decode fuel pressure from OBD-II response data (kPa)
///
/// Formula: A * 3
#[must_use]
pub fn decode_fuel_pressure(data: &[u8]) -> Option<u16> {
    data.first().map(|&a| (a as u16) * 3)
}

/// Decode intake manifold pressure from OBD-II response data (kPa)
///
/// Formula: A
#[must_use]
pub fn decode_intake_map(data: &[u8]) -> Option<u8> {
    data.first().copied()
}

/// Decode control module voltage from OBD-II response data (V)
///
/// Formula: ((A * 256) + B) / 1000
#[must_use]
pub fn decode_control_module_voltage(data: &[u8]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    Some((((data[0] as u16) * 256 + (data[1] as u16)) as f64) / 1000.0)
}

/// Decode run time since engine start from OBD-II response data (seconds)
///
/// Formula: (A * 256) + B
#[must_use]
pub fn decode_run_time(data: &[u8]) -> Option<u16> {
    if data.len() < 2 {
        return None;
    }
    Some((data[0] as u16) * 256 + (data[1] as u16))
}

/// Decode distance traveled with MIL on from OBD-II response data (km)
///
/// Formula: (A * 256) + B
#[must_use]
pub fn decode_distance_with_mil(data: &[u8]) -> Option<u16> {
    if data.len() < 2 {
        return None;
    }
    Some((data[0] as u16) * 256 + (data[1] as u16))
}

/// Decode VIN from OBD-II response data
#[must_use]
pub fn decode_vin(data: &[u8]) -> Option<String> {
    // VIN is 17 ASCII characters, may have padding byte at start
    let vin_bytes = if data.len() > 17 { &data[1..18] } else { data };
    if vin_bytes.len() < 17 {
        return None;
    }
    String::from_utf8(vin_bytes.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obd2_mode_positive_response() {
        assert_eq!(Obd2Mode::CurrentData.positive_response(), 0x41);
        assert_eq!(Obd2Mode::StoredDtcs.positive_response(), 0x43);
        assert_eq!(Obd2Mode::VehicleInformation.positive_response(), 0x49);
    }

    #[test]
    fn test_obd2_request_encode() {
        let request = Obd2Request::new(Obd2Mode::CurrentData, Obd2Pid::ENGINE_RPM);
        assert_eq!(request.encode(), vec![0x01, 0x0C]);

        let request = Obd2Request::stored_dtcs();
        assert_eq!(request.encode(), vec![0x03]);
    }

    #[test]
    fn test_obd2_dtc_parsing() {
        // P0420 - Catalyst System Efficiency Below Threshold
        let dtc = Obd2Dtc::new(0x04, 0x20);
        assert_eq!(dtc.category(), 'P');
        assert_eq!(dtc.digit1(), 0);
        assert_eq!(dtc.digit2(), 4);
        assert_eq!(dtc.digit3(), 2);
        assert_eq!(dtc.digit4(), 0);
        assert_eq!(dtc.to_string(), "P0420");
        assert!(dtc.is_generic());
    }

    #[test]
    fn test_obd2_dtc_from_string() {
        let dtc = Obd2Dtc::from_string("P0420").unwrap();
        assert_eq!(dtc.category(), 'P');
        assert_eq!(dtc.to_string(), "P0420");

        let dtc = Obd2Dtc::from_string("C1234").unwrap();
        assert_eq!(dtc.category(), 'C');
        assert!(dtc.is_manufacturer_specific());

        let dtc = Obd2Dtc::from_string("B0100").unwrap();
        assert_eq!(dtc.category(), 'B');

        let dtc = Obd2Dtc::from_string("U0001").unwrap();
        assert_eq!(dtc.category(), 'U');
    }

    #[test]
    fn test_obd2_dtc_invalid_string() {
        assert!(Obd2Dtc::from_string("X0420").is_none()); // Invalid category
        assert!(Obd2Dtc::from_string("P042").is_none()); // Too short
        assert!(Obd2Dtc::from_string("P04200").is_none()); // Too long
        assert!(Obd2Dtc::from_string("P4420").is_none()); // First digit > 3
    }

    #[test]
    fn test_decode_engine_rpm() {
        // 3000 RPM = (A * 256 + B) / 4 = 12000 / 4
        // 12000 = 0x2EE0, A = 0x2E, B = 0xE0
        let rpm = decode_engine_rpm(&[0x2E, 0xE0]).unwrap();
        assert!((rpm - 3000.0).abs() < 0.1);

        // Idle ~800 RPM
        let rpm = decode_engine_rpm(&[0x0C, 0x80]).unwrap();
        assert!((rpm - 800.0).abs() < 0.1);
    }

    #[test]
    fn test_decode_vehicle_speed() {
        assert_eq!(decode_vehicle_speed(&[0x00]), Some(0));
        assert_eq!(decode_vehicle_speed(&[0x64]), Some(100)); // 100 km/h
        assert_eq!(decode_vehicle_speed(&[0xFF]), Some(255)); // 255 km/h
    }

    #[test]
    fn test_decode_coolant_temp() {
        assert_eq!(decode_coolant_temp(&[0x00]), Some(-40)); // -40°C
        assert_eq!(decode_coolant_temp(&[0x28]), Some(0)); // 0°C
        assert_eq!(decode_coolant_temp(&[0x73]), Some(75)); // 75°C (normal operating)
        assert_eq!(decode_coolant_temp(&[0xFF]), Some(215)); // 215°C (max)
    }

    #[test]
    fn test_decode_engine_load() {
        let load = decode_engine_load(&[0x00]).unwrap();
        assert!((load - 0.0).abs() < 0.1);

        let load = decode_engine_load(&[0x80]).unwrap();
        assert!((load - 50.2).abs() < 0.5); // ~50%

        let load = decode_engine_load(&[0xFF]).unwrap();
        assert!((load - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_decode_maf_flow_rate() {
        // 150 g/s = 15000 / 100
        let maf = decode_maf_flow_rate(&[0x3A, 0x98]).unwrap();
        assert!((maf - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_obd2_response_parse_current_data() {
        // Response to engine RPM request
        let bytes = vec![0x41, 0x0C, 0x2E, 0xE0];
        let response = Obd2Response::parse(&bytes).unwrap();

        match response {
            Obd2Response::CurrentData { pid, data } => {
                assert_eq!(pid, 0x0C);
                assert_eq!(data, vec![0x2E, 0xE0]);
            },
            _ => panic!("Expected CurrentData response"),
        }
    }

    #[test]
    fn test_obd2_response_parse_dtcs() {
        // Response with 2 DTCs: P0420 and P0171
        let bytes = vec![0x43, 0x04, 0x20, 0x01, 0x71];
        let response = Obd2Response::parse(&bytes).unwrap();

        match response {
            Obd2Response::Dtcs { dtcs } => {
                assert_eq!(dtcs.len(), 2);
                assert_eq!(dtcs[0].to_string(), "P0420");
                assert_eq!(dtcs[1].to_string(), "P0171");
            },
            _ => panic!("Expected Dtcs response"),
        }
    }

    #[test]
    fn test_obd2_response_parse_clear_ack() {
        let bytes = vec![0x44];
        let response = Obd2Response::parse(&bytes).unwrap();
        assert!(matches!(response, Obd2Response::ClearDtcsAck));
    }

    #[test]
    fn test_obd2_response_parse_negative() {
        let bytes = vec![0x7F, 0x01, 0x12]; // Mode 01 not supported
        let response = Obd2Response::parse(&bytes).unwrap();

        match response {
            Obd2Response::Negative { mode } => {
                assert_eq!(mode, Obd2Mode::CurrentData);
            },
            _ => panic!("Expected Negative response"),
        }
    }

    #[test]
    fn test_decode_vin() {
        // VIN: "1HGBH41JXMN109186"
        let vin_bytes = b"1HGBH41JXMN109186";
        let vin = decode_vin(vin_bytes).unwrap();
        assert_eq!(vin, "1HGBH41JXMN109186");
    }
}
