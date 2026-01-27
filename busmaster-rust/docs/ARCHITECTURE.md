# BUSMASTER Rust Architecture

## Overview

The BUSMASTER Rust implementation follows a layered architecture designed for modularity, testability, and cross-platform support.

## Layer Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Application Layer                                   │
│  busmaster-cli (MVP) → busmaster-tui → busmaster-gui → busmaster-web    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      Business Logic Layer                                │
│  busmaster-engine: orchestration, filtering, logging                    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────┬───────────────┬───────────────┬─────────────────────────┐
│ busmaster-db  │busmaster-proto│ busmaster-log │ (future: busmaster-diag)│
│ DBC/DBF/ARXML │ CAN/LIN/J1939 │ ASC/BLF/MDF4  │ UDS/OBD-II/KWP          │
└───────────────┴───────────────┴───────────────┴─────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      Hardware Abstraction Layer                          │
│  busmaster-dil: traits, busmaster-hardware: implementations             │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      Platform Layer                                      │
│  busmaster-platform: OS-specific USB, timing, networking                │
└─────────────────────────────────────────────────────────────────────────┘
```

## Crate Responsibilities

### Core Layer
- **busmaster-core**: Fundamental types (CanFrame, SignalDef, errors)
  - `#![forbid(unsafe_code)]` - No unsafe code allowed
  - Serde serialization for all types
  - Property-based tests for correctness

### Protocol Layer
- **busmaster-proto**: Protocol parsing/encoding
  - CAN 2.0, CAN FD frame handling
  - Future: LIN, FlexRay, J1939, Ethernet protocols

### Database Layer
- **busmaster-db**: Database file parsers
  - DBC (CAN database) - MVP
  - Future: DBF, LDF, ARXML, ODX, A2L

### Logging Layer
- **busmaster-log**: Log file formats
  - ASC (ASCII) - MVP
  - Future: BLF, MDF4, PCAP

### Hardware Layer
- **busmaster-dil**: Driver Interface Layer (traits only)
  - `#![forbid(unsafe_code)]` - Safe abstractions
  - CanDriver trait, ChannelConfig, DeviceInfo
- **busmaster-hardware**: Driver implementations
  - StubDriver (MVP) - Software simulation
  - Future: PEAK, Vector, Kvaser drivers

### Platform Layer
- **busmaster-platform**: OS-specific code
  - High-precision timestamps
  - USB device enumeration
  - Platform detection

### Application Layer
- **busmaster-engine**: Main orchestrator
- **busmaster-cli**: Command-line interface
- **busmaster-tui**: Terminal UI (ratatui)

## Design Principles

1. **MVP First**: Build working software before optimizing
2. **Modular Design**: Independent crates with clear boundaries
3. **Type Safety**: Leverage Rust's type system for correctness
4. **Explicit over Implicit**: Clear code that AI can understand
5. **Test-Driven**: Tests define behavior, implementation follows
6. **Platform Abstraction**: Isolate platform-specific code early

## Dependency Flow

```
busmaster-core (no deps)
    ↑
busmaster-proto, busmaster-dil, busmaster-db, busmaster-log
    ↑
busmaster-hardware (depends on dil)
    ↑
busmaster-engine (depends on all above)
    ↑
busmaster-cli, busmaster-tui (depends on engine)
```

## Quality Requirements

From requirements.md section 10.3:
- `cargo clippy` passes with no warnings
- `cargo fmt` applied to all code
- `cargo test` passes with >80% coverage
- `cargo audit` shows no vulnerabilities
- All public APIs documented with rustdoc
