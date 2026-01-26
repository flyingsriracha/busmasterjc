# BUSMASTER Rust Conversion - Requirements Specification

**Project Name:** BUSMASTER Rust Conversion (macOS First)  
**Project Owner:** JC  
**Version:** 4.0  
**Date:** January 2026  
**Status:** APPROVED FOR AI DEVELOPMENT  
**Development Model:** AI-ONLY (No human coders)  
**Target Platform:** macOS (MVP) → Windows → Linux → Cloud-native  
**Competitor Benchmark:** Vector CANoe, CANalyzer, CANape, vTESTstudio, Vehicle Spy, ETAS INCA, SavvyCAN

---

## 1. Executive Summary

This document outlines the requirements for converting the BUSMASTER automotive bus monitoring and simulation tool from C++ to Rust. This is an **AI-only development project** - all code will be written by AI agents with iterative refinement.

**Key Objectives:**
- Deliver a working MVP on macOS within 6 months
- Achieve feature parity with Vector CANoe incrementally over 36 months
- Support Automotive Ethernet (DoIP, SOME/IP, XCP over Ethernet)
- Enable cloud-native deployment (headless, serverless, REST API)
- Integrate AI assistance (OpenAI/Azure OpenAI) for analysis
- Improve memory safety and concurrency through Rust's type system
- Enable cross-platform support (macOS → Windows → Linux → Cloud)

**AI Development Philosophy:**
- Rapid prototyping with iterative refinement
- Fail fast, fix fast - AI can regenerate code quickly
- Prioritize working code over perfect architecture
- Test-driven development with property-based testing
- Continuous integration with automated quality gates

---

## 2. CANoe Feature Comparison

### 2.1 Protocol Support Comparison

| Feature | Vector CANoe | BUSMASTER C++ | Our Rust Target |
|---------|--------------|---------------|-----------------|
| CAN 2.0A/B | ✓ | ✓ | ✓ MVP |
| CAN FD | ✓ | Partial | ✓ Phase 2 |
| **CAN XL** | ✓ | ✗ | ✓ Phase 4 (NEW) |
| LIN | ✓ | ✓ | ✓ Phase 3 |
| FlexRay | ✓ | Partial | ✓ Phase 5 |
| J1939 | ✓ | ✓ | ✓ Phase 2 |
| **Automotive Ethernet** | ✓ | ✗ | ✓ Phase 2 |
| DoIP | ✓ | ✗ | ✓ Phase 2 |
| SOME/IP | ✓ | ✗ | ✓ Phase 2 |
| XCP/CCP | ✓ | ✗ | ✓ Phase 3 |
| UDS | ✓ | ✓ | ✓ Phase 2 |
| OBD-II | ✓ | Partial | ✓ Phase 2 |
| KWP2000 | ✓ | ✗ | ✓ Phase 3 |
| **ISO 15118** | ✓ | ✗ | ✓ Phase 4 (NEW - EV Charging) |
| **SecOC** | ✓ | ✗ | ✓ Phase 4 (NEW - Security) |

### 2.2 Analysis & Simulation Comparison

| Feature | Vector CANoe | BUSMASTER C++ | Our Rust Target |
|---------|--------------|---------------|-----------------|
| Message Trace | ✓ | ✓ | ✓ MVP |
| Signal Watch | ✓ | ✓ | ✓ MVP |
| Signal Graphing | ✓ | ✓ | ✓ Phase 3 |
| Bus Statistics | ✓ | ✓ | ✓ Phase 2 |
| **Bus Load Analysis** | ✓ | Partial | ✓ Phase 2 (NEW) |
| ECU Simulation | ✓ (CAPL) | ✓ (C++) | ✓ Phase 4 (Rust/Lua) |
| Node Simulation | ✓ | ✓ | ✓ Phase 4 |
| **Gateway Simulation** | ✓ | ✗ | ✓ Phase 4 (NEW) |
| Replay | ✓ | ✓ | ✓ Phase 2 |
| **Trace Playback** | ✓ | Partial | ✓ Phase 2 (NEW) |
| **REST API** | ✓ | ✗ | ✓ Phase 3 |
| **Cloud/Headless** | Partial | ✗ | ✓ Phase 3 |
| **AI Analysis** | ✗ | ✗ | ✓ Phase 3 |
| **Wireshark Export** | ✓ | ✗ | ✓ Phase 2 (NEW) |
| **Reverse Engineering** | Partial | ✗ | ✓ Phase 3 (NEW - Signal Discovery) |
| **Test Automation** | ✓ (vTESTstudio) | Partial | ✓ Phase 4 (NEW) |
| **Measurement/Calibration** | ✓ (CANape) | ✗ | ✓ Phase 4 (NEW) |

