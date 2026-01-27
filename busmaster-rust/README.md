# BUSMASTER Rust

[![CI](https://github.com/rbei-etas/busmaster/actions/workflows/ci.yml/badge.svg)](https://github.com/rbei-etas/busmaster/actions/workflows/ci.yml)
[![License: LGPL-3.0](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)

A modern Rust implementation of BUSMASTER - an open-source automotive bus monitoring and simulation tool.

## Overview

BUSMASTER Rust is a complete rewrite of the original C++ BUSMASTER application, designed for:

- **Memory Safety**: Leveraging Rust's ownership system
- **Cross-Platform**: macOS (MVP), Windows, Linux
- **Cloud-Native**: REST API, Docker, serverless support
- **Modern Protocols**: CAN, CAN FD, CAN XL, Automotive Ethernet

## Features (MVP)

- ✅ CAN 2.0A/B protocol support
- ✅ Standard and Extended frame IDs
- ✅ DBC database parsing
- ✅ ASC log file format
- ✅ Message filtering
- ✅ CLI interface
- ✅ TUI interface (ratatui)
- ✅ Stub driver (software simulation)
- 🔄 PEAK USB driver (macOS)

## Quick Start

### Prerequisites

- Rust 1.75 or later
- macOS 12+ (MVP target)

### Installation

```bash
# Clone the repository
git clone https://github.com/rbei-etas/busmaster.git
cd busmaster/source/busmaster-rust

# Build
cargo build --release

# Run CLI
./target/release/busmaster --help
```

### Basic Usage

```bash
# List available drivers
busmaster list

# Monitor CAN bus with stub driver
busmaster monitor --driver stub --channel 0

# Monitor with DBC file for signal decoding
busmaster monitor --driver stub --dbc path/to/database.dbc

# Log messages to ASC file
busmaster monitor --driver stub --log output.asc

# Send a CAN message
busmaster send --driver stub --id 0x123 --data "01 02 03 04"
```

## Project Structure

```
busmaster-rust/
├── crates/
│   ├── busmaster-core/      # Core types (CanFrame, SignalDef, errors)
│   ├── busmaster-proto/     # Protocol implementations
│   ├── busmaster-dil/       # Driver Interface Layer (traits)
│   ├── busmaster-hardware/  # Hardware drivers
│   ├── busmaster-db/        # Database parsers (DBC, etc.)
│   ├── busmaster-log/       # Log formats (ASC, BLF, etc.)
│   ├── busmaster-engine/    # Main orchestrator
│   ├── busmaster-cli/       # CLI application
│   ├── busmaster-tui/       # TUI application
│   └── busmaster-platform/  # Platform abstraction
├── docs/                    # Development documentation
├── tests/                   # Integration tests
├── benches/                 # Benchmarks
└── examples/                # Usage examples
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

### Running Tests

```bash
# All tests
cargo test --all-features --workspace

# Specific crate
cargo test -p busmaster-core

# With coverage
cargo llvm-cov --all-features --workspace
```

### Documentation

```bash
# Generate docs
cargo doc --no-deps --all-features --workspace --open
```

## Roadmap

| Phase | Timeline | Features |
|-------|----------|----------|
| MVP | Months 1-6 | CAN, CLI/TUI, DBC, ASC, PEAK driver |
| Phase 2 | Months 7-12 | CAN FD, J1939, DoIP, SOME/IP, UDS |
| Phase 3 | Months 13-18 | REST API, Cloud, AI integration |
| Phase 4 | Months 19-30 | Windows, Linux, GUI |
| Phase 5 | Months 31-36 | FlexRay, full CANoe parity |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the LGPL-3.0 License - see the [LICENSE](../../COPYING.LESSER.txt) file for details.

## Acknowledgments

- Original BUSMASTER C++ project
- Vector CANoe for feature inspiration
- Rust community for excellent tooling

## Links

- [Original BUSMASTER](https://rbei-etas.github.io/busmaster/)
- [Documentation](docs/README.md)
- [Issue Tracker](https://github.com/rbei-etas/busmaster/issues)
