# Task 3.4 Alternatives - No PEAK Hardware Available

**Date:** January 26, 2026  
**Issue:** PEAK PCAN library not installed on macOS  
**Status:** Evaluating alternatives

---

## 🔍 Current Situation

### What We Found
- ❌ No PEAK PCAN library at `/Library/Frameworks/PCBUSB.framework`
- ❌ No CAN tools installed (`candump`, `cantools`)
- ✅ macOS development environment ready
- ✅ All other code working perfectly

### What This Means
We cannot implement Task 3.4 (PEAK Driver) without:
1. PEAK PCAN-USB hardware adapter
2. PEAK PCAN library for macOS
3. Physical CAN bus for testing

---

## 🎯 Alternative Approaches

### Option 1: Virtual CAN Driver (Recommended for MVP)

**Implement a virtual CAN driver that simulates real hardware**

**Pros:**
- ✅ No hardware required
- ✅ Can test CLI-TUI communication
- ✅ Completes MVP without hardware dependency
- ✅ Can be implemented immediately
- ✅ Useful for development and testing

**Cons:**
- ❌ Not real hardware
- ❌ Can't connect to actual CAN bus
- ❌ Limited to simulation

**Implementation:**
- Create `VirtualDriver` that uses shared memory or sockets
- Multiple processes can connect to same virtual bus
- CLI and TUI can communicate through virtual bus
- Simulates real hardware behavior

**Time:** 1-2 days

---

### Option 2: SocketCAN on Linux (Real Hardware Alternative)

**Use Linux SocketCAN instead of PEAK on macOS**

**Pros:**
- ✅ Native Linux CAN support
- ✅ Works with many USB-CAN adapters
- ✅ No proprietary drivers needed
- ✅ Well-documented and mature

**Cons:**
- ❌ Requires Linux (VM or dual boot)
- ❌ Still needs USB-CAN hardware
- ❌ Platform switch from macOS

**Implementation:**
- Set up Linux VM or dual boot
- Install SocketCAN tools
- Implement SocketCAN driver
- Test with USB-CAN adapter

**Time:** 2-3 days (+ hardware acquisition)

---

### Option 3: Skip Hardware Driver for MVP

**Complete MVP without real hardware driver**

**Pros:**
- ✅ Focus on software completion
- ✅ Complete Tasks 3.5 and 3.6
- ✅ Reach MVP milestone faster
- ✅ Add hardware support in Phase 2

**Cons:**
- ❌ No real hardware support in MVP
- ❌ CLI-TUI still can't communicate
- ❌ Less impressive demo

**Implementation:**
- Mark Task 3.4 as "deferred"
- Proceed to Task 3.5 (Platform Layer)
- Proceed to Task 3.6 (MVP Integration & Testing)
- Add hardware support in Phase 2

**Time:** 0 days (skip to next task)

---

### Option 4: Purchase PEAK Hardware

**Buy PEAK PCAN-USB adapter and implement as planned**

**Pros:**
- ✅ Original plan
- ✅ Professional hardware
- ✅ Well-documented
- ✅ Industry standard

**Cons:**
- ❌ Costs money (~$150-300)
- ❌ Shipping time
- ❌ Need CAN bus for testing

**Implementation:**
- Purchase PEAK PCAN-USB adapter
- Install PCAN library for macOS
- Implement as originally planned
- Test with hardware

**Time:** 1-2 weeks (shipping) + 4-7 days (implementation)

---

## 💡 Recommendation: Option 1 - Virtual CAN Driver

### Why Virtual Driver?

1. **Completes MVP Goal**
   - Enables CLI-TUI communication
   - Demonstrates pub/sub architecture
   - Shows multi-process messaging

2. **No Hardware Dependency**
   - Can develop and test immediately
   - No waiting for hardware
   - No additional costs

3. **Still Valuable**
   - Useful for automated testing
   - Good for CI/CD pipelines
   - Developers can test without hardware

4. **Easy to Add Real Hardware Later**
   - Virtual driver proves the architecture
   - Real hardware driver is just another implementation
   - Can add PEAK/SocketCAN in Phase 2

### What Virtual Driver Provides

**Architecture:**
```
CLI Process          Virtual CAN Bus          TUI Process
┌─────────┐         ┌──────────────┐         ┌─────────┐
│ Virtual │────────▶│ Shared Memory│────────▶│ Virtual │
│ Driver  │         │  or Socket   │         │ Driver  │
└─────────┘         └──────────────┘         └─────────┘
```

**Features:**
- Multiple processes can connect
- Messages broadcast to all connected processes
- Simulates real CAN bus behavior
- Configurable baudrate (for realism)
- Error injection (for testing)

**Use Cases:**
- CLI-TUI communication demo
- Automated testing
- Development without hardware
- CI/CD integration
- Training and education

---

## 🏗️ Virtual Driver Implementation Plan

### Phase 1: Design (2 hours)
1. Choose IPC mechanism (Unix domain sockets recommended)
2. Define message protocol
3. Design connection management
4. Plan error handling

### Phase 2: Implementation (1 day)
1. Create `VirtualDriver` struct
2. Implement `CanDriver` trait
3. Implement socket server/client
4. Implement message broadcasting
5. Add connection management

