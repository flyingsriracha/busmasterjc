# BUSMASTER Testing Guide

**Welcome!** This guide will walk you through testing both the CLI and TUI applications.

---

## Prerequisites

Make sure you're in the correct directory:
```bash
cd source/busmaster-rust
```

---

## Test 1: CLI Basic Commands (5 minutes)

### Step 1: Check CLI Help
```bash
cargo run --bin busmaster -- --help
```

**Expected output:**
```
BUSMASTER - Automotive Bus Monitor

Usage: busmaster [OPTIONS] [COMMAND]

Commands:
  monitor  Monitor CAN bus traffic
  send     Send a CAN message
  list     List available hardware
  help     Print this message or the help of the given subcommand(s)
```

✅ **Success:** Help displays correctly

---

### Step 2: List Available Drivers
```bash
cargo run --bin busmaster -- list
```

**Expected output:**
```
Available drivers:

  stub    - Virtual CAN device (loopback)
            Always available for testing

  peak    - PEAK USB/PCIe devices
            Requires PCAN hardware (not yet implemented)
```

✅ **Success:** Drivers listed

---

### Step 3: Send a Test Message
```bash
cargo run --bin busmaster -- send --id 0x123 --data "01 02 03 04"
```

**Expected output:**
```
✓ Sent: ID=0x123 DLC=4 Data=01 02 03 04
```

✅ **Success:** Message sent

---

## Test 2: CLI with DBC Database (5 minutes)

### Step 1: Check the Sample DBC File
```bash
cat examples/test.dbc | head -20
```

You should see a DBC file with EngineData, VehicleSpeed, and BrakeStatus messages.

---

### Step 2: Monitor with DBC (Terminal 1)
```bash
cargo run --bin busmaster -- monitor --driver stub --dbc examples/test.dbc --max-messages 10
```

**Expected output:**
```
✓ Loaded database: "examples/test.dbc"
✓ Monitoring started (Ctrl+C to stop)

Time         Ch   ID         DLC Data
------------------------------------------------------------
```

**Keep this terminal open!**

---

### Step 3: Send Messages (Terminal 2)

Open a **new terminal** and run:

```bash
cd source/busmaster-rust

# Send EngineData (ID 0x100 = 256)
cargo run --bin busmaster -- send --id 0x100 --data "40 1F 82 00 00 00 00 00"

# Send VehicleSpeed (ID 0x200 = 512)
cargo run --bin busmaster -- send --id 0x200 --data "10 27 00 00 00 00 00 00"

# Send BrakeStatus (ID 0x300 = 768)
cargo run --bin busmaster -- send --id 0x300 --data "F4 01 01 00"
```

**Check Terminal 1** - You should see the messages appear!

✅ **Success:** Messages displayed in real-time

---

## Test 3: CLI with Filtering (5 minutes)

### Step 1: Monitor with ID Range Filter (Terminal 1)
```bash
cargo run --bin busmaster -- monitor --driver stub --filter-range 0x100-0x1FF --max-messages 10
```

**Expected output:**
```
✓ Filter: ID range 0x100-0x1FF
✓ Monitoring started (Ctrl+C to stop)
```

---

### Step 2: Send Mixed Messages (Terminal 2)
```bash
cd source/busmaster-rust

# This should appear (in range)
cargo run --bin busmaster -- send --id 0x100 --data "11 22 33 44"

# This should appear (in range)
cargo run --bin busmaster -- send --id 0x150 --data "AA BB CC DD"

# This should NOT appear (out of range)
cargo run --bin busmaster -- send --id 0x200 --data "FF FF FF FF"

# This should appear (in range)
cargo run --bin busmaster -- send --id 0x1FF --data "00 11 22 33"
```

**Check Terminal 1** - Only messages with IDs 0x100-0x1FF should appear!

✅ **Success:** Filter working correctly

---

## Test 4: CLI with Logging (5 minutes)

### Step 1: Monitor with Logging
```bash
cargo run --bin busmaster -- monitor --driver stub --log /tmp/test.asc --max-messages 5
```

**Expected output:**
```
✓ Logging to: "/tmp/test.asc"
✓ Monitoring started (Ctrl+C to stop)
```

