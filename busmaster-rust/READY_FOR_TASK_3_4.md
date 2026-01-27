# Ready for Task 3.4: PEAK Driver

**Date:** January 26, 2026  
**Status:** ✅ READY TO START  
**Previous Tasks:** 3.1, 3.2, 3.3 COMPLETE

---

## ✅ Pre-Flight Checklist

### Code Quality
- ✅ **168 tests passing** (1 CAN FD test deferred to Phase 2)
- ✅ **Zero test failures**
- ✅ **Zero clippy errors** (6 warnings for unused future features)
- ✅ **All code formatted** with rustfmt
- ✅ **Documentation complete** for all completed tasks

### Functionality
- ✅ **CLI working** - All 20 QA tests passing
- ✅ **TUI working** - Test mode functional
- ✅ **Engine working** - All 12 tests passing
- ✅ **DBC parser working** - Signal extraction functional
- ✅ **ASC logger working** - File output correct
- ✅ **Message filtering working** - All filter types functional

### Documentation
- ✅ **User guides** - WELCOME, TESTING_GUIDE, QUICKSTART_CLI, QUICKSTART_TUI
- ✅ **Technical docs** - CLI_IMPLEMENTATION, TUI_IMPLEMENTATION, ENGINE_IMPLEMENTATION
- ✅ **Test results** - QA_TEST_RESULTS, test_basic.sh
- ✅ **Status reports** - PROJECT_STATUS_CHECKPOINT, CANFD_BUG_DEFERRED

---

## 📋 Task 3.4: PEAK Driver (macOS)

### Overview
Implement real CAN hardware support using PEAK PCAN USB adapter on macOS.

### Subtasks (11 total)
- [ ] 3.4.1 Create PEAK FFI bindings module 🔌
- [ ] 3.4.2 Document all unsafe blocks with SAFETY comments 🔌
- [ ] 3.4.3 Implement PeakDriver struct 🔌
- [ ] 3.4.4 Implement device discovery 🔌
- [ ] 3.4.5 Implement channel open/close 🔌
- [ ] 3.4.6 Implement frame transmission 🔌
- [ ] 3.4.7 Implement frame reception 🔌
- [ ] 3.4.8 Implement baudrate configuration 🔌
- [ ] 3.4.9 Implement error handling 🔌
- [ ] 3.4.10 Write integration tests (requires hardware) 🧪
- [ ] 3.4.11 Document PEAK driver setup 📚

### Time Estimate
- **Implementation:** 2-4 days
- **Testing:** 1-2 days (requires hardware)
- **Documentation:** 1 day
- **Total:** 4-7 days

---

## 🔧 Prerequisites

### Hardware
- **PEAK PCAN-USB adapter** (or compatible)
- **CAN bus** (real or simulated with 2 adapters)
- **macOS computer** (current development environment)

### Software
- **PEAK PCAN library for macOS**
  - Download from: https://www.peak-system.com/
  - Install PCAN-Basic API
  - Verify installation: `/Library/Frameworks/PCBUSB.framework`

### Knowledge
- **FFI (Foreign Function Interface)** in Rust
- **Unsafe Rust** - careful memory management
- **CAN bus protocols** - already implemented
- **macOS frameworks** - linking and loading

---

## 📚 Resources

### PEAK Documentation
- **PCAN-Basic API Documentation**
  - https://www.peak-system.com/PCAN-Basic.239.0.html
- **PCAN-USB User Manual**
  - https://www.peak-system.com/produktcd/Pdf/English/PCAN-USB_UserMan_eng.pdf

### Rust FFI Resources
- **Rust FFI Guide:** https://doc.rust-lang.org/nomicon/ffi.html
- **bindgen:** https://rust-lang.github.io/rust-bindgen/
- **Example:** Similar to how we'd bind to C libraries

### CAN Bus Resources
- **ISO 11898-1** - CAN specification
- **Bosch CAN Specification 2.0**

---

## 🏗️ Implementation Plan

### Phase 1: FFI Bindings (Day 1)
1. Create `crates/busmaster-hardware/src/peak/` module
2. Use `bindgen` to generate Rust bindings from PCAN headers
3. Create safe wrapper types around raw FFI
4. Document all `unsafe` blocks with SAFETY comments

**Files to create:**
- `crates/busmaster-hardware/src/peak/mod.rs`
- `crates/busmaster-hardware/src/peak/ffi.rs`
- `crates/busmaster-hardware/src/peak/types.rs`

### Phase 2: Driver Implementation (Days 2-3)
1. Implement `PeakDriver` struct
2. Implement `CanDriver` trait for `PeakDriver`
3. Implement device discovery
4. Implement channel management
5. Implement frame TX/RX
6. Implement error handling

**Files to create:**
- `crates/busmaster-hardware/src/peak/driver.rs`

### Phase 3: Testing (Day 4)
1. Write unit tests (mocked)
2. Write integration tests (requires hardware)
3. Test with CLI
4. Test with TUI
5. Verify CLI-TUI communication works!

**Files to create:**
- `crates/busmaster-hardware/src/peak/tests.rs`
- `crates/busmaster-hardware/tests/peak_integration.rs`

### Phase 4: Documentation (Day 5)
1. Document PEAK driver setup
2. Document hardware requirements
3. Update CLI/TUI docs
4. Create troubleshooting guide

**Files to update:**
- `crates/busmaster-hardware/README.md`
- `WELCOME.md`
- `QUICKSTART_CLI.md`
- `QUICKSTART_TUI.md`

---

## 🎯 Success Criteria

### Functional Requirements
- ✅ Driver loads PCAN library
- ✅ Device enumeration works
- ✅ Channel opens at various baudrates (125k, 250k, 500k, 1M)
- ✅ Frames transmit successfully
- ✅ Frames receive correctly
- ✅ Error conditions handled gracefully

