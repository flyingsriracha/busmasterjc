//! BUSMASTER Database File Parsers
//!
//! This crate provides parsers for various automotive database file formats:
//!
//! - **DBC** - CAN database format (Vector)
//! - **LDF** - LIN Description File
//! - **DBF** - CAN database format (BUSMASTER) (future)
//! - **ARXML** - AUTOSAR XML (future)
//!
//! # Example
//!
//! ```
//! use busmaster_db::dbc::DbcParser;
//!
//! let dbc_content = r#"
//! VERSION ""
//!
//! NS_ :
//!     NS_DESC_
//!     CM_
//!     BA_DEF_
//!     BA_
//!     VAL_
//!     CAT_DEF_
//!     CAT_
//!     FILTER
//!     BA_DEF_DEF_
//!     EV_DATA_
//!     ENVVAR_DATA_
//!     SGTYPE_
//!     SGTYPE_VAL_
//!     BA_DEF_SGTYPE_
//!     BA_SGTYPE_
//!     SIG_TYPE_REF_
//!     VAL_TABLE_
//!     SIG_GROUP_
//!     SIG_VALTYPE_
//!     SIGTYPE_VALTYPE_
//!     BO_TX_BU_
//!     BA_DEF_REL_
//!     BA_REL_
//!     BA_SGTYPE_REL_
//!     SG_MUL_VAL_
//!
//! BS_:
//!
//! BU_: Node1 Node2
//!
//! BO_ 100 TestMessage: 8 Node1
//!  SG_ TestSignal : 0|8@1+ (1,0) [0|255] "" Node2
//! "#;
//!
//! let database = DbcParser::parse(dbc_content).unwrap();
//! assert_eq!(database.messages.len(), 1);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::type_complexity)]
#![allow(clippy::missing_errors_doc)]

pub mod a2l;
pub mod arxml;
pub mod dbc;
pub mod dbf;
pub mod ecu_database;
pub mod ldf;
pub mod odx;

pub use a2l::{
    A2lAnnotation, A2lAxisDescr, A2lAxisPts, A2lAxisPtsLayout, A2lAxisRescale, A2lByteOrder,
    A2lCharacteristic, A2lCompuMethod, A2lCompuTab, A2lCompuVtab, A2lCompuVtabRange, A2lDatabase,
    A2lDataType, A2lFncValues, A2lFunction, A2lGenerator, A2lGroup, A2lHeader, A2lIfData,
    A2lMeasurement, A2lMemorySegment, A2lModCommon, A2lModPar, A2lModule, A2lParser, A2lProject,
    A2lRecordLayout, A2lUnit, Asap2Version, AxisType, CharacteristicType, CompuMethodType,
    DepositMode,
};
pub use dbc::{DbcDatabase, DbcMessage, DbcParser, DbcSignal};
pub use ldf::{
    LdfDatabase, LdfFrame, LdfFrameSignal, LdfMasterNode, LdfNodeAttributes, LdfParser,
    LdfScheduleEntry, LdfScheduleEntryType, LdfScheduleTable, LdfSignal, LdfSlaveNode,
};

pub use ecu_database::{
    A2lAssociationManager, AiSuggestion, DetectionMethod, EcuDatabase, EcuDetectionResult,
    EcuFamily, EcuIdentifier, EcuInstance, EcuManufacturer, EcuScanConfig, EcuType,
    IdentificationResult, UdsDid, VinDecoder, VinInfo,
};

pub use odx::{
    Audience, BaseDataType, ByteOrder, Comparam, ComparamClass, ComparamSpec, CompuCategory,
    CompuMethod, CompuRationalCoeffs, CompuScale, DataDictionaryEntry, DataObjectProperty,
    DiagCodedType, DiagLayer, DiagLayerType, DiagService, Dtc, DtcSeverity, EcuSharedData,
    FunctionalClass, FunctionalGroup, LogicalLink, OdxDatabase, OdxParseError, OdxParser, Param,
    ParamSemantic, ParamType, PhysicalType, Request, Response, SingleEcuJob, VehicleInfo,
};

pub use dbf::{
    DbfByteOrder, DbfComment, DbfDatabase, DbfFrameFormat, DbfGenerator, DbfMessage,
    DbfParamType, DbfParameter, DbfParseError, DbfParser, DbfProtocol, DbfSignal, DbfSignalType,
    DbfValueDescriptor, DbfValueTable,
};

pub use arxml::{
    ArPackage, ArxmlByteOrder, ArxmlDatabase, ArxmlParseError, ArxmlParser, AutosarVersion,
    BaseTypeCategory, BaseTypeEncoding, CanCluster, CanFrame, CanFrameTriggering,
    CanPhysicalChannel, CompuMethodCategory, CompuScale as ArxmlCompuScale, EcuInstance as ArxmlEcuInstance,
    FrameTriggeringType, ISignal, ISignalIPdu, ISignalToPduMapping, PduToFrameMapping,
    SwBaseType, Unit,
};
