//! ECU Database and Auto-Detection Module
//!
//! This module provides a comprehensive database of Electronic Control Units (ECUs)
//! from major automotive suppliers, along with AI-assisted auto-detection capabilities.
//!
//! # Supported ECU Manufacturers
//!
//! - **Bosch** - EDC15, EDC16, EDC17, ME7, MED9, MED17, etc.
//! - **Continental/Siemens** - SIMOS, SID, EMS, etc.
//! - **Denso** - Various Toyota, Honda, Subaru ECUs
//! - **Delphi** - DCM, MT series
//! - **Magneti Marelli** - IAW, MJD series
//! - **Hitachi** - Various Nissan ECUs
//! - **Mitsubishi Electric** - Various Asian OEM ECUs
//! - **Aptiv** (formerly Delphi Technologies)
//! - **ZF** - Transmission and chassis ECUs
//! - **Valeo** - Various European OEM ECUs
//!
//! # Auto-Detection Features
//!
//! - UDS-based ECU scanning (0x7E0-0x7E7 range)
//! - Functional addressing broadcast (0x7DF)
//! - VIN extraction and decoding
//! - ECU identification via ReadDataByIdentifier
//! - A2L file association for measurement/calibration
//! - AI-assisted pattern recognition for unknown ECUs
//!
//! # Example
//!
//! ```ignore
//! use busmaster_db::ecu_database::{EcuDatabase, EcuScanner};
//!
//! // Load the built-in ECU database
//! let db = EcuDatabase::load_builtin();
//!
//! // Scan for ECUs on the bus
//! let scanner = EcuScanner::new(driver);
//! let detected = scanner.scan_all().await?;
//!
//! // Match detected ECUs against database
//! for ecu in detected {
//!     if let Some(info) = db.identify(&ecu) {
//!         println!("Found: {} - {}", info.manufacturer, info.family);
//!     }
//! }
//! ```

#![allow(clippy::too_many_lines)]
#![allow(clippy::useless_vec)]
#![allow(clippy::inefficient_to_string)]
#![allow(clippy::unused_self)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::similar_names)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;


// ============================================================================
// ECU Manufacturer Definitions
// ============================================================================

/// ECU Manufacturer/Supplier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcuManufacturer {
    /// Robert Bosch GmbH - World's largest automotive supplier
    Bosch,
    /// Continental AG (includes former Siemens VDO)
    Continental,
    /// Denso Corporation - Major Japanese supplier
    Denso,
    /// Delphi Technologies (now Aptiv/BorgWarner)
    Delphi,
    /// Magneti Marelli (now Marelli Holdings)
    MagnetiMarelli,
    /// Hitachi Automotive Systems
    Hitachi,
    /// Mitsubishi Electric
    MitsubishiElectric,
    /// ZF Friedrichshafen AG
    Zf,
    /// Valeo SA
    Valeo,
    /// Visteon Corporation
    Visteon,
    /// Hyundai Kefico
    HyundaiKefico,
    /// Keihin Corporation (now part of Hitachi Astemo)
    Keihin,
    /// Marelli (formerly Calsonic Kansei + Magneti Marelli)
    Marelli,
    /// Aptiv (formerly Delphi Automotive)
    Aptiv,
    /// Unknown manufacturer
    Unknown,
}

impl EcuManufacturer {
    /// Get manufacturer name as string
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bosch => "Bosch",
            Self::Continental => "Continental/Siemens",
            Self::Denso => "Denso",
            Self::Delphi => "Delphi",
            Self::MagnetiMarelli => "Magneti Marelli",
            Self::Hitachi => "Hitachi",
            Self::MitsubishiElectric => "Mitsubishi Electric",
            Self::Zf => "ZF",
            Self::Valeo => "Valeo",
            Self::Visteon => "Visteon",
            Self::HyundaiKefico => "Hyundai Kefico",
            Self::Keihin => "Keihin",
            Self::Marelli => "Marelli",
            Self::Aptiv => "Aptiv",
            Self::Unknown => "Unknown",
        }
    }
}


/// ECU Type/Domain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcuType {
    /// Engine Control Module
    Engine,
    /// Transmission Control Module
    Transmission,
    /// Body Control Module
    Body,
    /// Anti-lock Braking System
    Abs,
    /// Electronic Stability Control
    Esc,
    /// Airbag Control Module
    Airbag,
    /// Instrument Cluster
    Cluster,
    /// Infotainment/Head Unit
    Infotainment,
    /// Gateway ECU
    Gateway,
    /// Battery Management System (EV/Hybrid)
    Bms,
    /// Motor Control Unit (EV)
    MotorControl,
    /// Inverter (EV)
    Inverter,
    /// ADAS/Driver Assistance
    Adas,
    /// Steering Control
    Steering,
    /// Climate Control
    Climate,
    /// Lighting Control
    Lighting,
    /// Door Control
    Door,
    /// Seat Control
    Seat,
    /// Parking Assist
    Parking,
    /// Telematics Control Unit
    Telematics,
    /// Other/Unknown
    Other,
}

impl EcuType {
    /// Get ECU type name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Engine => "Engine Control Module",
            Self::Transmission => "Transmission Control Module",
            Self::Body => "Body Control Module",
            Self::Abs => "ABS Control Module",
            Self::Esc => "ESC/ESP Control Module",
            Self::Airbag => "Airbag Control Module",
            Self::Cluster => "Instrument Cluster",
            Self::Infotainment => "Infotainment System",
            Self::Gateway => "Gateway ECU",
            Self::Bms => "Battery Management System",
            Self::MotorControl => "Motor Control Unit",
            Self::Inverter => "Inverter",
            Self::Adas => "ADAS Control Module",
            Self::Steering => "Steering Control Module",
            Self::Climate => "Climate Control Module",
            Self::Lighting => "Lighting Control Module",
            Self::Door => "Door Control Module",
            Self::Seat => "Seat Control Module",
            Self::Parking => "Parking Assist Module",
            Self::Telematics => "Telematics Control Unit",
            Self::Other => "Other",
        }
    }
}


// ============================================================================
// ECU Family Definitions (Major ECU Platforms)
// ============================================================================

/// ECU Family/Platform information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuFamily {
    /// Family name (e.g., "EDC17", "MED17", "SIMOS")
    pub name: String,
    /// Manufacturer
    pub manufacturer: EcuManufacturer,
    /// ECU type/domain
    pub ecu_type: EcuType,
    /// Description
    pub description: String,
    /// Processor type (e.g., "TC1796", "TC1797", "MPC5xx")
    pub processor: Option<String>,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// Year range (start, end)
    pub year_range: Option<(u16, u16)>,
    /// Typical CAN baudrate
    pub can_baudrate: Option<u32>,
    /// Typical diagnostic CAN ID (request)
    pub diag_can_id_request: Option<u32>,
    /// Typical diagnostic CAN ID (response)
    pub diag_can_id_response: Option<u32>,
    /// Known variants
    pub variants: Vec<String>,
}

