# Implementation Plan - Virtual Driver Now, ETAS Later

**Date:** January 26, 2026  
**Decision:** Implement Virtual driver now, ETAS BOA driver when hardware arrives  
**Status:** APPROVED

---

## 📋 Plan Summary

### Phase 1: Virtual Driver (Now - 1-2 days)
✅ Implement Virtual CAN driver for MVP  
✅ Enable CLI-TUI communication  
✅ Complete MVP without hardware dependency

### Phase 2: ETAS BOA Driver (When hardware arrives - 5-7 days)
⏳ User brings ETAS hardware from work  
⏳ Implement ETAS BOA driver using existing C++ code as reference  
⏳ Test with real hardware

---

## 🎯 Phase 1: Virtual Driver Implementation

### What We'll Build
A virtual CAN bus that allows multiple processes to communicate:

```
CLI Process          Virtual CAN Bus          TUI Process
┌─────────┐         ┌──────────────┐         ┌─────────┐
│ Virtual │────────▶│ Unix Socket  │────────▶│ Virtual │
│ Driver  │         │  Broadcast   │         │ Driver  │
└─────────┘         └──────────────┘         └─────────┘
```

### Features
- Multiple processes can connect
- Messages broadcast to all connected processes
- Simulates real CAN bus behavior
- Works on macOS, Linux, Windows
- No external dependencies

### Implementation Tasks

**Task 3.4: Virtual CAN Driver** (Modified from PEAK Driver)

- [ ] 3.4.1 Design virtual bus architecture
- [ ] 3.4.2 Implement socket server (bus coordinator)
- [ ] 3.4.3 Implement VirtualDriver struct
- [ ] 3.4.4 Implement CanDriver trait for VirtualDriver
- [ ] 3.4.5 Implement device discovery (virtual devices)
- [ ] 3.4.6 Implement channel open/close
- [ ] 3.4.7 Implement frame transmission
- [ ] 3.4.8 Implement frame reception
- [ ] 3.4.9 Write unit tests
- [ ] 3.4.10 Write integration tests (CLI-TUI communication)
- [ ] 3.4.11 Document virtual driver usage

**Time Estimate:** 1-2 days

---

## 🔧 Phase 2: ETAS BOA Driver (Later)

### What We Have
**Existing C++ Implementation:**
- `source/Sources/BUSMASTER/CAN_ETAS_BOA/` - Full C++ implementation
- BOA SDK headers in `EXTERNAL/BOA_V2/Include/`
- Reference for API usage and patterns

### What We'll Do
**Convert C++ to Rust:**
1. Study existing C++ implementation
2. Create Rust FFI bindings for BOA API
3. Implement EtasBoaDriver struct
4. Test with real hardware

### BOA API Overview
From the C++ code, we can see:
- Uses OCI (Open Controller Interface) for CAN
- Uses CSI (Common System Interface) for device management
- Supports BOA versions 1.4, 1.5, and 2.0+
- Complex but well-documented API

### Implementation Tasks

**Task 4.10: ETAS BOA Driver** (New task for Phase 2)

- [ ] 4.10.1 Install ETAS BOA SDK on macOS
- [ ] 4.10.2 Verify hardware connection
- [ ] 4.10.3 Create Rust FFI bindings for BOA API
- [ ] 4.10.4 Implement EtasBoaDriver struct
- [ ] 4.10.5 Implement CanDriver trait for EtasBoaDriver
- [ ] 4.10.6 Implement device discovery
- [ ] 4.10.7 Implement channel management
- [ ] 4.10.8 Implement frame TX/RX
- [ ] 4.10.9 Write integration tests with hardware
- [ ] 4.10.10 Document ETAS BOA setup
- [ ] 4.10.11 Update CLI/TUI to support ETAS driver

**Time Estimate:** 5-7 days (when hardware available)

---

## 📚 Reference Materials

### Existing C++ Code
**Files to study:**
- `source/Sources/BUSMASTER/CAN_ETAS_BOA/CAN_ETAS_BOA.cpp` - Main implementation
- `source/Sources/BUSMASTER/CAN_ETAS_BOA/CAN_ETAS_BOA.h` - Header
- `source/Sources/BUSMASTER/CAN_ETAS_BOA/EXTERNAL/BOA_V2/Include/` - BOA headers