---

### Step 2: Send Some Messages (Terminal 2)
```bash
cd source/busmaster-rust

cargo run --bin busmaster -- send --id 0x100 --data "11 22 33 44"
cargo run --bin busmaster -- send --id 0x200 --data "AA BB CC DD"
cargo run --bin busmaster -- send --id 0x300 --data "01 02 03 04"
```

---

### Step 3: Check the Log File
```bash
cat /tmp/test.asc
```

**Expected output:** Vector ASC format with header and messages

✅ **Success:** Logging works

---

## Test 5: TUI Interactive Mode (10 minutes)

### Step 1: Start the TUI
```bash
cargo run --package busmaster-tui
```

**Expected:** Full-screen terminal interface appears with:
- Title bar: "BUSMASTER - CAN Bus Monitor"
- Message list (empty)
- Statistics: "Total: 0 | Rate: 0.0 msg/s"
- Status: "Monitoring started | Press 'h' for help, 'q' to quit"

✅ **Success:** TUI started

---

### Step 2: View Help Screen

**In the TUI terminal:**
- Press **h** (or F1)

**Expected:** Help screen appears with keyboard shortcuts

- Press **h** again to return to message list

✅ **Success:** Help toggle works

---

### Step 3: Send Messages and Watch (Terminal 2)

**Keep TUI running in Terminal 1**

Open Terminal 2:
```bash
cd source/busmaster-rust

# Send a few messages
cargo run --bin busmaster -- send --id 0x100 --data "11 22 33 44"
cargo run --bin busmaster -- send --id 0x200 --data "AA BB CC DD"
cargo run --bin busmaster -- send --id 0x300 --data "01 02 03 04 05 06 07 08"
```

**Watch Terminal 1 (TUI):**
- Messages should appear in the table
- Statistics should update (Total: 3)
- Timestamps should be displayed

✅ **Success:** Real-time updates work

---

### Step 4: Test Keyboard Navigation

**In the TUI terminal:**

1. Press **↓** (down arrow) - Selection should move down
2. Press **↑** (up arrow) - Selection should move up
3. Press **End** - Jump to last message
4. Press **Home** - Jump to first message
5. Press **c** - Clear all messages
6. Status should say "Messages cleared"

✅ **Success:** Keyboard navigation works

---

### Step 5: Test Burst Mode (Terminal 2)

Send a burst of messages:
```bash
cd source/busmaster-rust

for i in {1..50}; do
  cargo run --bin busmaster -- send --id 0x$i --data "00 00 00 $i"
done
```

**Watch Terminal 1 (TUI):**
- Messages should appear rapidly
- Statistics should update (Total: 50+)
- Message rate should show (msg/s)
- You can scroll through the history

✅ **Success:** Burst handling works

---

### Step 6: Test Scrolling

**In the TUI terminal:**

1. Press **PgDn** (Page Down) - Jump 10 messages down
2. Press **PgUp** (Page Up) - Jump 10 messages up
3. Try **j** and **k** (vim-style) - Should scroll like arrows

✅ **Success:** All navigation methods work

---

### Step 7: Quit the TUI

**In the TUI terminal:**
- Press **q** (or Esc, or Ctrl+C)

**Expected:** TUI exits cleanly, terminal restored

✅ **Success:** Clean exit

---

## Test 6: Side-by-Side Comparison (5 minutes)

### Setup: 3 Terminals

**Terminal 1 - CLI Monitor:**
```bash
cd source/busmaster-rust
cargo run --bin busmaster -- monitor --driver stub
```

**Terminal 2 - TUI Monitor:**
```bash
cd source/busmaster-rust
cargo run --package busmaster-tui
```

**Terminal 3 - Sender:**
```bash
cd source/busmaster-rust

# Send messages and watch both monitors
for i in {1..20}; do
  cargo run --bin busmaster -- send --id 0x$i --data "00 00 00 $i"
  sleep 0.1
done
```

**Compare:**
- Both should show the same messages
- CLI shows continuous stream
- TUI shows interactive table with selection
- TUI has statistics panel
- TUI allows scrolling back

✅ **Success:** Both work identically

---