### 2.3 Database Format Comparison

| Format | Vector CANoe | BUSMASTER C++ | Our Rust Target |
|--------|--------------|---------------|-----------------|
| DBC (CAN) | ✓ R/W | ✓ R/W | ✓ MVP Read, Phase 2 R/W |
| DBF (LIN) | ✓ R/W | ✓ R/W | ✓ Phase 3 |
| LDF (LIN) | ✓ R/W | ✓ R/W | ✓ Phase 3 |
| ARXML (AUTOSAR) | ✓ R/W | Partial | ✓ Phase 3 |
| ODX (Diagnostics) | ✓ R/W | ✗ | ✓ Phase 3 |
| FIBEX (FlexRay) | ✓ R/W | ✗ | ✓ Phase 5 |
| A2L (XCP) | ✓ R/W | ✗ | ✓ Phase 3 |
| CDD (CANdela) | ✓ R/W | ✗ | ✓ Phase 4 |

### 2.4 Logging Format Comparison

| Format | Vector CANoe | BUSMASTER C++ | Our Rust Target |
|--------|--------------|---------------|-----------------|
| ASC (ASCII) | ✓ R/W | ✓ R/W | ✓ MVP |
| BLF (Binary) | ✓ R/W | ✓ R/W | ✓ Phase 2 |
| MDF4 | ✓ R/W | ✗ | ✓ Phase 3 |
| **MDF4.3** | ✓ R/W | ✗ | ✓ Phase 4 (NEW - Latest ASAM) |
| CSV | ✓ | ✓ | ✓ MVP |
| PCAP (Ethernet) | ✓ | ✗ | ✓ Phase 2 |
| **PCAPNG** | ✓ | ✗ | ✓ Phase 3 (NEW - Extended PCAP) |

### 2.5 Hardware Support Comparison

| Vendor | Vector CANoe | BUSMASTER C++ | Our Rust Target |
|--------|--------------|---------------|-----------------|
| Vector XL | ✓ | ✓ | ✓ Phase 2 |
| PEAK USB | ✓ | ✓ | ✓ MVP |
| Kvaser | ✓ | ✓ | ✓ Phase 3 |
| ETAS BOA | ✓ | ✓ | ✓ Phase 3 |
| Intrepid neoVI | ✓ | ✓ | ✓ Phase 4 |
| IXXAT VCI | ✓ | ✓ | ✓ Phase 4 |
| SocketCAN (Linux) | ✓ | ✗ | ✓ Phase 4 |
| Virtual/Stub | ✓ | ✓ | ✓ MVP |

---

## 3. Minimum Viable Product (MVP) Definition

### 3.1 MVP Scope (Phase 1 - Months 1-6)

The MVP delivers a **functional CAN bus monitor** on macOS with:

#### 3.1.1 Must Have (MVP Core)
- [ ] **CAN Protocol Only** - Standard and Extended IDs
- [ ] **Stub Driver** - Software simulation (no hardware required)
- [ ] **CLI Interface** - Command-line message monitoring
- [ ] **DBC File Parsing** - Basic signal extraction
- [ ] **ASC Logging** - Simple text-based logging
- [ ] **Message Filtering** - By ID range

#### 3.1.2 Should Have (MVP Extended)
- [ ] **PEAK USB Support** - One hardware vendor
- [ ] **Signal Watch** - Real-time signal values
- [ ] **Basic TUI** - Terminal UI with `ratatui`
- [ ] **CSV Export** - Basic data export

#### 3.1.3 Won't Have (Post-MVP)
- LIN, FlexRay, J1939 protocols
- Automotive Ethernet (DoIP, SOME/IP)
- Full GUI application
- Multiple hardware vendors
- Plugin system
- BLF logging format
- Cloud deployment
- AI integration

### 3.2 MVP Success Criteria

```
✓ Can connect to stub driver or PEAK USB
✓ Can receive and display CAN messages in real-time
✓ Can parse DBC file and show signal values
✓ Can filter messages by ID
✓ Can log messages to ASC file
✓ Latency < 5ms for message display
✓ No crashes during 1-hour continuous operation
✓ All tests passing (>80% coverage)
```

---

## 4. Automotive Ethernet Requirements (NEW)

### 4.1 DoIP (Diagnostics over IP) - ISO 13400