impl EcuFamily {
    /// Create a new ECU family
    #[must_use]
    pub fn new(name: &str, manufacturer: EcuManufacturer, ecu_type: EcuType) -> Self {
        Self {
            name: name.to_string(),
            manufacturer,
            ecu_type,
            description: String::new(),
            processor: None,
            protocols: Vec::new(),
            year_range: None,
            can_baudrate: None,
            diag_can_id_request: None,
            diag_can_id_response: None,
            variants: Vec::new(),
        }
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set processor
    #[must_use]
    pub fn with_processor(mut self, proc: &str) -> Self {
        self.processor = Some(proc.to_string());
        self
    }

    /// Add protocol
    #[must_use]
    pub fn with_protocol(mut self, protocol: &str) -> Self {
        self.protocols.push(protocol.to_string());
        self
    }

    /// Set year range
    #[must_use]
    pub fn with_years(mut self, start: u16, end: u16) -> Self {
        self.year_range = Some((start, end));
        self
    }

    /// Set diagnostic CAN IDs
    #[must_use]
    pub fn with_diag_ids(mut self, request: u32, response: u32) -> Self {
        self.diag_can_id_request = Some(request);
        self.diag_can_id_response = Some(response);
        self
    }

    /// Add variant
    pub fn add_variant(&mut self, variant: &str) {
        self.variants.push(variant.to_string());
    }
}


// ============================================================================
// ECU Instance (Detected/Known ECU)
// ============================================================================

/// Detected or known ECU instance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuInstance {
    /// Unique identifier
    pub id: String,
    /// ECU family reference
    pub family: Option<String>,
    /// Manufacturer
    pub manufacturer: EcuManufacturer,
    /// ECU type
    pub ecu_type: EcuType,
    /// Part number (OEM)
    pub part_number: Option<String>,
    /// Hardware version
    pub hw_version: Option<String>,
    /// Software version
    pub sw_version: Option<String>,
    /// Calibration version
    pub cal_version: Option<String>,
    /// Boot software version
    pub boot_version: Option<String>,
    /// VIN (if available)
    pub vin: Option<String>,
    /// Diagnostic CAN ID (request)
    pub diag_can_id_request: u32,
    /// Diagnostic CAN ID (response)
    pub diag_can_id_response: u32,
    /// Associated A2L file path
    pub a2l_file: Option<String>,
    /// Associated DBC file path
    pub dbc_file: Option<String>,
    /// Custom properties
    pub properties: HashMap<String, String>,
    /// Detection timestamp
    pub detected_at: Option<String>,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f32,
}

impl EcuInstance {
    /// Create a new ECU instance
    #[must_use]
    pub fn new(id: &str, diag_request: u32, diag_response: u32) -> Self {
        Self {
            id: id.to_string(),
            family: None,
            manufacturer: EcuManufacturer::Unknown,
            ecu_type: EcuType::Other,
            part_number: None,
            hw_version: None,
            sw_version: None,
            cal_version: None,
            boot_version: None,
            vin: None,
            diag_can_id_request: diag_request,
            diag_can_id_response: diag_response,
            a2l_file: None,
            dbc_file: None,
            properties: HashMap::new(),
            detected_at: None,
            confidence: 0.0,
        }
    }

    /// Set manufacturer
    #[must_use]
    pub fn with_manufacturer(mut self, mfr: EcuManufacturer) -> Self {
        self.manufacturer = mfr;
        self
    }

    /// Set ECU type
    #[must_use]
    pub fn with_type(mut self, ecu_type: EcuType) -> Self {
        self.ecu_type = ecu_type;
        self
    }

    /// Set part number
    #[must_use]
    pub fn with_part_number(mut self, pn: &str) -> Self {
        self.part_number = Some(pn.to_string());
        self
    }

    /// Set A2L file
    #[must_use]
    pub fn with_a2l(mut self, path: &str) -> Self {
        self.a2l_file = Some(path.to_string());
        self
    }

    /// Set DBC file
    #[must_use]
    pub fn with_dbc(mut self, path: &str) -> Self {
        self.dbc_file = Some(path.to_string());
        self
    }

    /// Add custom property
    pub fn set_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }
}


// ============================================================================
// UDS Data Identifiers for ECU Identification
// ============================================================================

/// Standard UDS Data Identifiers (DIDs) for ECU identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdsDid;

impl UdsDid {
    /// VIN (Vehicle Identification Number)
    pub const VIN: u16 = 0xF190;
    /// ECU Manufacturing Date
    pub const MFG_DATE: u16 = 0xF18B;
    /// ECU Serial Number
    pub const SERIAL_NUMBER: u16 = 0xF18C;
    /// Vehicle Manufacturer ECU Software Number
    pub const SW_NUMBER: u16 = 0xF188;
    /// Vehicle Manufacturer ECU Software Version
    pub const SW_VERSION: u16 = 0xF189;
    /// System Supplier ECU Hardware Number
    pub const HW_NUMBER: u16 = 0xF191;
    /// System Supplier ECU Hardware Version
    pub const HW_VERSION: u16 = 0xF193;
    /// System Supplier ECU Software Number
    pub const SUPPLIER_SW_NUMBER: u16 = 0xF194;
    /// System Supplier ECU Software Version
    pub const SUPPLIER_SW_VERSION: u16 = 0xF195;
    /// Boot Software Identification
    pub const BOOT_SW_ID: u16 = 0xF180;
    /// Application Software Identification
    pub const APP_SW_ID: u16 = 0xF181;
    /// Application Data Identification
    pub const APP_DATA_ID: u16 = 0xF182;
    /// Boot Software Fingerprint
    pub const BOOT_FINGERPRINT: u16 = 0xF183;
    /// Application Software Fingerprint
    pub const APP_FINGERPRINT: u16 = 0xF184;
    /// Application Data Fingerprint
    pub const DATA_FINGERPRINT: u16 = 0xF185;
    /// Active Diagnostic Session
    pub const ACTIVE_SESSION: u16 = 0xF186;
    /// Vehicle Manufacturer Spare Part Number
    pub const SPARE_PART_NUMBER: u16 = 0xF187;
    /// ECU Calibration Data Identification
    pub const CAL_DATA_ID: u16 = 0xF18A;
    /// System Name or Engine Type
    pub const SYSTEM_NAME: u16 = 0xF197;
    /// Repair Shop Code or Tester Serial Number
    pub const REPAIR_SHOP_CODE: u16 = 0xF198;
    /// Programming Date
    pub const PROGRAMMING_DATE: u16 = 0xF199;
    /// Calibration Repair Shop Code
    pub const CAL_REPAIR_SHOP: u16 = 0xF19A;
    /// Calibration Date
    pub const CAL_DATE: u16 = 0xF19B;
    /// Calibration Equipment Software Number
    pub const CAL_EQUIPMENT_SW: u16 = 0xF19C;
    /// ECU Installation Date
    pub const INSTALL_DATE: u16 = 0xF19D;
    /// ODX File Identifier
    pub const ODX_FILE_ID: u16 = 0xF19E;
    /// Entity Identifier
    pub const ENTITY_ID: u16 = 0xF19F;

    /// Get all standard identification DIDs
    #[must_use]
    pub fn identification_dids() -> Vec<u16> {
        vec![
            Self::VIN,
            Self::SW_NUMBER,
            Self::SW_VERSION,
            Self::HW_NUMBER,
            Self::HW_VERSION,
            Self::SERIAL_NUMBER,
            Self::MFG_DATE,
            Self::BOOT_SW_ID,
            Self::APP_SW_ID,
            Self::CAL_DATA_ID,
            Self::SYSTEM_NAME,
            Self::SPARE_PART_NUMBER,
        ]
    }
}


// ============================================================================
// ECU Database
// ============================================================================

/// Comprehensive ECU Database
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EcuDatabase {
    /// ECU families by name
    pub families: HashMap<String, EcuFamily>,
    /// Known ECU instances
    pub instances: Vec<EcuInstance>,
    /// Part number to family mapping
    pub part_number_map: HashMap<String, String>,
    /// A2L file associations
    pub a2l_associations: HashMap<String, String>,
    /// Database version
    pub version: String,
    /// Last updated timestamp
    pub last_updated: String,
}