### Phase 3: Testing (4 hours)
1. Unit tests
2. Integration tests
3. CLI-TUI communication test
4. Multi-process test

### Phase 4: Documentation (2 hours)
1. Document virtual driver usage
2. Update CLI/TUI docs
3. Create demo guide

**Total Time:** 1-2 days

---

## 📋 Virtual Driver Specification

### Requirements
- Multiple processes can connect
- Messages sent by one process received by all others
- Simulates CAN bus behavior
- Works on macOS (and Linux, Windows)
- No external dependencies

### API
```rust
pub struct VirtualDriver {
    socket_path: PathBuf,
    connection: Option<UnixStream>,
    rx_thread: Option<JoinHandle<()>>,
}

impl CanDriver for VirtualDriver {
    fn list_devices() -> Result<Vec<DeviceInfo>> {
        // Return virtual device info
    }
    
    fn open_channel(&mut self, config: ChannelConfig) -> Result<ChannelHandle> {
        // Connect to virtual bus socket
    }
    
    fn close_channel(&mut self, handle: ChannelHandle) -> Result<()> {
        // Disconnect from socket
    }
    
    fn send_frame(&mut self, frame: &CanFrame) -> Result<()> {
        // Send to socket, broadcast to all
    }
    
    fn receive_frame(&mut self) -> Result<Option<CanFrame>> {
        // Receive from socket
    }
}
```

### Socket Protocol
```
Message Format:
[4 bytes: message length]
[N bytes: serialized CanFrame]

Connection:
1. Client connects to Unix socket
2. Server adds client to broadcast list
3. Messages sent to all connected clients
4. Client disconnects, removed from list
```

---

## 🎯 Decision Matrix

| Criteria | Virtual Driver | SocketCAN | Skip | Buy PEAK |
|----------|---------------|-----------|------|----------|
| **Time to implement** | 1-2 days | 2-3 days | 0 days | 1-2 weeks |
| **Cost** | $0 | $50-150 | $0 | $150-300 |
| **Hardware needed** | None | USB-CAN | None | PEAK adapter |
| **Platform** | macOS ✅ | Linux | macOS ✅ | macOS ✅ |
| **CLI-TUI comm** | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes |
| **Real CAN bus** | ❌ No | ✅ Yes | ❌ No | ✅ Yes |
| **MVP complete** | ✅ Yes | ✅ Yes | ⚠️ Partial | ✅ Yes |
| **Testing** | ✅ Easy | ⚠️ Needs HW | ✅ Easy | ⚠️ Needs HW |
| **CI/CD** | ✅ Yes | ⚠️ Limited | ✅ Yes | ❌ No |

**Winner:** Virtual Driver (best balance of time, cost, and functionality)

---

## 🚀 Proposed Action Plan

### Immediate (Today)
1. ✅ Implement Virtual Driver
2. ✅ Test CLI-TUI communication
3. ✅ Update documentation

### Short-term (This Week)
1. Complete Task 3.5 (Platform Layer)
2. Complete Task 3.6 (MVP Integration & Testing)
3. Reach MVP milestone!

### Long-term (Phase 2)
1. Add PEAK driver (if hardware acquired)
2. Add SocketCAN driver (for Linux)
3. Add other hardware vendors

---

## 📝 Task List Update

### Modify Task 3.4
**Old:** 3.4 PEAK Driver (macOS)  
**New:** 3.4 Virtual CAN Driver (cross-platform)

**Subtasks:**
- [ ] 3.4.1 Design virtual bus architecture
- [ ] 3.4.2 Implement VirtualDriver struct
- [ ] 3.4.3 Implement socket server/client
- [ ] 3.4.4 Implement message broadcasting
- [ ] 3.4.5 Implement connection management
- [ ] 3.4.6 Implement CanDriver trait
- [ ] 3.4.7 Write unit tests
- [ ] 3.4.8 Write integration tests
- [ ] 3.4.9 Test CLI-TUI communication
- [ ] 3.4.10 Document virtual driver
- [ ] 3.4.11 Create demo guide

### Add to Phase 2
- [ ] 4.10 PEAK Driver (macOS) - when hardware available
- [ ] 4.11 SocketCAN Driver (Linux)

---

## ✅ Benefits of Virtual Driver

### For Development
- No hardware dependency
- Fast iteration
- Easy debugging
- Consistent behavior

### For Testing
- Automated tests
- CI/CD integration
- Error injection
- Load testing

### For Users
- Try BUSMASTER without hardware
- Learn CAN protocols
- Develop applications
- Training and education

### For Project
- Complete MVP on time
- Demonstrate architecture
- Prove multi-process messaging
- Foundation for real drivers

---

## 🎉 Conclusion

**Recommendation:** Implement Virtual CAN Driver (Option 1)

**Rationale:**
1. Completes MVP without hardware
2. Enables CLI-TUI communication
3. Valuable for testing and development
4. Can add real hardware later
5. Fast to implement (1-2 days)

**Next Steps:**
1. Get user approval
2. Start implementing Virtual Driver
3. Test CLI-TUI communication
4. Complete MVP!

**Status:** AWAITING USER DECISION