**REQ-DOIP-001:** Support DoIP vehicle identification
**REQ-DOIP-002:** Support DoIP routing activation
**REQ-DOIP-003:** Support diagnostic message transmission over TCP
**REQ-DOIP-004:** Support DoIP entity status request
**REQ-DOIP-005:** Support DoIP alive check
**REQ-DOIP-006:** Support DoIP power mode information

### 4.2 SOME/IP (Scalable service-Oriented MiddlewarE over IP)

**REQ-SOMEIP-001:** Support SOME/IP service discovery (SD)
**REQ-SOMEIP-002:** Support SOME/IP request/response pattern
**REQ-SOMEIP-003:** Support SOME/IP fire-and-forget pattern
**REQ-SOMEIP-004:** Support SOME/IP publish/subscribe (events)
**REQ-SOMEIP-005:** Support SOME/IP-TP for large payloads
**REQ-SOMEIP-006:** Support SOME/IP serialization

### 4.3 XCP over Ethernet

**REQ-XCP-ETH-001:** Support XCP on Ethernet transport layer
**REQ-XCP-ETH-002:** Support XCP measurement and calibration
**REQ-XCP-ETH-003:** Support A2L file parsing for XCP
**REQ-XCP-ETH-004:** Support XCP DAQ lists
**REQ-XCP-ETH-005:** Support XCP STIM for stimulation

### 4.4 Ethernet Infrastructure

**REQ-ETH-001:** Support raw Ethernet frame capture
**REQ-ETH-002:** Support VLAN tagging (802.1Q)
**REQ-ETH-003:** Support PTP (Precision Time Protocol) timestamps
**REQ-ETH-004:** Support AVB (Audio Video Bridging) analysis
**REQ-ETH-005:** Support PCAP file format for logging

---

## 5. Diagnostic Protocol Requirements (NEW)

### 5.1 UDS (Unified Diagnostic Services) - ISO 14229

**REQ-UDS-001:** Support all UDS service identifiers (SIDs)
**REQ-UDS-002:** Support diagnostic session control
**REQ-UDS-003:** Support security access (seed-key)
**REQ-UDS-004:** Support read/write data by identifier
**REQ-UDS-005:** Support routine control
**REQ-UDS-006:** Support DTC (Diagnostic Trouble Code) management
**REQ-UDS-007:** Support ECU reset
**REQ-UDS-008:** Support transfer data (flash programming)

### 5.2 OBD-II (On-Board Diagnostics)

**REQ-OBD-001:** Support OBD-II PIDs (Parameter IDs)
**REQ-OBD-002:** Support OBD-II modes 01-0A
**REQ-OBD-003:** Support DTC reading and clearing
**REQ-OBD-004:** Support freeze frame data
**REQ-OBD-005:** Support vehicle information (VIN)

### 5.3 KWP2000 (Keyword Protocol 2000)

**REQ-KWP-001:** Support KWP2000 over CAN
**REQ-KWP-002:** Support KWP2000 diagnostic services
**REQ-KWP-003:** Support KWP2000 session management

---

## 6. Cloud-Native Requirements (NEW)

### 6.1 Headless Operation

**REQ-CLOUD-001:** Support headless mode without GUI
**REQ-CLOUD-002:** Support configuration via TOML/YAML files
**REQ-CLOUD-003:** Support command-line automation
**REQ-CLOUD-004:** Support daemon mode for continuous operation
**REQ-CLOUD-005:** Support Docker containerization

### 6.2 REST API

**REQ-API-001:** Provide REST API for remote control
**REQ-API-002:** Support WebSocket for real-time data streaming
**REQ-API-003:** Support OpenAPI/Swagger documentation
**REQ-API-004:** Support authentication (API keys, OAuth2)
**REQ-API-005:** Support rate limiting and quotas

### 6.3 Serverless Deployment

**REQ-SERVERLESS-001:** Support AWS Lambda deployment
**REQ-SERVERLESS-002:** Support Azure Functions deployment
**REQ-SERVERLESS-003:** Support log file processing as serverless function
**REQ-SERVERLESS-004:** Support database parsing as serverless function

---

## 7. AI Integration Requirements (NEW)

### 7.1 OpenAI/Azure OpenAI Integration

**REQ-AI-001:** Support OpenAI API for chat-based analysis
**REQ-AI-002:** Support Azure OpenAI for enterprise deployments
**REQ-AI-003:** Support configurable API endpoints and keys
**REQ-AI-004:** Support streaming responses for real-time interaction
**REQ-AI-005:** Support context management for conversation history

### 7.2 AI-Assisted Analysis Features

