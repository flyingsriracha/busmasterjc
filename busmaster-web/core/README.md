# BUSMASTER Core - C++ Library

Platform-independent C++ library for CAN/LIN bus communication.

## Overview

This library provides the core functionality for BUSMASTER Web, extracted and modernized from the legacy Windows MFC application. It removes all Windows-specific dependencies and provides clean, modern C++ interfaces.

## Features

- ✅ Platform-independent interfaces (no Windows dependencies)
- ✅ Modern C++17 code
- ✅ Virtual CAN driver for testing
- ⏳ Real hardware driver support (Phase 2)
- ⏳ Message database management (Phase 2)
- ⏳ Signal interpretation (Phase 2)

## Architecture

```
┌─────────────────────────────────────┐
│     Node.js Backend (N-API)         │
└────────────────┬────────────────────┘
                 │
┌────────────────┴────────────────────┐
│      IDriverInterface (Abstract)    │
└────────────────┬────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
┌───────┴──────┐  ┌──────┴────────┐
│  VirtualCAN  │  │  HardwareCAN  │
│   Driver     │  │   Drivers     │
└──────────────┘  └───────────────┘
```

## Building

### Prerequisites
- CMake 3.20+
- C++17 compatible compiler
  - Windows: Visual Studio 2019+ or MinGW
  - Linux: GCC 11+ or Clang 13+
  - macOS: Xcode 13+

### Build Steps

```bash
# Create build directory
mkdir build
cd build

# Configure
cmake ..

# Build
cmake --build .

# Install (optional)
cmake --install .
```

### Build Options

```bash
# Debug build
cmake -DCMAKE_BUILD_TYPE=Debug ..

# Release build
cmake -DCMAKE_BUILD_TYPE=Release ..

# Without tests
cmake -DBUILD_TESTS=OFF ..
```

## Usage

### C++ Example

```cpp
#include <busmaster/IDriverInterface.h>
#include <busmaster/BusmasterTypes.h>

using namespace BusmasterCore;

// Create driver instance
auto driver = std::make_unique<VirtualCANDriver>();

// Initialize
driver->Initialize();

// List hardware
std::vector<HardwareInterface> interfaces;
driver->ListHardware(interfaces);

// Configure
ControllerConfig config;
config.baudrate = 500000;
config.mode = ControllerMode::ACTIVE;
config.selfReception = true;

driver->SelectHardware(interfaces[0].id, config);

// Register callback
driver->RegisterMessageCallback([](const CANMessage& msg, void* userData) {
    printf("Received: ID=0x%X, Data=[", msg.id);
    for (int i = 0; i < msg.length; i++) {
        printf("%02X ", msg.data[i]);
    }
    printf("]\n");
}, nullptr);

// Start
driver->Start();

// Send message
CANMessage msg = {};
msg.id = 0x123;
msg.extended = false;
msg.length = 8;
msg.data[0] = 0xAA;
msg.data[1] = 0xBB;
driver->SendMessage(msg);

// Stop
driver->Stop();
driver->Shutdown();
```

## API Reference

### IDriverInterface

Main interface for all hardware drivers.

**Key Methods:**
- `Initialize()` - Initialize driver
- `ListHardware()` - Get available hardware
- `SelectHardware()` - Configure hardware
- `Start()` / `Stop()` - Control CAN controller
- `SendMessage()` - Send CAN message
- `RegisterMessageCallback()` - Receive messages
- `GetStatistics()` - Get network statistics

See `include/IDriverInterface.h` for full API.

### BusmasterTypes

Common data structures used throughout the library.

**Key Types:**
- `CANMessage` - CAN message structure
- `ControllerConfig` - Controller configuration
- `NetworkStatistics` - Network statistics
- `ErrorCounter` - Error counters

See `include/BusmasterTypes.h` for full definitions.

## Integration with Node.js

The C++ core is exposed to Node.js via N-API bindings:

```
Node.js (TypeScript)
    ↓ N-API
C++ Core Library
    ↓ Driver Interface
Hardware Drivers
```

See `../server/src/native/` for N-API binding code.

## Testing

```bash
cd build
ctest --output-on-failure
```

## License

LGPL-3.0 - Same as original BUSMASTER

## Contributing

See main project CONTRIBUTING.md