## Test 7: Performance Test (5 minutes)

### Test: Rapid Message Sending

**Terminal 1 - TUI:**
```bash
cargo run --package busmaster-tui
```

**Terminal 2 - Rapid Sender:**
```bash
cd source/busmaster-rust

# Send 100 messages as fast as possible
for i in {1..100}; do
  cargo run --bin busmaster -- send --id 0x$i --data "00 00 00 $i" &
done
wait
```

**Watch Terminal 1:**
- All messages should appear
- No lag or freezing
- Statistics should update
- Message rate should be high

✅ **Success:** Handles high message rates

---

## Test 8: Error Handling (3 minutes)

### Test 1: Invalid Driver
```bash
cargo run --bin busmaster -- monitor --driver invalid
```

**Expected:** Error message: "Unknown driver: invalid"

✅ **Success:** Error handled

---

### Test 2: Invalid ID Format
```bash
cargo run --bin busmaster -- send --id INVALID --data "01 02"
```

**Expected:** Error message about invalid format

✅ **Success:** Error handled

---

### Test 3: Missing Required Argument
```bash
cargo run --bin busmaster -- send --id 0x123
```

**Expected:** Error message about missing --data

✅ **Success:** Error handled

---

## Summary Checklist

After completing all tests, you should have verified:

### CLI Features
- [ ] Help command works
- [ ] List command works
- [ ] Send command works
- [ ] Monitor command works
- [ ] DBC loading works
- [ ] Filtering works (ID range)
- [ ] Logging works (ASC format)
- [ ] Error handling works

### TUI Features
- [ ] TUI starts successfully
- [ ] Message display works
- [ ] Real-time updates work
- [ ] Keyboard navigation works (arrows, vim keys)
- [ ] Page up/down works
- [ ] Home/End works
- [ ] Clear messages works
- [ ] Help screen works
- [ ] Statistics update
- [ ] Clean exit works

### Integration
- [ ] CLI and TUI work together
- [ ] Messages sent via CLI appear in TUI
- [ ] Both handle high message rates
- [ ] Error handling works in both

---

## Troubleshooting

### Issue: "command not found: cargo"
**Solution:** Use the full path:
```bash
~/.cargo/bin/cargo run --bin busmaster -- --help
```

### Issue: TUI doesn't render correctly
**Solution:** 
- Try a different terminal (iTerm2, Alacritty)
- Make sure terminal supports ANSI colors
- Resize terminal window

### Issue: No messages appearing
**Solution:**
- Make sure you're sending from a different terminal
- Check that the stub driver is working
- Verify the engine is running (check status bar)

### Issue: Build errors
**Solution:**
```bash
cd source/busmaster-rust
~/.cargo/bin/cargo clean
~/.cargo/bin/cargo build --package busmaster-cli --package busmaster-tui
```

---

## Next Steps

After testing:

1. **Provide feedback** - What works? What doesn't?
2. **Try your own scenarios** - Create custom test cases
3. **Check documentation** - Read the READMEs for more features
4. **Report issues** - Let me know if anything doesn't work

---

## Quick Reference

### CLI Commands
```bash
# Help
cargo run --bin busmaster -- --help

# List drivers
cargo run --bin busmaster -- list

# Send message
cargo run --bin busmaster -- send --id 0x123 --data "01 02 03 04"

# Monitor
cargo run --bin busmaster -- monitor --driver stub

# Monitor with DBC
cargo run --bin busmaster -- monitor --dbc examples/test.dbc

# Monitor with filter
cargo run --bin busmaster -- monitor --filter-range 0x100-0x1FF

# Monitor with logging
cargo run --bin busmaster -- monitor --log /tmp/test.asc
```

### TUI Commands
```bash
# Start TUI
cargo run --package busmaster-tui

# Keyboard shortcuts (in TUI):
# h - Help
# ↑↓ or jk - Scroll
# PgUp/PgDn - Page
# Home/End - Jump
# c - Clear
# q - Quit
```

---

**Happy Testing!** 🚀

If you encounter any issues, check the logs:
- CLI: Uses stdout/stderr
- TUI: Logs to `/tmp/busmaster-tui.log`