**REQ-AI-ANALYSIS-001:** AI-powered anomaly detection in bus traffic
**REQ-AI-ANALYSIS-002:** Natural language queries for log analysis
**REQ-AI-ANALYSIS-003:** Automatic DBC signal interpretation suggestions
**REQ-AI-ANALYSIS-004:** AI-generated test case recommendations
**REQ-AI-ANALYSIS-005:** Intelligent error diagnosis and suggestions
**REQ-AI-ANALYSIS-006:** Pattern recognition in message sequences

### 7.3 AI Configuration

**REQ-AI-CONFIG-001:** Support model selection (GPT-4, GPT-3.5, etc.)
**REQ-AI-CONFIG-002:** Support temperature and token limit configuration
**REQ-AI-CONFIG-003:** Support custom system prompts for domain expertise
**REQ-AI-CONFIG-004:** Support offline mode with local models (optional)

---

## 8. CAN XL Protocol Requirements (NEW)

### 8.1 CAN XL Core Features

**REQ-CANXL-001:** Support CAN XL frame format (up to 2048 bytes payload)
**REQ-CANXL-002:** Support CAN XL data rates (up to 10 Mbps)
**REQ-CANXL-003:** Support CAN XL priority field
**REQ-CANXL-004:** Support CAN XL acceptance field
**REQ-CANXL-005:** Support CAN XL SDT (Service Data Unit Type)
**REQ-CANXL-006:** Support CAN XL SEC (Simple Extended Content)
**REQ-CANXL-007:** Backward compatibility with CAN FD and CAN 2.0

### 8.2 CAN XL Use Cases

**REQ-CANXL-UC-001:** Bridge CAN networks to Ethernet backbone
**REQ-CANXL-UC-002:** Support software-defined vehicle (SDV) architectures
**REQ-CANXL-UC-003:** Support ADAS high-bandwidth data transfer

---

## 9. ISO 15118 EV Charging Requirements (NEW)

### 9.1 ISO 15118-2 (AC/DC Charging)

**REQ-ISO15118-001:** Support vehicle identification
**REQ-ISO15118-002:** Support Plug & Charge (PnC) authentication
**REQ-ISO15118-003:** Support smart charging (load balancing)
**REQ-ISO15118-004:** Support bidirectional power transfer (V2G)
**REQ-ISO15118-005:** Support charging session management
**REQ-ISO15118-006:** Support TLS-based secure communication

### 9.2 ISO 15118-20 (Extended Features)

**REQ-ISO15118-20-001:** Support wireless charging communication
**REQ-ISO15118-20-002:** Support automatic connection device (ACD)
**REQ-ISO15118-20-003:** Support bidirectional energy flow for V2G
**REQ-ISO15118-20-004:** Support DC BPT (Bidirectional Power Transfer)

---

## 10. SecOC Security Requirements (NEW)

### 10.1 AUTOSAR SecOC Implementation

**REQ-SECOC-001:** Support message authentication code (MAC) generation
**REQ-SECOC-002:** Support freshness value management
**REQ-SECOC-003:** Support AES-128-CMAC algorithm
**REQ-SECOC-004:** Support key management interface
**REQ-SECOC-005:** Support SecOC PDU structure
**REQ-SECOC-006:** Support truncated MAC for bandwidth efficiency

### 10.2 SecOC Analysis Features

**REQ-SECOC-ANALYSIS-001:** Detect SecOC-protected messages
**REQ-SECOC-ANALYSIS-002:** Display MAC and freshness values
**REQ-SECOC-ANALYSIS-003:** Support key import for decryption
**REQ-SECOC-ANALYSIS-004:** Validate message authenticity

---

## 11. Test Automation Requirements (NEW - vTESTstudio-like)

### 11.1 Test Design

**REQ-TEST-001:** Support graphical test sequence design
**REQ-TEST-002:** Support table-based test case definition
**REQ-TEST-003:** Support programming-based test scripts (Rust/Lua)
**REQ-TEST-004:** Support test case parameterization
**REQ-TEST-005:** Support test case reusability

### 11.2 Test Execution

**REQ-TEST-EXEC-001:** Support automated test execution
**REQ-TEST-EXEC-002:** Support headless test runner (CI/CD integration)
**REQ-TEST-EXEC-003:** Support test scheduling
**REQ-TEST-EXEC-004:** Support parallel test execution
**REQ-TEST-EXEC-005:** Generate test reports (HTML, XML, JSON)

### 11.3 Test Traceability

**REQ-TEST-TRACE-001:** Link test cases to requirements
**REQ-TEST-TRACE-002:** Support test coverage reporting
**REQ-TEST-TRACE-003:** Support test result history

---

## 12. Measurement & Calibration Requirements (NEW - CANape-like)