impl EcuDatabase {
    /// Create a new empty database
    #[must_use]
    pub fn new() -> Self {
        Self {
            families: HashMap::new(),
            instances: Vec::new(),
            part_number_map: HashMap::new(),
            a2l_associations: HashMap::new(),
            version: "1.0.0".to_string(),
            last_updated: "2026-01-26".to_string(),
        }
    }

    /// Load the built-in ECU database with comprehensive manufacturer data
    #[must_use]
    pub fn load_builtin() -> Self {
        let mut db = Self::new();
        db.load_bosch_families();
        db.load_bosch_families_continued();
        db.load_continental_families();
        db.load_denso_families();
        db.load_delphi_families();
        db.load_magneti_marelli_families();
        db.load_other_families();
        db
    }

    /// Add an ECU family
    pub fn add_family(&mut self, family: EcuFamily) {
        self.families.insert(family.name.clone(), family);
    }

    /// Add an ECU instance
    pub fn add_instance(&mut self, instance: EcuInstance) {
        self.instances.push(instance);
    }

    /// Find family by name
    #[must_use]
    pub fn find_family(&self, name: &str) -> Option<&EcuFamily> {
        self.families.get(name)
    }

    /// Find family by part number prefix
    #[must_use]
    pub fn find_family_by_part_number(&self, part_number: &str) -> Option<&EcuFamily> {
        // Check direct mapping first
        if let Some(family_name) = self.part_number_map.get(part_number) {
            return self.families.get(family_name);
        }

        // Try prefix matching
        for (prefix, family_name) in &self.part_number_map {
            if part_number.starts_with(prefix) {
                return self.families.get(family_name);
            }
        }

        None
    }

    /// Associate A2L file with ECU family
    pub fn associate_a2l(&mut self, family_name: &str, a2l_path: &str) {
        self.a2l_associations
            .insert(family_name.to_string(), a2l_path.to_string());
    }

    /// Get A2L file for family
    #[must_use]
    pub fn get_a2l_for_family(&self, family_name: &str) -> Option<&String> {
        self.a2l_associations.get(family_name)
    }

    /// Get all families for a manufacturer
    #[must_use]
    pub fn families_by_manufacturer(&self, mfr: EcuManufacturer) -> Vec<&EcuFamily> {
        self.families
            .values()
            .filter(|f| f.manufacturer == mfr)
            .collect()
    }

    /// Get all families for an ECU type
    #[must_use]
    pub fn families_by_type(&self, ecu_type: EcuType) -> Vec<&EcuFamily> {
        self.families
            .values()
            .filter(|f| f.ecu_type == ecu_type)
            .collect()
    }
}


// ============================================================================
// Built-in ECU Family Data - Bosch
// ============================================================================

