# BUSMASTER CLI - Quick Start Guide

**You can now test the CLI!** 🎉

---

## Installation

### Option 1: Run from source (recommended for development)
```bash
cd source/busmaster-rust
cargo run --bin busmaster -- [COMMAND]
```

### Option 2: Build release binary
```bash
cd source/busmaster-rust
cargo build --release --package busmaster-cli
# Binary will be at: target/release/busmaster
```

### Option 3: Install globally
```bash
cd source/busmaster-rust
cargo install --path crates/busmaster-cli
# Now you can run: busmaster [COMMAND]
```

---

## Quick Test

### 1. Check it works
```bash
cd source/busmaster-rust
cargo run --bin busmaster -- --help
```

### 2. List available drivers
```bash
cargo run --bin busmaster -- list
```

### 3. Send a test message
```bash
cargo run --bin busmaster -- send --id 0x123 --data "01 02 03 04"
```

Expected output:
```
✓ Sent: ID=0x123 DLC=4 Data=01 02 03 04
```

### 4. Monitor CAN bus (will wait for messages)
```bash
cargo run --bin busmaster -- monitor --driver stub --max-messages 5
```

This will start monitoring and wait for messages. Press Ctrl+C to stop.

---

## Fun Demo: Send and Monitor

Open two terminals:

**Terminal 1 - Start monitoring:**
```bash
cd source/busmaster-rust
cargo run --bin busmaster -- monitor --driver stub
```

**Terminal 2 - Send messages:**
```bash
cd source/busmaster-rust

# Send some test messages
cargo run --bin busmaster -- send --id 0x100 --data "11 22 33 44"
cargo run --bin busmaster -- send --id 0x200 --data "AA BB CC DD"
cargo run --bin busmaster -- send --id 0x300 --data "01 02 03 04 05 06 07 08"
```

You should see the messages appear in Terminal 1!

---

## Test with DBC Database

### 1. Use the sample DBC file
```bash
cd source/busmaster-rust
cargo run --bin busmaster -- monitor --driver stub --dbc examples/test.dbc --signals
```

### 2. Send messages that match the DBC
```bash
# Send EngineData (ID 0x100 = 256)
# Engine speed = 2000 rpm, Temp = 90°C
cargo run --bin busmaster -- send --id 0x100 --data "40 1F 82 00 00 00 00 00"

# Send VehicleSpeed (ID 0x200 = 512)
# Speed = 100 km/h
cargo run --bin busmaster -- send --id 0x200 --data "10 27 00 00 00 00 00 00"

# Send BrakeStatus (ID 0x300 = 768)
# Pressure = 50 bar, ABS active
cargo run --bin busmaster -- send --id 0x300 --data "F4 01 01 00"
```

---

## Test with Filtering

### Filter by ID range
```bash
cargo run --bin busmaster -- monitor --driver stub --filter-range 0x100-0x1FF
```

Now only messages with IDs 0x100-0x1FF will be displayed.

### Filter by ID list
```bash
cargo run --bin busmaster -- monitor --driver stub --filter-ids 0x100,0x200,0x300
```

Only messages with IDs 0x100, 0x200, or 0x300 will be displayed.

---

## Test with Logging

### Enable ASC logging
```bash
cargo run --bin busmaster -- monitor --driver stub --log /tmp/test.asc --max-messages 10
```

Then send some messages and check the log file:
```bash
cat /tmp/test.asc
```

You should see Vector ASC format output!

---

## Complete Example

Here's a complete workflow:

```bash
cd source/busmaster-rust

# Terminal 1: Monitor with all features
cargo run --bin busmaster -- monitor \
  --driver stub \
  --dbc examples/test.dbc \
  --log /tmp/session.asc \
  --filter-range 0x100-0x300 \
  --signals \
  --max-messages 20

# Terminal 2: Send test messages
cargo run --bin busmaster -- send --id 0x100 --data "40 1F 82 00 00 00 00 00"
cargo run --bin busmaster -- send --id 0x200 --data "10 27 00 00 00 00 00 00"
cargo run --bin busmaster -- send --id 0x300 --data "F4 01 01 00"

# Check the log file
cat /tmp/session.asc
```

---

## Troubleshooting

### "command not found: cargo"
Make sure Rust is installed and cargo is in your PATH:
```bash
~/.cargo/bin/cargo --version
```

### "package not found"
Make sure you're in the correct directory:
```bash
cd source/busmaster-rust
```

### "DBC file not found"
Use the full path or relative path from the workspace root:
```bash
cargo run --bin busmaster -- monitor --dbc examples/test.dbc
```

---

## What's Working

✅ **Commands:**
- `monitor` - Real-time CAN bus monitoring
- `send` - Send CAN messages
- `list` - List available drivers

✅ **Features:**
- Stub driver (no hardware needed)
- DBC database loading
- ASC logging
- Message filtering (ID range, ID list)
- Signal extraction
- Real-time display
- Graceful shutdown (Ctrl+C)

✅ **Options:**
- `--driver` - Select driver
- `--dbc` - Load database
- `--log` - Enable logging
- `--filter-range` - Filter by ID range
- `--filter-ids` - Filter by ID list
- `--signals` - Enable signal extraction
- `--max-messages` - Limit message count
- `--verbose` - Debug logging

---

## What's Next

The CLI is fully functional! Next steps:

1. **Task 3.3: TUI (Terminal UI)** - Interactive terminal interface
2. **Task 3.4: PEAK Driver** - Real hardware support
3. **Task 3.5: Platform Layer** - macOS platform support
4. **Task 3.6: MVP Integration** - End-to-end testing

---

## Need Help?

Check the documentation:
- `crates/busmaster-cli/README.md` - Full user guide
- `crates/busmaster-cli/QA_TEST_RESULTS.md` - Test results
- `docs/CLI_IMPLEMENTATION.md` - Implementation details

Or run:
```bash
cargo run --bin busmaster -- --help
cargo run --bin busmaster -- monitor --help
cargo run --bin busmaster -- send --help
```

---

**Enjoy testing!** 🚀