### 12.1 XCP Measurement

**REQ-MCA-001:** Support XCP DAQ (Data Acquisition) lists
**REQ-MCA-002:** Support configurable sampling rates
**REQ-MCA-003:** Support event-based measurement
**REQ-MCA-004:** Support time-based measurement
**REQ-MCA-005:** Support multi-channel measurement

### 12.2 XCP Calibration

**REQ-MCA-CAL-001:** Support XCP STIM (Stimulation)
**REQ-MCA-CAL-002:** Support parameter modification
**REQ-MCA-CAL-003:** Support calibration page switching
**REQ-MCA-CAL-004:** Support flash programming via XCP

### 12.3 A2L Support

**REQ-MCA-A2L-001:** Parse A2L files (ASAM MCD-2 MC)
**REQ-MCA-A2L-002:** Display measurement/calibration objects
**REQ-MCA-A2L-003:** Support A2L editing (basic)
**REQ-MCA-A2L-004:** Support A2L generation from ELF/DWARF

---

## 13. Gateway & Routing Requirements (NEW)

### 13.1 Protocol Translation

**REQ-GW-001:** Support CAN-to-Ethernet translation
**REQ-GW-002:** Support CAN-to-CAN FD translation
**REQ-GW-003:** Support CAN-to-LIN translation
**REQ-GW-004:** Support message routing rules
**REQ-GW-005:** Support message filtering at gateway

### 13.2 Gateway Simulation

**REQ-GW-SIM-001:** Simulate gateway ECU behavior
**REQ-GW-SIM-002:** Support routing table configuration
**REQ-GW-SIM-003:** Support message transformation rules
**REQ-GW-SIM-004:** Support timing analysis for gateway latency

---

## 14. Reverse Engineering Requirements (NEW - SavvyCAN-like)

### 14.1 Signal Discovery

**REQ-RE-001:** Automatic signal boundary detection
**REQ-RE-002:** Signal value correlation analysis
**REQ-RE-003:** Message pattern recognition
**REQ-RE-004:** Unknown message identification

### 14.2 DBC Generation

**REQ-RE-DBC-001:** Generate DBC from discovered signals
**REQ-RE-DBC-002:** Support manual signal annotation
**REQ-RE-DBC-003:** Export discovered signals to various formats

---

## 15. Protocol Support (Phased)

### 15.1 Protocol Implementation Timeline

| Protocol | MVP | Phase 2 | Phase 3 | Phase 4 | Phase 5 |
|----------|-----|---------|---------|---------|---------|
| CAN 2.0A/B | ✓ | ✓ | ✓ | ✓ | ✓ |
| CAN FD | - | ✓ | ✓ | ✓ | ✓ |
| **CAN XL** | - | - | - | ✓ | ✓ |
| J1939 | - | ✓ | ✓ | ✓ | ✓ |
| UDS | - | ✓ | ✓ | ✓ | ✓ |
| OBD-II | - | ✓ | ✓ | ✓ | ✓ |
| DoIP | - | ✓ | ✓ | ✓ | ✓ |
| SOME/IP | - | ✓ | ✓ | ✓ | ✓ |
| LIN | - | - | ✓ | ✓ | ✓ |
| XCP/CCP | - | - | ✓ | ✓ | ✓ |
| KWP2000 | - | - | ✓ | ✓ | ✓ |
| **ISO 15118** | - | - | - | ✓ | ✓ |
| **SecOC** | - | - | - | ✓ | ✓ |
| FlexRay | - | - | - | - | ✓ |

### 15.2 Hardware Support Timeline

| Vendor | MVP | Phase 2 | Phase 3 | Phase 4 |
|--------|-----|---------|---------|---------|
| Stub (Simulation) | ✓ | ✓ | ✓ | ✓ |
| PEAK USB | ✓ | ✓ | ✓ | ✓ |
| Vector XL | - | ✓ | ✓ | ✓ |
| Kvaser | - | - | ✓ | ✓ |
| ETAS BOA | - | - | ✓ | ✓ |
| Intrepid neoVI | - | - | - | ✓ |
| SocketCAN | - | - | - | ✓ |
| **CSS Electronics** | - | - | - | ✓ |

---

## 16. Platform Roadmap

### 9.1 Platform Support Timeline

