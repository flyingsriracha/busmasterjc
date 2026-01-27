# Demo Instructions - TUI Test Mode

## ✅ SOLUTION: Use Test Mode

The TUI now has a **test mode** that generates random CAN messages automatically!

```bash
cd source/busmaster-rust
cargo run --package busmaster-tui -- --test
```

This will:
- Generate a new CAN message every 500ms
- Display messages in real-time
- Show statistics (message rate, total count)
- Let you test all TUI features (scrolling, help, clear)

**Keyboard shortcuts:**
- `↑/↓` or `j/k` - Scroll through messages
- `PgUp/PgDn` - Page up/down
- `Home/End` - Jump to first/last message
- `c` - Clear messages
- `h` or `F1` - Toggle help screen
- `q` or `Esc` - Quit

---

## Why CLI and TUI Don't Communicate

You may have noticed that when you send messages from the CLI, they don't appear in the TUI. This is **expected behavior** with the current stub driver!

### The Technical Reason

The **stub driver** works in **loopback mode**, which means:
- Each process (CLI or TUI) has its own driver instance
- Messages sent by one process only loop back to **that same process**
- They don't go to other processes

Think of it like this:
```
CLI Process          TUI Process
┌─────────┐         ┌─────────┐
│ Driver  │         │ Driver  │
│ (stub)  │         │ (stub)  │
└─────────┘         └─────────┘
     ↓↑                  ↓↑
  Loopback            Loopback
  (internal)          (internal)
```

Messages sent in CLI loop back to CLI only.
Messages sent in TUI loop back to TUI only.


### When Will CLI and TUI Communicate?

When we implement the **PEAK driver** (Task 3.4), it will use **real CAN hardware** where:
- Messages sent by CLI go on the physical CAN bus
- TUI receives them from the physical CAN bus
- They work together perfectly!

```
CLI Process          Physical CAN Bus          TUI Process
┌─────────┐         ┌──────────────┐         ┌─────────┐
│ PEAK    │────────▶│   CAN Bus    │────────▶│ PEAK    │
│ Driver  │         │  (Hardware)  │         │ Driver  │
└─────────┘         └──────────────┘         └─────────┘
```

---

## What You Can Test Now

### 1. ✅ TUI Test Mode (Recommended)

The easiest way to see the TUI in action:

```bash
cd source/busmaster-rust
cargo run --package busmaster-tui -- --test
```

This generates random messages automatically so you can test:
- Message display
- Scrolling and navigation
- Statistics
- Help screen
- Clear function

### 2. CLI Features (Separately)

Test CLI features independently:

```bash
# Send a message (loopback to same process)
cargo run --bin busmaster -- send --id 0x123 --data "01 02 03 04"

# Monitor messages (only sees its own in loopback)
cargo run --bin busmaster -- monitor --driver stub --max-messages 5
```

### 3. Wait for Real Hardware

Once we implement the PEAK driver (Task 3.4), CLI and TUI will communicate through real CAN hardware.

---

## Summary

- **Test Mode**: Use `--test` flag to see TUI with generated messages ✅
- **Stub Driver**: Each process is isolated (by design)
- **Real Hardware**: Coming in Task 3.4 (PEAK driver)

The test mode is the best way to explore the TUI right now!