### Integration Requirements
- ✅ CLI can send messages via PEAK driver
- ✅ CLI can monitor messages via PEAK driver
- ✅ TUI can display messages via PEAK driver
- ✅ **CLI and TUI communicate through real CAN bus!**

### Quality Requirements
- ✅ All unsafe code documented with SAFETY comments
- ✅ Integration tests pass (with hardware)
- ✅ No memory leaks
- ✅ Proper error handling
- ✅ Documentation complete

---

## 🚨 Challenges & Risks

### Technical Challenges
1. **FFI Complexity**
   - Unsafe code requires careful review
   - Memory management across FFI boundary
   - Error handling from C library

2. **Hardware Dependency**
   - Need physical PEAK adapter
   - Need CAN bus for testing
   - Platform-specific code (macOS only)

3. **Async Integration**
   - PCAN library is synchronous
   - Need to integrate with async Tokio runtime
   - Polling vs. event-driven

### Mitigation Strategies
1. **FFI Safety**
   - Use `bindgen` for automatic binding generation
   - Wrap all unsafe code in safe abstractions
   - Document every unsafe block with SAFETY comments
   - Review all pointer operations carefully

2. **Hardware Testing**
   - Start with device enumeration (no CAN bus needed)
   - Use loopback mode for initial testing
   - Test with 2 adapters for full integration

3. **Async Integration**
   - Use `tokio::task::spawn_blocking` for sync calls
   - Implement polling loop in background task
   - Use channels for message passing

---

## 📝 Code Structure

### Proposed Module Layout
```
crates/busmaster-hardware/src/
├── lib.rs                    # Re-exports
├── stub.rs                   # Existing stub driver
└── peak/
    ├── mod.rs                # Module exports
    ├── ffi.rs                # Raw FFI bindings
    ├── types.rs              # Safe wrapper types
    ├── driver.rs             # PeakDriver implementation
    ├── error.rs              # Error types
    └── tests.rs              # Unit tests
```

### Key Types
```rust
pub struct PeakDriver {
    handle: PcanHandle,
    config: ChannelConfig,
    rx_thread: Option<JoinHandle<()>>,
}

impl CanDriver for PeakDriver {
    fn list_devices() -> Result<Vec<DeviceInfo>>;
    fn open_channel(&mut self, config: ChannelConfig) -> Result<ChannelHandle>;
    fn close_channel(&mut self, handle: ChannelHandle) -> Result<()>;
    fn send_frame(&mut self, frame: &CanFrame) -> Result<()>;
    fn receive_frame(&mut self) -> Result<Option<CanFrame>>;
}
```

---

## 🔗 Dependencies

### Cargo.toml Updates
```toml
[target.'cfg(target_os = "macos")'.dependencies]
# PEAK PCAN library bindings
# May need to create or find existing crate
```

### System Dependencies
- PCAN-Basic framework installed
- USB permissions configured
- Driver loaded

---

## 🧪 Testing Strategy

### Unit Tests (No Hardware)
- Device info parsing
- Error handling
- Type conversions
- Configuration validation

### Integration Tests (Requires Hardware)
- Device enumeration
- Channel open/close
- Frame transmission
- Frame reception
- Baudrate configuration
- Error conditions

### Manual Tests
- CLI send/receive
- TUI monitoring
- CLI-TUI communication
- Long-duration stability
- Error recovery

---

## 📖 Documentation Plan

### User Documentation
1. **Hardware Setup Guide**
   - PEAK adapter installation
   - Driver installation
   - USB permissions
   - Troubleshooting

2. **CLI Usage with PEAK**
   - `--driver peak` option
   - Device selection
   - Baudrate configuration

3. **TUI Usage with PEAK**
   - Automatic device detection
   - Status display

### Technical Documentation
1. **PEAK Driver Implementation**
   - Architecture overview
   - FFI safety considerations
   - Async integration
   - Error handling

2. **API Documentation**
   - Rustdoc for all public APIs
   - Examples
   - Safety notes

---

## 🎉 What This Enables

### New Capabilities
- ✅ **Real CAN hardware support**
- ✅ **CLI-TUI communication** (through physical bus)
- ✅ **Production-ready monitoring**
- ✅ **Real-world testing**

### User Benefits
- Monitor real CAN buses
- Send messages to real ECUs
- Debug automotive systems
- Validate CAN protocols
- Professional tool for automotive development

---

## 🚀 Next Steps

### Immediate Actions
1. **Verify PEAK hardware availability**
   - Do we have PEAK adapter?
   - Is PCAN library installed?
   - Can we test on macOS?

2. **Review PEAK documentation**
   - PCAN-Basic API
   - Example code
   - Error codes

3. **Plan FFI approach**
   - Use bindgen?
   - Manual bindings?
   - Existing crate?

### Start Implementation
Once hardware is confirmed:
1. Create `peak` module
2. Generate FFI bindings
3. Implement `PeakDriver`
4. Test with hardware
5. Document everything

---

## 📞 Questions to Answer

Before starting Task 3.4:

1. **Do we have PEAK hardware?**
   - PCAN-USB adapter available?
   - CAN bus for testing?

2. **Is PCAN library installed?**
   - Check: `/Library/Frameworks/PCBUSB.framework`
   - Version?

3. **Alternative hardware?**
   - If no PEAK, consider:
     - SocketCAN on Linux
     - Other USB-CAN adapters
     - Virtual CAN (vcan)

4. **Timeline expectations?**
   - How much time for this task?
   - Hardware testing availability?

---

**Status:** ✅ READY TO START TASK 3.4

**Waiting for:** Hardware confirmation and go-ahead from user