| Platform | MVP | Phase 2 | Phase 3 | Phase 4 | Phase 5 |
|----------|-----|---------|---------|---------|---------|
| macOS (Apple Silicon) | ✓ | ✓ | ✓ | ✓ | ✓ |
| macOS (Intel) | ✓ | ✓ | ✓ | ✓ | ✓ |
| Windows 10/11 | - | - | - | ✓ | ✓ |
| Linux (Ubuntu) | - | - | - | ✓ | ✓ |
| Linux (Fedora) | - | - | - | - | ✓ |
| Docker | - | - | ✓ | ✓ | ✓ |
| AWS Lambda | - | - | ✓ | ✓ | ✓ |
| Azure Functions | - | - | - | ✓ | ✓ |

### 9.2 UI Evolution

| Interface | MVP | Phase 2 | Phase 3 | Phase 4 |
|-----------|-----|---------|---------|---------|
| CLI | ✓ | ✓ | ✓ | ✓ |
| TUI (ratatui) | ✓ | ✓ | ✓ | ✓ |
| GUI (egui) | - | - | ✓ | ✓ |
| Web UI | - | - | ✓ | ✓ |
| REST API | - | - | ✓ | ✓ |

---

## 10. Non-Functional Requirements

### 10.1 Performance Baselines

| Metric | MVP Target | Full Target | CANoe Baseline |
|--------|------------|-------------|----------------|
| Message latency | < 5ms | < 1ms | ~0.5ms |
| Throughput | 1000 msg/s | 10000 msg/s | ~10000 msg/s |
| Memory (idle) | < 100MB | < 200MB | ~200MB |
| Memory (active) | < 300MB | < 500MB | ~500MB |
| CPU (idle) | < 5% | < 2% | ~1% |
| Startup time | < 3s | < 1s | ~2s |
| Ethernet throughput | - | 1 Gbps | 1 Gbps |

### 10.2 Reliability
- **REQ-REL-001:** No memory leaks (enforced by Rust)
- **REQ-REL-002:** No data races (enforced by Rust)
- **REQ-REL-003:** Graceful error handling with Result types
- **REQ-REL-004:** Recovery from hardware disconnection
- **REQ-REL-005:** 99.9% uptime for cloud deployments

### 10.3 Code Quality (AI-Enforced)
- **REQ-QUAL-001:** `cargo clippy` passes with no warnings
- **REQ-QUAL-002:** `cargo fmt` applied to all code
- **REQ-QUAL-003:** `cargo test` passes with >80% coverage
- **REQ-QUAL-004:** `cargo audit` shows no vulnerabilities
- **REQ-QUAL-005:** All public APIs documented with rustdoc

---

## 11. AI Agent Role Definitions

### 11.1 Architect Agent
**Responsibilities:**
- System design and crate structure
- API design and interface definitions
- Cross-cutting concerns (error handling, logging)
- Performance architecture decisions
- Security architecture

**Input Requirements:**
- Feature requirements from JC
- Protocol specifications
- Performance targets

**Output Deliverables:**
- Architecture decision records (ADRs)
- Crate dependency graphs
- API specifications
- Design documents

**Validation:**
- Design review by JC
- Consistency checks across modules

### 11.2 Protocol Agent
**Responsibilities:**
- CAN, CAN FD, LIN, FlexRay implementations
- J1939, UDS, OBD-II protocol stacks
- Automotive Ethernet (DoIP, SOME/IP, XCP)
- Protocol state machines
- Message encoding/decoding

**Input Requirements:**
- Protocol specifications (ISO, SAE standards)
- Existing C++ implementations for reference
- Test vectors from standards

**Output Deliverables:**
- Protocol crates with full implementation
- Unit tests with >90% coverage
- Property-based tests for encoding/decoding
- Protocol documentation

**Validation:**
- Conformance tests against standards
- Interoperability tests with real ECUs

### 11.3 Hardware Agent
**Responsibilities:**
- Driver implementations for all vendors
- FFI bindings to vendor SDKs
- Platform abstraction layer
- USB device enumeration
- Hardware error handling

**Input Requirements:**
- Vendor SDK documentation
- Hardware samples for testing
- Platform-specific requirements

**Output Deliverables:**
- Hardware driver crates
- FFI binding crates
- Platform abstraction traits
- Hardware setup documentation

**Validation:**
- Hardware integration tests
- Cross-platform compilation tests

### 11.4 Parser Agent
**Responsibilities:**
- DBC, DBF, LDF file parsers
- ARXML (AUTOSAR) parser
- ODX (diagnostics) parser
- A2L (XCP) parser
- FIBEX (FlexRay) parser
- BLF, MDF4 log parsers

**Input Requirements:**
- File format specifications
- Sample files for testing
- Existing parser implementations

**Output Deliverables:**
- Parser crates with full format support
- Fuzz tests for robustness
- Format documentation
- Sample file generators