**Key Insights:**
```cpp
// BOA includes different versions
#if BOA_VERSION == BOA_VERSION_1_4
#include "EXTERNAL/BOA 1.4/Include/OCI/ocican.h"
#include "EXTERNAL/BOA 1.4/Include/CSI/csisfs.h"
#elif BOA_VERSION == BOA_VERSION_1_5
#include "EXTERNAL/BOA 1.5/Include/OCI/ocican.h"
#include "EXTERNAL/BOA 1.5/Include/CSI/csisfs.h"
#elif BOA_VERSION >= BOA_VERSION_2_0
#include "EXTERNAL/BOA_V2/Include/OCI/ocican.h"
#include "EXTERNAL/BOA_V2/Include/CSI/csisfs.h"
#endif
```

### BOA API Structure
- **OCI (Open Controller Interface)** - CAN communication
- **CSI (Common System Interface)** - Device management
- **BOA** - Overall framework

---

## 🚀 Immediate Next Steps

### Today: Start Virtual Driver
1. ✅ Create `crates/busmaster-hardware/src/virtual/` module
2. ✅ Implement socket-based virtual bus
3. ✅ Implement VirtualDriver
4. ✅ Test CLI-TUI communication
5. ✅ Update documentation

### When Hardware Arrives: ETAS BOA Driver
1. ⏳ User brings ETAS hardware from work
2. ⏳ Install BOA SDK
3. ⏳ Study C++ implementation
4. ⏳ Create Rust bindings
5. ⏳ Implement driver
6. ⏳ Test with hardware

---

## 📊 Benefits of This Approach

### Virtual Driver First
✅ **Fast MVP completion** - 1-2 days vs weeks  
✅ **No hardware dependency** - Can develop immediately  
✅ **Enables CLI-TUI communication** - Achieves MVP goal  
✅ **Useful long-term** - Testing, CI/CD, development  
✅ **No cost** - Free to implement

### ETAS BOA Later
✅ **Real hardware when available** - Professional grade  
✅ **Reference implementation** - C++ code to learn from  
✅ **Proven approach** - Already works in original BUSMASTER  
✅ **Time to prepare** - Study API while waiting for hardware  
✅ **Better implementation** - Can learn from virtual driver experience

---

## 🎯 Success Criteria

### Phase 1: Virtual Driver (MVP)
- ✅ Virtual driver implements CanDriver trait
- ✅ Multiple processes can connect
- ✅ Messages broadcast correctly
- ✅ CLI can send, TUI can receive
- ✅ TUI can send, CLI can receive
- ✅ All tests pass
- ✅ Documentation complete

### Phase 2: ETAS BOA Driver
- ✅ BOA SDK installed and working
- ✅ Hardware detected
- ✅ Driver implements CanDriver trait
- ✅ Can send/receive frames
- ✅ CLI and TUI work with ETAS hardware
- ✅ Integration tests pass
- ✅ Documentation complete

---

## 📝 Task List Updates

### Update tasks.md

**Task 3.4: Virtual CAN Driver** (Modified)
- Change from "PEAK Driver" to "Virtual CAN Driver"
- Update subtasks for virtual implementation
- Keep time estimate: 1-2 days

**Task 4.10: ETAS BOA Driver** (New - Phase 2)
- Add to Phase 2 task list
- Reference existing C++ code
- Time estimate: 5-7 days
- Requires hardware

---

## 🔗 Resources

### For Virtual Driver
- Unix domain sockets: `std::os::unix::net::UnixStream`
- Tokio async: `tokio::net::UnixListener`
- Message serialization: `serde` + `bincode`

### For ETAS BOA Driver
- **C++ Reference:** `source/Sources/BUSMASTER/CAN_ETAS_BOA/`
- **BOA SDK:** Will be installed when hardware arrives
- **BOA Documentation:** Included with SDK
- **Rust FFI:** `bindgen` for automatic binding generation

---

## 💡 Learning Opportunity

### While Waiting for Hardware
We can:
1. Study the C++ ETAS implementation
2. Understand BOA API structure
3. Plan the Rust FFI bindings
4. Design the driver architecture
5. Prepare test cases

This way, when hardware arrives, we'll be ready to implement quickly!

---

## 🎉 Conclusion

**Perfect plan!**

1. **Now:** Implement Virtual driver (1-2 days)
   - Complete MVP
   - Enable CLI-TUI communication
   - No hardware needed

2. **Later:** Implement ETAS BOA driver (5-7 days)
   - When you bring hardware from work
   - Use C++ code as reference
   - Professional hardware support

**This gives us:**
- ✅ Fast MVP completion
- ✅ Working CLI-TUI communication
- ✅ Real hardware support later
- ✅ Best of both worlds!

**Ready to start implementing the Virtual driver?**

