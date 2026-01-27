# BUSMASTER TUI - Quick Start Guide

**Interactive Terminal UI is ready!** 🎉

---

## What is the TUI?

The TUI (Terminal User Interface) is an **interactive** version of the CLI that runs in your terminal with:
- 📊 Real-time message display
- ⌨️ Keyboard navigation
- 📈 Live statistics
- 🎨 Highlighted selection
- 📜 Message history (1000 messages)
- ❓ Built-in help

---

## Quick Start (Test Mode) ⭐ Recommended

### The Easiest Way to Try the TUI

```bash
cd source/busmaster-rust
cargo run --package busmaster-tui -- --test
```

This starts the TUI in **test mode** which:
- ✅ Generates random CAN messages automatically (every 500ms)
- ✅ No need for a second terminal
- ✅ Perfect for testing all features
- ✅ See messages, statistics, and navigation in action

**Try it now!** Then press:
- **↑/↓** to scroll through messages
- **h** to see help
- **c** to clear messages
- **q** to quit

---

## Alternative: Normal Mode (Two Terminals)

If you want to send messages manually:

### 1. Start the TUI (Terminal 1)
```bash
cd source/busmaster-rust
cargo run --package busmaster-tui
```

The TUI will start immediately and show:
```
┌─────────────────────────────────────────────────────────────┐
│ BUSMASTER - CAN Bus Monitor                                 │
├─────────────────────────────────────────────────────────────┤
│ Time         Ch   ID         DLC Data                       │
│ ────────────────────────────────────────────────────────────│
│ (waiting for messages...)                                   │
├─────────────────────────────────────────────────────────────┤
│ Statistics                                                   │
│ Total: 0 | Rate: 0.0 msg/s | Selected: 0/0                 │
├─────────────────────────────────────────────────────────────┤
│ Status                                                       │
│ Monitoring started | Press 'h' for help, 'q' to quit       │
└─────────────────────────────────────────────────────────────┘
```

### 2. Important Note About Stub Driver

⚠️ **The stub driver uses loopback mode** - each process (CLI/TUI) is isolated. Messages sent by CLI won't appear in TUI because they're separate processes.

**Solution:** Use test mode (see above) or wait for the PEAK driver (Task 3.4) which uses real CAN hardware.

For more details, see `DEMO_INSTRUCTIONS.md`.

---

## Keyboard Shortcuts

### Navigation
- **↑** or **k** - Scroll up one message
- **↓** or **j** - Scroll down one message
- **PgUp** - Page up (10 messages)
- **PgDn** - Page down (10 messages)
- **Home** - Jump to first message
- **End** - Jump to last message

### Actions
- **c** - Clear all messages
- **h** or **F1** - Toggle help screen
- **q** or **Esc** - Quit
- **Ctrl+C** - Force quit

---

## What You Can Do

### ✅ Monitor CAN Bus
- Real-time message display
- See messages as they arrive
- Scroll through history

### ✅ Navigate Messages
- Use arrow keys or vim keys (j/k)
- Page up/down for fast scrolling
- Jump to start/end

### ✅ View Statistics
- Total messages received
- Messages per second
- Current selection position

### ✅ Clear Messages
- Press **c** to clear the buffer
- Start fresh anytime

### ✅ Get Help
- Press **h** to see all shortcuts
- Press **h** again to return

---

## Comparison: CLI vs TUI

| Feature | CLI | TUI |
|---------|-----|-----|
| **Interactive** | ❌ | ✅ |
| **Message History** | ❌ | ✅ (1000 msgs) |
| **Keyboard Navigation** | ❌ | ✅ |
| **Message Selection** | ❌ | ✅ |
| **Real-time Display** | ✅ | ✅ |
| **Statistics** | ✅ | ✅ |
| **Scriptable** | ✅ | ❌ |

**Use CLI for:** Scripting, automation, logging  
**Use TUI for:** Interactive monitoring, debugging, exploration

---

## Tips

1. **Use two terminals** - One for TUI, one for sending messages
2. **Try vim keys** - If you know vim, use j/k for scrolling
3. **Clear often** - Press 'c' to clear old messages
4. **Check help** - Press 'h' to see all shortcuts
5. **Watch statistics** - See message rate in real-time

---

## Troubleshooting

### TUI doesn't start
```bash
# Check if it compiles
cd source/busmaster-rust
cargo build --package busmaster-tui

# Check the log
cat /tmp/busmaster-tui.log
```

### No messages appearing
1. Make sure you're sending messages from another terminal
2. Check that the stub driver is working:
   ```bash
   cargo run --bin busmaster -- send --id 0x123 --data "01 02 03 04"
   ```

### Terminal looks weird
- Make sure your terminal supports ANSI colors
- Try a different terminal (iTerm2, Alacritty, etc.)
- Resize your terminal window

### Keyboard shortcuts don't work
- Make sure the TUI terminal is in focus
- Try pressing the keys again
- Check if your terminal is intercepting the keys

---

## Next Steps

Now that you have both CLI and TUI working:

1. **Test the CLI** - See `QUICKSTART_CLI.md`
2. **Test the TUI** - You're here! ✅
3. **Compare them** - Try both and see which you prefer
4. **Send feedback** - What features would you like?

---

## What's Next in Development

**Completed:**
- ✅ Task 3.1: Engine
- ✅ Task 3.2: CLI Application
- ✅ Task 3.3: TUI Application

**Coming Soon:**
- ⏳ Task 3.4: PEAK Driver (real hardware support)
- ⏳ Task 3.5: Platform Layer (macOS platform)
- ⏳ Task 3.6: MVP Integration & Testing

---

**Enjoy the interactive TUI!** 🚀

Press **h** in the TUI to see all keyboard shortcuts!