**Validation:**
- Roundtrip tests (parse → serialize → parse)
- Compatibility tests with Vector tools
- Fuzz testing for crash resistance

### 11.5 UI Agent
**Responsibilities:**
- CLI implementation (clap)
- TUI implementation (ratatui)
- GUI implementation (egui)
- Web UI implementation
- REST API implementation

**Input Requirements:**
- UI/UX requirements
- Accessibility requirements
- Platform guidelines

**Output Deliverables:**
- UI crates for each interface
- User documentation
- Keyboard shortcut guides
- API documentation (OpenAPI)

**Validation:**
- Usability testing
- Accessibility compliance
- API contract tests

### 11.6 Test Agent
**Responsibilities:**
- Unit test implementation
- Integration test suites
- Property-based tests
- Performance benchmarks
- Fuzz testing
- End-to-end tests

**Input Requirements:**
- Test requirements from other agents
- Coverage targets
- Performance baselines

**Output Deliverables:**
- Comprehensive test suites
- Coverage reports
- Benchmark results
- Test documentation

**Validation:**
- Coverage > 80%
- All tests passing
- No performance regressions

### 11.7 DevOps Agent
**Responsibilities:**
- CI/CD pipeline configuration
- Docker containerization
- Cloud deployment (AWS, Azure)
- Release management
- Security scanning

**Input Requirements:**
- Deployment requirements
- Security requirements
- Platform targets

**Output Deliverables:**
- GitHub Actions workflows
- Dockerfiles
- Terraform/CloudFormation templates
- Release scripts

**Validation:**
- Successful deployments
- Security scan passing
- Automated release process

### 11.8 Documentation Agent
**Responsibilities:**
- API documentation (rustdoc)
- User guides and tutorials
- Architecture documentation
- Migration guides
- Example code

**Input Requirements:**
- Code from all agents
- User feedback
- Feature specifications

**Output Deliverables:**
- Complete rustdoc documentation
- User manual (mdBook)
- Video tutorials (scripts)
- Example projects

**Validation:**
- Documentation coverage
- User feedback
- Example code compiles and runs

---

## 12. Implementation Phases (Revised Timeline)

### Phase 1: MVP (Months 1-6)
**Goal:** Working CAN monitor on macOS

| Month | Deliverable | Agent |
|-------|-------------|-------|
| 1-2 | Core types, CAN protocol, stub driver | Architect, Protocol |
| 3-4 | DBC parser, message filtering, ASC logging | Parser, Protocol |
| 5-6 | CLI/TUI interface, PEAK driver, integration | UI, Hardware |

### Phase 2: Automotive Ethernet & Diagnostics (Months 7-12)
**Goal:** DoIP, SOME/IP, UDS, J1939

| Month | Deliverable | Agent |
|-------|-------------|-------|
| 7-8 | CAN FD, J1939 protocol | Protocol |
| 9-10 | DoIP, SOME/IP implementation | Protocol |
| 11-12 | UDS, OBD-II, Vector driver, BLF logging | Protocol, Hardware |

### Phase 3: Cloud & AI (Months 13-18)
**Goal:** REST API, Cloud deployment, AI integration

| Month | Deliverable | Agent |
|-------|-------------|-------|
| 13-14 | REST API, WebSocket streaming | UI, DevOps |
| 15-16 | Docker, AWS Lambda deployment | DevOps |
| 17-18 | AI integration, LIN protocol, XCP | Protocol, UI |

### Phase 4: Cross-Platform (Months 19-30)
**Goal:** Windows, Linux, GUI

| Month | Deliverable | Agent |
|-------|-------------|-------|
| 19-22 | Windows platform, drivers | Hardware |
| 23-26 | Linux platform, SocketCAN | Hardware |
| 27-30 | GUI (egui), additional hardware | UI, Hardware |

### Phase 5: Feature Parity (Months 31-36)
**Goal:** FlexRay, full CANoe parity

| Month | Deliverable | Agent |
|-------|-------------|-------|
| 31-33 | FlexRay protocol, FIBEX parser | Protocol, Parser |
| 34-36 | Plugin system, optimization, polish | All |

---

## 13. Gap Analysis

### 13.1 C++ Implementation vs Rust Specs

| Feature | C++ Status | Rust Spec | Gap |
|---------|------------|-----------|-----|
| CAN 2.0 | Complete | MVP | None |
| CAN FD | Partial | Phase 2 | Need full implementation |
| LIN | Complete | Phase 3 | Deferred |
| FlexRay | Partial | Phase 5 | Need full implementation |
| J1939 | Complete | Phase 2 | None |
| UDS | Complete | Phase 2 | None |
| DoIP | Missing | Phase 2 | **NEW FEATURE** |
| SOME/IP | Missing | Phase 2 | **NEW FEATURE** |
| XCP | Missing | Phase 3 | **NEW FEATURE** |
| REST API | Missing | Phase 3 | **NEW FEATURE** |
| AI Integration | Missing | Phase 3 | **NEW FEATURE** |