impl EcuDatabase {
    fn load_bosch_families(&mut self) {
        // EDC15 - Diesel, older generation
        let mut edc15 = EcuFamily::new("EDC15", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Electronic Diesel Control 15 - Common rail diesel")
            .with_processor("C167")
            .with_protocol("KWP2000")
            .with_protocol("CAN")
            .with_years(1998, 2008);
        edc15.variants = vec![
            "EDC15C0", "EDC15C2", "EDC15C3", "EDC15C4", "EDC15C5", "EDC15C6", "EDC15C7",
            "EDC15M", "EDC15P", "EDC15P+", "EDC15V", "EDC15VM+",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(edc15);

        // EDC16 - Diesel, mid generation
        let mut edc16 = EcuFamily::new("EDC16", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Electronic Diesel Control 16 - Common rail diesel")
            .with_processor("MPC555/MPC556")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2004, 2012)
            .with_diag_ids(0x7E0, 0x7E8);
        edc16.variants = vec![
            "EDC16C0", "EDC16C2", "EDC16C3", "EDC16C4", "EDC16C8", "EDC16C9",
            "EDC16C31", "EDC16C32", "EDC16C34", "EDC16C35", "EDC16C36", "EDC16C39",
            "EDC16CP31", "EDC16CP33", "EDC16CP34", "EDC16CP35",
            "EDC16U1", "EDC16U31", "EDC16U34",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(edc16);

        // EDC17 - Diesel, current generation
        let mut edc17 = EcuFamily::new("EDC17", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Electronic Diesel Control 17 - Latest diesel platform")
            .with_processor("TC1766/TC1796/TC1797")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_protocol("CAN-FD")
            .with_years(2008, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        edc17.variants = vec![
            "EDC17C01", "EDC17C06", "EDC17C08", "EDC17C10", "EDC17C11",
            "EDC17C41", "EDC17C43", "EDC17C44", "EDC17C45", "EDC17C46", "EDC17C47",
            "EDC17C49", "EDC17C50", "EDC17C53", "EDC17C54", "EDC17C55", "EDC17C56",
            "EDC17C57", "EDC17C60", "EDC17C64", "EDC17C69", "EDC17C74",
            "EDC17CP04", "EDC17CP14", "EDC17CP20", "EDC17CP24", "EDC17CP44",
            "EDC17CP45", "EDC17CP46", "EDC17CP47", "EDC17CP52", "EDC17CP54",
            "EDC17U01", "EDC17U05",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(edc17);

        // ME7 - Gasoline, older generation
        let mut me7 = EcuFamily::new("ME7", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Motronic ME7 - Gasoline engine management")
            .with_processor("C167")
            .with_protocol("KWP2000")
            .with_protocol("CAN")
            .with_years(1999, 2010);
        me7.variants = vec![
            "ME7.0", "ME7.1", "ME7.1.1", "ME7.2", "ME7.3", "ME7.3.1", "ME7.3.2",
            "ME7.4", "ME7.4.1", "ME7.5", "ME7.5.1", "ME7.5.5", "ME7.5.10",
            "ME7.6", "ME7.6.1", "ME7.6.2", "ME7.7", "ME7.8",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(me7);
    }
}


impl EcuDatabase {
    fn load_bosch_families_continued(&mut self) {
        // MED9 - Gasoline Direct Injection
        let mut med9 = EcuFamily::new("MED9", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Motronic MED9 - Gasoline Direct Injection")
            .with_processor("MPC555/MPC556")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2004, 2012);
        med9.variants = vec![
            "MED9.1", "MED9.1.1", "MED9.5", "MED9.5.10",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(med9);

        // MED17 - Gasoline, current generation
        let mut med17 = EcuFamily::new("MED17", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Motronic MED17 - Latest gasoline platform")
            .with_processor("TC1766/TC1796/TC1797")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_protocol("CAN-FD")
            .with_years(2008, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        med17.variants = vec![
            "MED17.1", "MED17.1.1", "MED17.1.6", "MED17.1.10", "MED17.1.21", "MED17.1.27",
            "MED17.3", "MED17.3.1", "MED17.3.5",
            "MED17.5", "MED17.5.1", "MED17.5.2", "MED17.5.5", "MED17.5.20", "MED17.5.21",
            "MED17.7", "MED17.7.1", "MED17.7.2", "MED17.7.3", "MED17.7.5",
            "MED17.8", "MED17.8.10", "MED17.8.31", "MED17.8.32",
            "MED17.9", "MED17.9.3", "MED17.9.7", "MED17.9.8",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(med17);

        // MEVD17 - Gasoline Direct Injection with Valvetronic
        let mut mevd17 = EcuFamily::new("MEVD17", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Motronic MEVD17 - GDI with variable valve timing")
            .with_processor("TC1797")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2010, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        mevd17.variants = vec![
            "MEVD17.2", "MEVD17.2.4", "MEVD17.2.6", "MEVD17.2.8", "MEVD17.2.9",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(mevd17);

        // MG1 - Next generation gasoline
        let mut mg1 = EcuFamily::new("MG1", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Motronic MG1 - Next generation gasoline platform")
            .with_processor("TC2xx")
            .with_protocol("UDS")
            .with_protocol("CAN-FD")
            .with_years(2016, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        mg1.variants = vec![
            "MG1CS001", "MG1CS002", "MG1CS003", "MG1CS011", "MG1CS024",
            "MG1CA002", "MG1CA007",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(mg1);

        // MD1 - Next generation diesel
        let mut md1 = EcuFamily::new("MD1", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("MD1 - Next generation diesel platform")
            .with_processor("TC2xx")
            .with_protocol("UDS")
            .with_protocol("CAN-FD")
            .with_years(2016, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        md1.variants = vec![
            "MD1CS001", "MD1CS003", "MD1CS004", "MD1CS006",
            "MD1CP001", "MD1CP002", "MD1CP004",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(md1);

        // ABS/ESP
        let abs9 = EcuFamily::new("ABS9", EcuManufacturer::Bosch, EcuType::Abs)
            .with_description("Bosch ABS 9 - Anti-lock braking system")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2010, 2026)
            .with_diag_ids(0x7E2, 0x7EA);
        self.add_family(abs9);

        let esp9 = EcuFamily::new("ESP9", EcuManufacturer::Bosch, EcuType::Esc)
            .with_description("Bosch ESP 9 - Electronic Stability Program")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2010, 2026)
            .with_diag_ids(0x7E2, 0x7EA);
        self.add_family(esp9);
    }
}


// ============================================================================
// Built-in ECU Family Data - Continental/Siemens
// ============================================================================

impl EcuDatabase {
    fn load_continental_families(&mut self) {
        // SIMOS - Gasoline
        let mut simos = EcuFamily::new("SIMOS", EcuManufacturer::Continental, EcuType::Engine)
            .with_description("Siemens/Continental SIMOS - Gasoline engine management")
            .with_processor("TC1766/TC1796/TC1797")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        simos.variants = vec![
            "SIMOS3.3", "SIMOS3.4", "SIMOS6.1", "SIMOS6.2", "SIMOS6.3",
            "SIMOS7.1", "SIMOS7.6", "SIMOS8.1", "SIMOS8.2", "SIMOS8.3", "SIMOS8.4", "SIMOS8.5",
            "SIMOS10", "SIMOS10.1", "SIMOS10.2",
            "SIMOS12", "SIMOS12.1", "SIMOS12.2",
            "SIMOS18", "SIMOS18.1", "SIMOS18.10", "SIMOS18.41",
            "SIMOS19", "SIMOS19.1", "SIMOS19.3", "SIMOS19.6",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(simos);

        // SID - Diesel
        let mut sid = EcuFamily::new("SID", EcuManufacturer::Continental, EcuType::Engine)
            .with_description("Siemens/Continental SID - Diesel engine management")
            .with_processor("MPC5xx/TC1xxx")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2003, 2020)
            .with_diag_ids(0x7E0, 0x7E8);
        sid.variants = vec![
            "SID201", "SID202", "SID203", "SID204", "SID206", "SID208", "SID209",
            "SID301", "SID302", "SID303", "SID305", "SID306", "SID307", "SID309", "SID310",
            "SID801", "SID802", "SID803", "SID804", "SID805", "SID806", "SID807",
            "SID901", "SID902", "SID903",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(sid);

        // EMS - Engine Management System
        let mut ems = EcuFamily::new("EMS", EcuManufacturer::Continental, EcuType::Engine)
            .with_description("Continental EMS - Engine Management System")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2010, 2026);
        ems.variants = vec![
            "EMS2", "EMS2.1", "EMS2.2", "EMS2.3", "EMS2.4",
            "EMS3", "EMS3.1", "EMS3.2", "EMS3.3",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(ems);

        // VDO - Instrument Cluster
        let vdo = EcuFamily::new("VDO", EcuManufacturer::Continental, EcuType::Cluster)
            .with_description("Continental VDO - Instrument Cluster")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2000, 2026)
            .with_diag_ids(0x7E4, 0x7EC);
        self.add_family(vdo);

        // MK100 - ABS/ESC
        let mk100 = EcuFamily::new("MK100", EcuManufacturer::Continental, EcuType::Esc)
            .with_description("Continental MK100 - ABS/ESC System")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2010, 2026)
            .with_diag_ids(0x7E2, 0x7EA);
        self.add_family(mk100);
    }
}


// ============================================================================
// Built-in ECU Family Data - Denso
// ============================================================================

impl EcuDatabase {
    fn load_denso_families(&mut self) {
        // Toyota/Lexus ECUs
        let mut denso_toyota = EcuFamily::new("DENSO_TOYOTA", EcuManufacturer::Denso, EcuType::Engine)
            .with_description("Denso ECUs for Toyota/Lexus vehicles")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        denso_toyota.variants = vec![
            "89661-xxxxx", // Toyota part number format
            "275xxx", // Denso internal format
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(denso_toyota);

        // Honda ECUs
        let mut denso_honda = EcuFamily::new("DENSO_HONDA", EcuManufacturer::Denso, EcuType::Engine)
            .with_description("Denso ECUs for Honda/Acura vehicles")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        denso_honda.variants = vec![
            "37820-xxx-xxx", // Honda part number format
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(denso_honda);

        // Subaru ECUs
        let mut denso_subaru = EcuFamily::new("DENSO_SUBARU", EcuManufacturer::Denso, EcuType::Engine)
            .with_description("Denso ECUs for Subaru vehicles")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        denso_subaru.variants = vec![
            "22611-xxxxx", // Subaru part number format
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(denso_subaru);

        // Mazda ECUs
        let denso_mazda = EcuFamily::new("DENSO_MAZDA", EcuManufacturer::Denso, EcuType::Engine)
            .with_description("Denso ECUs for Mazda vehicles")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        self.add_family(denso_mazda);

        // Generic Denso
        let mut genk = EcuFamily::new("GENK", EcuManufacturer::Denso, EcuType::Engine)
            .with_description("Denso GENK - Generic Engine Control")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2010, 2026);
        genk.variants = vec![
            "GENK1", "GENK2", "GENK3", "GENK4",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(genk);
    }
}


// ============================================================================
// Built-in ECU Family Data - Delphi
// ============================================================================

impl EcuDatabase {
    fn load_delphi_families(&mut self) {
        // DCM - Diesel Control Module
        let mut dcm = EcuFamily::new("DCM", EcuManufacturer::Delphi, EcuType::Engine)
            .with_description("Delphi DCM - Diesel Control Module")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2020)
            .with_diag_ids(0x7E0, 0x7E8);
        dcm.variants = vec![
            "DCM3.2", "DCM3.4", "DCM3.5", "DCM3.7",
            "DCM5", "DCM5.1", "DCM5.2",
            "DCM6.1", "DCM6.2", "DCM6.2C",
            "DCM7.1", "DCM7.1A",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(dcm);

        // MT - Multec
        let mut mt = EcuFamily::new("MT", EcuManufacturer::Delphi, EcuType::Engine)
            .with_description("Delphi Multec - Gasoline engine management")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2000, 2015);
        mt.variants = vec![
            "MT35", "MT35E", "MT38", "MT39",
            "MT60", "MT80", "MT86",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(mt);

        // Delco - GM ECUs
        let mut delco = EcuFamily::new("DELCO", EcuManufacturer::Delphi, EcuType::Engine)
            .with_description("Delphi/Delco - GM Engine Control")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(1995, 2020);
        delco.variants = vec![
            "E38", "E40", "E67", "E78", "E83", "E92",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(delco);
    }
}


// ============================================================================
// Built-in ECU Family Data - Magneti Marelli
// ============================================================================

impl EcuDatabase {
    fn load_magneti_marelli_families(&mut self) {
        // IAW - Integrated Automatic Wiring
        let mut iaw = EcuFamily::new("IAW", EcuManufacturer::MagnetiMarelli, EcuType::Engine)
            .with_description("Magneti Marelli IAW - Gasoline engine management")
            .with_protocol("KWP2000")
            .with_protocol("CAN")
            .with_years(1995, 2015);
        iaw.variants = vec![
            "IAW4AF", "IAW4CF", "IAW4DF", "IAW4EF", "IAW4FF", "IAW4GF",
            "IAW4MV", "IAW4SF", "IAW4TV", "IAW4VP",
            "IAW5AF", "IAW5AM", "IAW5NF", "IAW5NP", "IAW5SF", "IAW5NR",
            "IAW6LP", "IAW6LPB",
            "IAW7GF", "IAW7SM",
            "IAW8F", "IAW8GMF", "IAW8P",
            "IAW16F", "IAW18F", "IAW18FD", "IAW48P", "IAW49F", "IAW59F",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(iaw);

        // MJD - Multijet Diesel
        let mut mjd = EcuFamily::new("MJD", EcuManufacturer::MagnetiMarelli, EcuType::Engine)
            .with_description("Magneti Marelli MJD - Multijet Diesel")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2020)
            .with_diag_ids(0x7E0, 0x7E8);
        mjd.variants = vec![
            "MJD6F3", "MJD6JF", "MJD6JX", "MJD6O1", "MJD6O2",
            "MJD8F2", "MJD8F3", "MJD8DF",
            "MJD9DF",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(mjd);

        // 4CE - Cluster
        let ce4 = EcuFamily::new("4CE", EcuManufacturer::MagnetiMarelli, EcuType::Cluster)
            .with_description("Magneti Marelli 4CE - Instrument Cluster")
            .with_protocol("CAN")
            .with_years(2005, 2020);
        self.add_family(ce4);
    }
}


// ============================================================================
// Built-in ECU Family Data - Other Manufacturers
// ============================================================================

impl EcuDatabase {
    fn load_other_families(&mut self) {
        // Hitachi - Nissan
        let mut hitachi = EcuFamily::new("HITACHI", EcuManufacturer::Hitachi, EcuType::Engine)
            .with_description("Hitachi ECUs for Nissan/Infiniti vehicles")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        hitachi.variants = vec![
            "MEC", "MEC30", "MEC32", "MEC37",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(hitachi);

        // Keihin - Honda
        let mut keihin = EcuFamily::new("KEIHIN", EcuManufacturer::Keihin, EcuType::Engine)
            .with_description("Keihin ECUs for Honda/Acura vehicles")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2000, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        keihin.variants = vec![
            "SH7055", "SH7058", "SH7059",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(keihin);

        // Hyundai Kefico
        let mut kefico = EcuFamily::new("KEFICO", EcuManufacturer::HyundaiKefico, EcuType::Engine)
            .with_description("Hyundai Kefico - Hyundai/Kia engine management")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E0, 0x7E8);
        kefico.variants = vec![
            "CPGDSH", "CPGDSH2", "CPGDSH3",
            "SIMK41", "SIMK43", "SIMK47",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(kefico);

        // ZF - Transmission
        let mut zf_trans = EcuFamily::new("ZF_TRANS", EcuManufacturer::Zf, EcuType::Transmission)
            .with_description("ZF Transmission Control Units")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E1, 0x7E9);
        zf_trans.variants = vec![
            "6HP", "8HP", "9HP",
            "GS19", "GS21", "GS24",
        ].iter().map(|s| s.to_string()).collect();
        self.add_family(zf_trans);

        // ZF - Steering
        let zf_steer = EcuFamily::new("ZF_EPS", EcuManufacturer::Zf, EcuType::Steering)
            .with_description("ZF Electric Power Steering")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2010, 2026)
            .with_diag_ids(0x7E3, 0x7EB);
        self.add_family(zf_steer);

        // Valeo
        let valeo = EcuFamily::new("VALEO", EcuManufacturer::Valeo, EcuType::Other)
            .with_description("Valeo Automotive Systems")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026);
        self.add_family(valeo);

        // Visteon
        let visteon = EcuFamily::new("VISTEON", EcuManufacturer::Visteon, EcuType::Cluster)
            .with_description("Visteon Instrument Clusters")
            .with_protocol("UDS")
            .with_protocol("CAN")
            .with_years(2005, 2026)
            .with_diag_ids(0x7E4, 0x7EC);
        self.add_family(visteon);
    }
}


// ============================================================================
// ECU Scanner Configuration
// ============================================================================

/// ECU Scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcuScanConfig {
    /// Start CAN ID for scanning (default: 0x7E0)
    pub start_id: u32,
    /// End CAN ID for scanning (default: 0x7E7)
    pub end_id: u32,
    /// Use functional addressing (0x7DF broadcast)
    pub use_functional_addressing: bool,
    /// Timeout per ECU in milliseconds
    pub timeout_ms: u32,
    /// DIDs to read for identification
    pub identification_dids: Vec<u16>,
    /// Extended ID range scanning
    pub scan_extended_range: bool,
    /// Extended range start (if enabled)
    pub extended_start: u32,
    /// Extended range end (if enabled)
    pub extended_end: u32,
}

impl Default for EcuScanConfig {
    fn default() -> Self {
        Self {
            start_id: 0x7E0,
            end_id: 0x7E7,
            use_functional_addressing: true,
            timeout_ms: 1000,
            identification_dids: UdsDid::identification_dids(),
            scan_extended_range: false,
            extended_start: 0x600,
            extended_end: 0x6FF,
        }
    }
}

impl EcuScanConfig {
    /// Create a new scan configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set ID range
    #[must_use]
    pub fn with_id_range(mut self, start: u32, end: u32) -> Self {
        self.start_id = start;
        self.end_id = end;
        self
    }

    /// Enable extended range scanning
    #[must_use]
    pub fn with_extended_range(mut self, start: u32, end: u32) -> Self {
        self.scan_extended_range = true;
        self.extended_start = start;
        self.extended_end = end;
        self
    }

    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}


// ============================================================================
// ECU Detection Result
// ============================================================================

/// Result of ECU detection/scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcuDetectionResult {
    /// Detected ECU instance
    pub ecu: EcuInstance,
    /// Raw identification data
    pub raw_data: HashMap<u16, Vec<u8>>,
    /// Matched family (if found in database)
    pub matched_family: Option<String>,
    /// Detection method used
    pub detection_method: DetectionMethod,
    /// AI suggestions for unknown ECUs
    pub ai_suggestions: Vec<AiSuggestion>,
}

/// Detection method used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// UDS ReadDataByIdentifier
    UdsReadDid,
    /// Functional addressing broadcast
    FunctionalBroadcast,
    /// OBD-II mode 09
    ObdMode09,
    /// Pattern matching from traffic
    TrafficAnalysis,
    /// A2L file association
    A2lAssociation,
    /// Manual configuration
    Manual,
}

/// AI suggestion for ECU identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    /// Suggested manufacturer
    pub manufacturer: EcuManufacturer,
    /// Suggested family
    pub family: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Reasoning
    pub reasoning: String,
    /// Suggested A2L files to try
    pub suggested_a2l_files: Vec<String>,
}

impl EcuDetectionResult {
    /// Create a new detection result
    #[must_use]
    pub fn new(ecu: EcuInstance, method: DetectionMethod) -> Self {
        Self {
            ecu,
            raw_data: HashMap::new(),
            matched_family: None,
            detection_method: method,
            ai_suggestions: Vec::new(),
        }
    }

    /// Add raw DID data
    pub fn add_raw_data(&mut self, did: u16, data: Vec<u8>) {
        self.raw_data.insert(did, data);
    }

    /// Set matched family
    pub fn set_matched_family(&mut self, family: &str) {
        self.matched_family = Some(family.to_string());
    }

    /// Add AI suggestion
    pub fn add_suggestion(&mut self, suggestion: AiSuggestion) {
        self.ai_suggestions.push(suggestion);
    }

    /// Get VIN from raw data
    #[must_use]
    pub fn get_vin(&self) -> Option<String> {
        self.raw_data.get(&UdsDid::VIN).and_then(|data| {
            // VIN is typically ASCII, skip first byte (length)
            if data.len() > 1 {
                String::from_utf8(data[1..].to_vec()).ok()
            } else {
                None
            }
        })
    }

    /// Get software version from raw data
    #[must_use]
    pub fn get_sw_version(&self) -> Option<String> {
        self.raw_data.get(&UdsDid::SW_VERSION).and_then(|data| {
            String::from_utf8(data.clone()).ok()
        })
    }

    /// Get hardware version from raw data
    #[must_use]
    pub fn get_hw_version(&self) -> Option<String> {
        self.raw_data.get(&UdsDid::HW_VERSION).and_then(|data| {
            String::from_utf8(data.clone()).ok()
        })
    }
}


// ============================================================================
// ECU Identifier (AI-Assisted Pattern Matching)
// ============================================================================

/// ECU Identifier with AI-assisted pattern matching
#[derive(Debug, Clone, Default)]
pub struct EcuIdentifier {
    /// Reference database
    database: EcuDatabase,
    /// Part number patterns for manufacturer identification
    part_number_patterns: Vec<(String, EcuManufacturer)>,
    /// Software version patterns
    sw_version_patterns: Vec<(String, String)>,
}

impl EcuIdentifier {
    /// Create a new ECU identifier with built-in database
    #[must_use]
    pub fn new() -> Self {
        let mut identifier = Self {
            database: EcuDatabase::load_builtin(),
            part_number_patterns: Vec::new(),
            sw_version_patterns: Vec::new(),
        };
        identifier.load_patterns();
        identifier
    }

    /// Create with custom database
    #[must_use]
    pub fn with_database(database: EcuDatabase) -> Self {
        let mut identifier = Self {
            database,
            part_number_patterns: Vec::new(),
            sw_version_patterns: Vec::new(),
        };
        identifier.load_patterns();
        identifier
    }

    fn load_patterns(&mut self) {
        // Bosch part number patterns
        self.part_number_patterns.push(("0261".to_string(), EcuManufacturer::Bosch)); // ME/MED
        self.part_number_patterns.push(("0281".to_string(), EcuManufacturer::Bosch)); // EDC
        self.part_number_patterns.push(("026".to_string(), EcuManufacturer::Bosch));
        self.part_number_patterns.push(("028".to_string(), EcuManufacturer::Bosch));

        // Continental/Siemens patterns
        self.part_number_patterns.push(("5WK".to_string(), EcuManufacturer::Continental));
        self.part_number_patterns.push(("5WP".to_string(), EcuManufacturer::Continental));
        self.part_number_patterns.push(("5WS".to_string(), EcuManufacturer::Continental));

        // Denso patterns
        self.part_number_patterns.push(("89661".to_string(), EcuManufacturer::Denso)); // Toyota
        self.part_number_patterns.push(("37820".to_string(), EcuManufacturer::Denso)); // Honda
        self.part_number_patterns.push(("22611".to_string(), EcuManufacturer::Denso)); // Subaru
        self.part_number_patterns.push(("275".to_string(), EcuManufacturer::Denso));

        // Delphi patterns
        self.part_number_patterns.push(("12".to_string(), EcuManufacturer::Delphi)); // GM
        self.part_number_patterns.push(("28".to_string(), EcuManufacturer::Delphi));

        // Magneti Marelli patterns
        self.part_number_patterns.push(("IAW".to_string(), EcuManufacturer::MagnetiMarelli));
        self.part_number_patterns.push(("MJD".to_string(), EcuManufacturer::MagnetiMarelli));
        self.part_number_patterns.push(("61601".to_string(), EcuManufacturer::MagnetiMarelli));

        // Software version patterns for family identification
        self.sw_version_patterns.push(("EDC17".to_string(), "EDC17".to_string()));
        self.sw_version_patterns.push(("EDC16".to_string(), "EDC16".to_string()));
        self.sw_version_patterns.push(("EDC15".to_string(), "EDC15".to_string()));
        self.sw_version_patterns.push(("MED17".to_string(), "MED17".to_string()));
        self.sw_version_patterns.push(("MED9".to_string(), "MED9".to_string()));
        self.sw_version_patterns.push(("ME7".to_string(), "ME7".to_string()));
        self.sw_version_patterns.push(("SIMOS".to_string(), "SIMOS".to_string()));
        self.sw_version_patterns.push(("SID".to_string(), "SID".to_string()));
    }

    /// Identify ECU from detection result
    #[must_use]
    pub fn identify(&self, result: &EcuDetectionResult) -> Option<IdentificationResult> {
        let mut identification = IdentificationResult::default();

        // Try to identify manufacturer from part number
        if let Some(pn) = &result.ecu.part_number {
            for (pattern, mfr) in &self.part_number_patterns {
                if pn.contains(pattern) {
                    identification.manufacturer = Some(*mfr);
                    identification.confidence += 0.3;
                    break;
                }
            }
        }

        // Try to identify family from software version
        if let Some(sw) = &result.ecu.sw_version {
            for (pattern, family) in &self.sw_version_patterns {
                if sw.to_uppercase().contains(pattern) {
                    identification.family = Some(family.clone());
                    identification.confidence += 0.3;
                    break;
                }
            }
        }

        // Look up in database
        if let Some(family_name) = &identification.family {
            if let Some(family) = self.database.find_family(family_name) {
                identification.family_info = Some(family.clone());
                identification.confidence += 0.2;
            }
        }

        // Check A2L associations
        if let Some(family_name) = &identification.family {
            if let Some(a2l) = self.database.get_a2l_for_family(family_name) {
                identification.suggested_a2l = Some(a2l.clone());
                identification.confidence += 0.1;
            }
        }

        if identification.confidence > 0.0 {
            Some(identification)
        } else {
            None
        }
    }

    /// Generate AI suggestions for unknown ECU
    #[must_use]
    pub fn generate_suggestions(&self, result: &EcuDetectionResult) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // Analyze raw data patterns
        for (did, data) in &result.raw_data {
            if let Some(suggestion) = self.analyze_did_data(*did, data) {
                suggestions.push(suggestion);
            }
        }

        // Sort by confidence
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        suggestions
    }

    fn analyze_did_data(&self, did: u16, data: &[u8]) -> Option<AiSuggestion> {
        // Convert to string for pattern matching
        let data_str = String::from_utf8_lossy(data);

        // Check for known patterns
        if data_str.contains("BOSCH") || data_str.contains("Bosch") {
            return Some(AiSuggestion {
                manufacturer: EcuManufacturer::Bosch,
                family: None,
                confidence: 0.8,
                reasoning: format!("DID 0x{:04X} contains 'BOSCH' identifier", did),
                suggested_a2l_files: vec![],
            });
        }

        if data_str.contains("CONTINENTAL") || data_str.contains("SIEMENS") {
            return Some(AiSuggestion {
                manufacturer: EcuManufacturer::Continental,
                family: None,
                confidence: 0.8,
                reasoning: format!("DID 0x{:04X} contains Continental/Siemens identifier", did),
                suggested_a2l_files: vec![],
            });
        }

        if data_str.contains("DENSO") {
            return Some(AiSuggestion {
                manufacturer: EcuManufacturer::Denso,
                family: None,
                confidence: 0.8,
                reasoning: format!("DID 0x{:04X} contains 'DENSO' identifier", did),
                suggested_a2l_files: vec![],
            });
        }

        None
    }

    /// Get reference to database
    #[must_use]
    pub fn database(&self) -> &EcuDatabase {
        &self.database
    }
}

/// Result of ECU identification
#[derive(Debug, Clone, Default)]
pub struct IdentificationResult {
    /// Identified manufacturer
    pub manufacturer: Option<EcuManufacturer>,
    /// Identified family name
    pub family: Option<String>,
    /// Family information from database
    pub family_info: Option<EcuFamily>,
    /// Suggested A2L file
    pub suggested_a2l: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}


// ============================================================================
// A2L File Association Manager
// ============================================================================

/// Manager for A2L file associations with ECUs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct A2lAssociationManager {
    /// ECU ID to A2L file path mapping
    associations: HashMap<String, String>,
    /// Family to A2L file path mapping
    family_associations: HashMap<String, String>,
    /// Part number prefix to A2L file mapping
    part_number_associations: HashMap<String, String>,
}

impl A2lAssociationManager {
    /// Create a new association manager
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Associate A2L file with specific ECU ID
    pub fn associate_ecu(&mut self, ecu_id: &str, a2l_path: &str) {
        self.associations.insert(ecu_id.to_string(), a2l_path.to_string());
    }

    /// Associate A2L file with ECU family
    pub fn associate_family(&mut self, family: &str, a2l_path: &str) {
        self.family_associations.insert(family.to_string(), a2l_path.to_string());
    }

    /// Associate A2L file with part number prefix
    pub fn associate_part_number(&mut self, prefix: &str, a2l_path: &str) {
        self.part_number_associations.insert(prefix.to_string(), a2l_path.to_string());
    }

    /// Find A2L file for ECU
    #[must_use]
    pub fn find_a2l(&self, ecu: &EcuInstance) -> Option<&String> {
        // Check direct ECU ID association
        if let Some(a2l) = self.associations.get(&ecu.id) {
            return Some(a2l);
        }

        // Check family association
        if let Some(family) = &ecu.family {
            if let Some(a2l) = self.family_associations.get(family) {
                return Some(a2l);
            }
        }

        // Check part number prefix
        if let Some(pn) = &ecu.part_number {
            for (prefix, a2l) in &self.part_number_associations {
                if pn.starts_with(prefix) {
                    return Some(a2l);
                }
            }
        }

        None
    }

    /// Get all associations
    #[must_use]
    pub fn all_associations(&self) -> Vec<(&String, &String)> {
        self.associations.iter().collect()
    }
}


// ============================================================================
// VIN Decoder
// ============================================================================

/// VIN (Vehicle Identification Number) decoder
#[derive(Debug, Clone, Default)]
pub struct VinDecoder;

impl VinDecoder {
    /// Decode VIN and extract information
    #[must_use]
    pub fn decode(vin: &str) -> Option<VinInfo> {
        if vin.len() != 17 {
            return None;
        }

        let vin = vin.to_uppercase();
        let chars: Vec<char> = vin.chars().collect();

        // World Manufacturer Identifier (WMI) - positions 1-3
        let wmi: String = chars[0..3].iter().collect();

        // Vehicle Descriptor Section (VDS) - positions 4-9
        let vds: String = chars[3..9].iter().collect();

        // Vehicle Identifier Section (VIS) - positions 10-17
        let vis: String = chars[9..17].iter().collect();

        // Model year (position 10)
        let model_year = Self::decode_model_year(chars[9]);

        // Plant code (position 11)
        let plant_code = chars[10];

        // Serial number (positions 12-17)
        let serial: String = chars[11..17].iter().collect();

        // Decode manufacturer from WMI
        let manufacturer = Self::decode_manufacturer(&wmi);

        Some(VinInfo {
            vin: vin.clone(),
            wmi,
            vds,
            vis,
            manufacturer,
            model_year,
            plant_code,
            serial_number: serial,
        })
    }

    fn decode_model_year(c: char) -> Option<u16> {
        match c {
            'A' => Some(2010),
            'B' => Some(2011),
            'C' => Some(2012),
            'D' => Some(2013),
            'E' => Some(2014),
            'F' => Some(2015),
            'G' => Some(2016),
            'H' => Some(2017),
            'J' => Some(2018),
            'K' => Some(2019),
            'L' => Some(2020),
            'M' => Some(2021),
            'N' => Some(2022),
            'P' => Some(2023),
            'R' => Some(2024),
            'S' => Some(2025),
            'T' => Some(2026),
            'V' => Some(2027),
            'W' => Some(2028),
            'X' => Some(2029),
            'Y' => Some(2030),
            '1' => Some(2001),
            '2' => Some(2002),
            '3' => Some(2003),
            '4' => Some(2004),
            '5' => Some(2005),
            '6' => Some(2006),
            '7' => Some(2007),
            '8' => Some(2008),
            '9' => Some(2009),
            _ => None,
        }
    }

    fn decode_manufacturer(wmi: &str) -> Option<String> {
        // Common WMI codes
        match &wmi[0..2] {
            "1G" | "2G" | "3G" => Some("General Motors".to_string()),
            "1F" | "2F" | "3F" => Some("Ford".to_string()),
            "1C" | "2C" | "3C" => Some("Chrysler/FCA".to_string()),
            "JT" => Some("Toyota".to_string()),
            "JH" => Some("Honda".to_string()),
            "JN" => Some("Nissan".to_string()),
            "JM" => Some("Mazda/Mitsubishi".to_string()),
            "KM" | "KN" => Some("Hyundai/Kia".to_string()),
            "WA" | "WV" | "WF" => Some("Volkswagen Group".to_string()),
            "WB" => Some("BMW".to_string()),
            "WD" => Some("Mercedes-Benz".to_string()),
            "WP" => Some("Porsche".to_string()),
            "ZA" | "ZF" => Some("Fiat/Alfa Romeo".to_string()),
            "SA" | "SJ" => Some("Jaguar/Land Rover".to_string()),
            "YV" => Some("Volvo".to_string()),
            "VF" => Some("Renault/Peugeot".to_string()),
            "TR" => Some("Audi Hungary".to_string()),
            "LF" | "LV" | "LS" => Some("Chinese Manufacturer".to_string()),
            _ => None,
        }
    }
}

/// Decoded VIN information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VinInfo {
    /// Full VIN
    pub vin: String,
    /// World Manufacturer Identifier
    pub wmi: String,
    /// Vehicle Descriptor Section
    pub vds: String,
    /// Vehicle Identifier Section
    pub vis: String,
    /// Manufacturer name
    pub manufacturer: Option<String>,
    /// Model year
    pub model_year: Option<u16>,
    /// Plant code
    pub plant_code: char,
    /// Serial number
    pub serial_number: String,
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecu_manufacturer_name() {
        assert_eq!(EcuManufacturer::Bosch.name(), "Bosch");
        assert_eq!(EcuManufacturer::Continental.name(), "Continental/Siemens");
        assert_eq!(EcuManufacturer::Denso.name(), "Denso");
    }

    #[test]
    fn test_ecu_type_name() {
        assert_eq!(EcuType::Engine.name(), "Engine Control Module");
        assert_eq!(EcuType::Transmission.name(), "Transmission Control Module");
        assert_eq!(EcuType::Abs.name(), "ABS Control Module");
    }

    #[test]
    fn test_ecu_family_creation() {
        let family = EcuFamily::new("EDC17", EcuManufacturer::Bosch, EcuType::Engine)
            .with_description("Electronic Diesel Control 17")
            .with_processor("TC1796")
            .with_protocol("UDS")
            .with_years(2008, 2026)
            .with_diag_ids(0x7E0, 0x7E8);

        assert_eq!(family.name, "EDC17");
        assert_eq!(family.manufacturer, EcuManufacturer::Bosch);
        assert_eq!(family.processor, Some("TC1796".to_string()));
        assert_eq!(family.year_range, Some((2008, 2026)));
        assert_eq!(family.diag_can_id_request, Some(0x7E0));
    }

    #[test]
    fn test_ecu_instance_creation() {
        let ecu = EcuInstance::new("ECU_001", 0x7E0, 0x7E8)
            .with_manufacturer(EcuManufacturer::Bosch)
            .with_type(EcuType::Engine)
            .with_part_number("0281012345")
            .with_a2l("ecu.a2l");

        assert_eq!(ecu.id, "ECU_001");
        assert_eq!(ecu.manufacturer, EcuManufacturer::Bosch);
        assert_eq!(ecu.part_number, Some("0281012345".to_string()));
        assert_eq!(ecu.a2l_file, Some("ecu.a2l".to_string()));
    }

    #[test]
    fn test_load_builtin_database() {
        let db = EcuDatabase::load_builtin();

        // Check Bosch families loaded
        assert!(db.find_family("EDC17").is_some());
        assert!(db.find_family("MED17").is_some());
        assert!(db.find_family("ME7").is_some());

        // Check Continental families loaded
        assert!(db.find_family("SIMOS").is_some());
        assert!(db.find_family("SID").is_some());

        // Check Denso families loaded
        assert!(db.find_family("DENSO_TOYOTA").is_some());

        // Check Delphi families loaded
        assert!(db.find_family("DCM").is_some());

        // Check Magneti Marelli families loaded
        assert!(db.find_family("IAW").is_some());
        assert!(db.find_family("MJD").is_some());
    }

    #[test]
    fn test_families_by_manufacturer() {
        let db = EcuDatabase::load_builtin();

        let bosch_families = db.families_by_manufacturer(EcuManufacturer::Bosch);
        assert!(!bosch_families.is_empty());
        assert!(bosch_families.iter().any(|f| f.name == "EDC17"));

        let continental_families = db.families_by_manufacturer(EcuManufacturer::Continental);
        assert!(!continental_families.is_empty());
        assert!(continental_families.iter().any(|f| f.name == "SIMOS"));
    }

    #[test]
    fn test_families_by_type() {
        let db = EcuDatabase::load_builtin();

        let engine_families = db.families_by_type(EcuType::Engine);
        assert!(!engine_families.is_empty());

        let trans_families = db.families_by_type(EcuType::Transmission);
        assert!(!trans_families.is_empty());
    }

    #[test]
    fn test_uds_did_constants() {
        assert_eq!(UdsDid::VIN, 0xF190);
        assert_eq!(UdsDid::SW_VERSION, 0xF189);
        assert_eq!(UdsDid::HW_VERSION, 0xF193);

        let dids = UdsDid::identification_dids();
        assert!(dids.contains(&UdsDid::VIN));
        assert!(dids.contains(&UdsDid::SW_VERSION));
    }

    #[test]
    fn test_scan_config_default() {
        let config = EcuScanConfig::default();
        assert_eq!(config.start_id, 0x7E0);
        assert_eq!(config.end_id, 0x7E7);
        assert!(config.use_functional_addressing);
        assert_eq!(config.timeout_ms, 1000);
    }

    #[test]
    fn test_scan_config_builder() {
        let config = EcuScanConfig::new()
            .with_id_range(0x700, 0x7FF)
            .with_timeout(2000)
            .with_extended_range(0x600, 0x6FF);

        assert_eq!(config.start_id, 0x700);
        assert_eq!(config.end_id, 0x7FF);
        assert_eq!(config.timeout_ms, 2000);
        assert!(config.scan_extended_range);
    }

    #[test]
    fn test_ecu_identifier() {
        let identifier = EcuIdentifier::new();

        // Create a detection result with Bosch part number
        let mut ecu = EcuInstance::new("test", 0x7E0, 0x7E8);
        ecu.part_number = Some("0281012345".to_string());
        ecu.sw_version = Some("EDC17C46".to_string());

        let result = EcuDetectionResult::new(ecu, DetectionMethod::UdsReadDid);

        let identification = identifier.identify(&result);
        assert!(identification.is_some());

        let id = identification.unwrap();
        assert_eq!(id.manufacturer, Some(EcuManufacturer::Bosch));
        assert_eq!(id.family, Some("EDC17".to_string()));
    }

    #[test]
    fn test_vin_decoder() {
        // Test VIN: WVWZZZ3CZWE123456
        // Position 10 (index 9) is 'W' = 2028
        let vin_info = VinDecoder::decode("WVWZZZ3CZWE123456");
        assert!(vin_info.is_some());

        let info = vin_info.unwrap();
        assert_eq!(info.wmi, "WVW");
        assert_eq!(info.model_year, Some(2028)); // 'W' = 2028
        assert!(info.manufacturer.is_some());
    }

    #[test]
    fn test_vin_decoder_invalid() {
        // Too short
        assert!(VinDecoder::decode("ABC123").is_none());
        // Too long
        assert!(VinDecoder::decode("WVWZZZ3CZWE1234567890").is_none());
    }

    #[test]
    fn test_a2l_association_manager() {
        let mut manager = A2lAssociationManager::new();

        manager.associate_ecu("ECU_001", "/path/to/ecu1.a2l");
        manager.associate_family("EDC17", "/path/to/edc17.a2l");
        manager.associate_part_number("0281", "/path/to/bosch.a2l");

        // Test direct ECU association
        let ecu1 = EcuInstance::new("ECU_001", 0x7E0, 0x7E8);
        assert_eq!(manager.find_a2l(&ecu1), Some(&"/path/to/ecu1.a2l".to_string()));

        // Test family association
        let mut ecu2 = EcuInstance::new("ECU_002", 0x7E0, 0x7E8);
        ecu2.family = Some("EDC17".to_string());
        assert_eq!(manager.find_a2l(&ecu2), Some(&"/path/to/edc17.a2l".to_string()));

        // Test part number association
        let mut ecu3 = EcuInstance::new("ECU_003", 0x7E0, 0x7E8);
        ecu3.part_number = Some("0281012345".to_string());
        assert_eq!(manager.find_a2l(&ecu3), Some(&"/path/to/bosch.a2l".to_string()));
    }

    #[test]
    fn test_detection_result() {
        let ecu = EcuInstance::new("test", 0x7E0, 0x7E8);
        let mut result = EcuDetectionResult::new(ecu, DetectionMethod::UdsReadDid);

        // Add VIN data (first byte is length in UDS response)
        let vin_data = b"\x11WVWZZZ3CZWE123456".to_vec();
        result.add_raw_data(UdsDid::VIN, vin_data);

        let vin = result.get_vin();
        assert!(vin.is_some());
        assert_eq!(vin.unwrap(), "WVWZZZ3CZWE123456");
    }
}
