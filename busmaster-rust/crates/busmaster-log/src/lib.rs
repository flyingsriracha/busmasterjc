//! # busmaster-log
//!
//! Log file format implementations for BUSMASTER.
//!
//! This crate provides writers and readers for various automotive log file formats:
//! - ASC (ASCII) - Vector CANoe/CANalyzer text format (MVP)
//! - BLF (Binary Logging Format) - Vector binary format (Phase 2)
//! - PCAP - Wireshark packet capture format (Phase 2)
//! - MDF4 - ASAM Measurement Data Format (Phase 5)

pub mod asc;
pub mod blf;
pub mod pcap;

pub use asc::AscWriter;
pub use blf::{
    BlfCanMessage, BlfFileHeader, BlfObject, BlfObjectHeader, BlfObjectType, BlfReader, BlfWriter,
    CanMessageFlags, BLF_HEADER_SIZE, BLF_OBJECT_HEADER_SIZE, BLF_OBJECT_SIGNATURE, BLF_SIGNATURE,
};
pub use pcap::{
    LinkType, PcapGlobalHeader, PcapPacket, PcapPacketHeader, PcapReader, PcapWriter,
    SocketCanFrame, PCAP_GLOBAL_HEADER_SIZE, PCAP_MAGIC, PCAP_MAGIC_NANO, PCAP_PACKET_HEADER_SIZE,
    SOCKETCAN_FRAME_SIZE,
};