### 13.2 Our Specs vs CANoe Features

| Feature | CANoe | Our Target | Gap |
|---------|-------|------------|-----|
| CAPL Scripting | ✓ | Rust/Lua | Different approach |
| vTESTstudio | ✓ | Phase 4 | Test automation |
| DiVa | ✓ | Phase 4 | Diagnostic validation |
| CANape Integration | ✓ | Phase 3 | XCP support |
| AUTOSAR Support | ✓ | Phase 3 | ARXML parsing |

### 13.3 MVP vs JC's Full Vision

| Aspect | MVP | Full Vision | Timeline |
|--------|-----|-------------|----------|
| Protocols | CAN only | All automotive | 36 months |
| Platforms | macOS | All + Cloud | 30 months |
| UI | CLI/TUI | Full GUI + Web | 30 months |
| AI | None | Full integration | 18 months |
| Hardware | PEAK + Stub | All vendors | 30 months |

---

## 14. Glossary

| Term | Definition |
|------|-----------|
| **MVP** | Minimum Viable Product - smallest useful release |
| **DIL** | Driver Interface Layer - hardware abstraction |
| **CAN** | Controller Area Network - automotive bus protocol |
| **DoIP** | Diagnostics over IP - ISO 13400 |
| **SOME/IP** | Scalable service-Oriented MiddlewarE over IP |
| **UDS** | Unified Diagnostic Services - ISO 14229 |
| **XCP** | Universal Measurement and Calibration Protocol |
| **DBC** | Database CAN - Vector's signal database format |
| **ASC** | ASCII log format - human-readable message log |
| **BLF** | Binary Logging Format - Vector's binary log format |
| **MDF4** | Measurement Data Format v4 - ASAM standard |
| **ARXML** | AUTOSAR XML - system description format |
| **ODX** | Open Diagnostic data eXchange - ISO 22901 |
| **A2L** | ASAM MCD-2 MC - XCP description format |
| **PGN** | Parameter Group Number - J1939 message identifier |

---

**Document Status:** APPROVED FOR AI DEVELOPMENT  
**Project Owner:** JC  
**Last Updated:** January 2026  
**Version:** 4.0

---

## Appendix A: Competitive Research Summary (January 2026)

### A.1 Tools Researched

| Tool | Vendor | Key Strengths |
|------|--------|---------------|
| CANoe | Vector | Industry standard, CAPL scripting, comprehensive protocol support |
| CANalyzer | Vector | Analysis-focused, cost-effective |
| CANape | Vector | Measurement & calibration, XCP/A2L |
| vTESTstudio | Vector | Test automation, graphical test design |
| Vehicle Spy | Intrepid | Diagnostics, node simulation, Wireshark plugin |
| ETAS INCA | ETAS | Measurement/calibration, SOME/IP add-on |
| PCAN-Explorer | PEAK | CAN XL support, affordable |
| Kvaser CanKing | Kvaser | J1939, Linux ARM support |
| SavvyCAN | Open Source | Reverse engineering, signal discovery |

### A.2 Key Features Added from Research

1. **CAN XL Protocol** - New high-bandwidth protocol (10 Mbps, 2048 bytes)
2. **ISO 15118** - EV charging communication (Plug & Charge, V2G)
3. **SecOC** - AUTOSAR secure onboard communication
4. **Test Automation** - vTESTstudio-like graphical test design
5. **Measurement/Calibration** - CANape-like XCP DAQ/STIM
6. **Gateway Simulation** - CAN-to-Ethernet translation
7. **Reverse Engineering** - SavvyCAN-like signal discovery
8. **Wireshark Integration** - PCAPNG export, dissector plugin
9. **Bus Statistics** - Network load analysis, timing analysis
10. **MDF4.3** - Latest ASAM measurement data format

### A.3 Competitive Positioning

Our Rust implementation will differentiate through:
- **Cross-platform from day one** (macOS → Windows → Linux)
- **Cloud-native architecture** (REST API, Docker, Lambda)
- **AI-assisted analysis** (OpenAI/Azure OpenAI integration)
- **Open source** (vs. expensive commercial licenses)
- **Memory safety** (Rust vs. C++ vulnerabilities)
- **Modern developer experience** (CLI-first, scriptable)
