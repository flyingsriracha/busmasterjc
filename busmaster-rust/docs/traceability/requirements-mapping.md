# Requirements Traceability Matrix

This document maps requirements from `requirements.md` to implementation tasks and code.

## MVP Requirements (Phase 1)

### 3.1.1 Must Have (MVP Core)

| Requirement | Task | Status | Implementation |
|-------------|------|--------|----------------|
| CAN Protocol - Standard/Extended IDs | 1.2, 1.3 | 🔄 | `busmaster-core/src/frame.rs` |
| Stub Driver | 1.5 | 🔄 | `busmaster-hardware/src/stub.rs` |
| CLI Interface | 3.2 | ⏳ | `busmaster-cli/src/main.rs` |
| DBC File Parsing | 2.1 | ⏳ | `busmaster-db/src/dbc.rs` |
| ASC Logging | 2.3 | 🔄 | `busmaster-log/src/asc.rs` |
| Message Filtering | 2.4 | ⏳ | TBD |

### Code Quality Requirements (Section 10.3)

| Requirement ID | Description | Task | Status |
|----------------|-------------|------|--------|
| REQ-QUAL-001 | cargo clippy passes | 1.1.2, 1.1.3 | 🔄 |
| REQ-QUAL-002 | cargo fmt applied | 1.1.2, 1.1.3 | 🔄 |
| REQ-QUAL-003 | cargo test >80% coverage | 1.1.5 | ⏳ |
| REQ-QUAL-004 | cargo audit no vulnerabilities | 1.1.3 | ⏳ |
| REQ-QUAL-005 | rustdoc documentation | 1.2.10 | ⏳ |

### Performance Requirements (Section 10.1)

| Metric | MVP Target | Task | Status |
|--------|------------|------|--------|
| Message latency | < 5ms | 3.6.5 | ⏳ |
| Throughput | 1000 msg/s | 3.6.5 | ⏳ |
| Memory (idle) | < 100MB | 3.6.7 | ⏳ |
| Startup time | < 3s | 3.6 | ⏳ |

## Legend

- ✅ Complete
- 🔄 In Progress
- ⏳ Not Started
- ❌ Blocked

## Task to Requirement Mapping

### Task 1.1 - Project Setup
- REQ-QUAL-001 through REQ-QUAL-005

### Task 1.2 - Core Types
- MVP Core: CAN Protocol
- REQ-REL-001: No memory leaks (Rust enforced)
- REQ-REL-002: No data races (Rust enforced)

### Task 1.3 - CAN Protocol
- MVP Core: CAN Protocol - Standard/Extended IDs

### Task 1.4 - DIL Interface
- Design requirement: Hardware abstraction

### Task 1.5 - Stub Driver
- MVP Core: Stub Driver

### Task 2.1 - DBC Parser
- MVP Core: DBC File Parsing

### Task 2.3 - ASC Logger
- MVP Core: ASC Logging

### Task 2.4 - Message Filtering
- MVP Core: Message Filtering

### Task 3.2 - CLI Application
- MVP Core: CLI Interface

## Verification Matrix

| Requirement | Test Type | Test Location |
|-------------|-----------|---------------|
| CanFrame valid IDs | Unit | `busmaster-core/src/frame.rs` |
| CanFrame invalid IDs | Unit | `busmaster-core/src/frame.rs` |
| StubDriver loopback | Unit | `busmaster-hardware/src/stub.rs` |
| ASC format | Unit | `busmaster-log/src/asc.rs` |
| DBC parsing | Unit + Fuzz | `busmaster-db/src/dbc.rs` |
| E2E monitoring | Integration | `tests/` |
