# BUSMASTER TUI

Interactive Terminal User Interface for BUSMASTER - Automotive Bus Monitor

## Features

- 📊 **Real-time message display** - See CAN messages as they arrive
- ⌨️ **Keyboard navigation** - Full keyboard control
- 📈 **Statistics** - Message rate and count
- 🎨 **Highlighted selection** - Easy to see selected message
- 📜 **Message history** - Last 1000 messages buffered
- 🔍 **Message details** - View full message data
- ❓ **Built-in help** - Press 'h' for keyboard shortcuts

## Installation

### Run from source
```bash
cd source/busmaster-rust
cargo run --package busmaster-tui
```

### Build release binary
```bash
cd source/busmaster-rust
cargo build --release --package busmaster-tui
# Binary will be at: target/release/busmaster-tui
```

## Usage

### Start the TUI (Normal Mode)
```bash
cargo run --package busmaster-tui
```

The TUI will start immediately and begin monitoring CAN bus traffic with the stub driver.

### Start the TUI (Test Mode) ⭐ Recommended for Testing

Test mode generates random CAN messages automatically - perfect for testing without hardware!

```bash
cargo run --package busmaster-tui -- --test
```

This will:
- Generate a new CAN message every 500ms
- Display messages in real-time
- Let you test all TUI features (scrolling, help, clear)
- Show statistics updating in real-time

**This is the easiest way to see the TUI in action!**

## Keyboard Shortcuts

### Navigation
- **↑** or **k** - Scroll up
- **↓** or **j** - Scroll down
- **PgUp** - Page up (10 messages)
- **PgDn** - Page down (10 messages)
- **Home** - Go to first message
- **End** - Go to last message

### Actions
- **c** - Clear all messages
- **h** or **F1** - Toggle help screen
- **q** or **Esc** - Quit
- **Ctrl+C** - Force quit

## Interface Layout

```
┌─────────────────────────────────────────────────────────────┐
│ BUSMASTER - CAN Bus Monitor                                 │
├─────────────────────────────────────────────────────────────┤
│ Time         Ch   ID         DLC Data                       │
│ ────────────────────────────────────────────────────────────│
│ 1234.567     0    100        4   11 22 33 44               │
│ 1235.123     0    200        8   AA BB CC DD EE FF 00 11   │
│ 1235.789     0    300        4   01 02 03 04               │
│ ...                                                          │
├─────────────────────────────────────────────────────────────┤
│ Statistics                                                   │
│ Total: 1234 | Rate: 45.6 msg/s | Selected: 1/1234          │
├─────────────────────────────────────────────────────────────┤
│ Status                                                       │
│ Monitoring started | Press 'h' for help, 'q' to quit       │
└─────────────────────────────────────────────────────────────┘
```

## Features in Detail

### Message List View
- Shows last 1000 messages
- Columns: Time, Channel, ID, DLC, Data
- Selected message is highlighted
- Scrollable with keyboard
- Auto-scrolls to new messages

### Statistics Panel
- **Total** - Total messages received
- **Rate** - Messages per second
- **Selected** - Current selection position

### Status Bar
- Shows current status
- Displays keyboard shortcuts
- Shows error messages

### Help Screen
- Press **h** or **F1** to toggle
- Shows all keyboard shortcuts
- Shows current configuration
- Press **h** or **F1** again to return

## Testing

### Test with Test Mode ⭐ Recommended

The easiest way to test the TUI is with test mode:

```bash
cargo run --package busmaster-tui -- --test
```

This generates random messages automatically. Try:
- Press **↓** to scroll down
- Press **↑** to scroll up
- Press **PgDn** to page down
- Press **PgUp** to page up
- Press **Home** to go to first message
- Press **End** to go to last message
- Press **c** to clear messages
- Press **h** to see help
- Press **q** to quit

### Test with Stub Driver

**Note:** The stub driver uses loopback mode, so each process (CLI/TUI) is isolated. Messages sent by CLI won't appear in TUI because they're separate processes. Use test mode instead!

If you want to test the CLI separately:

**Terminal 1 - Start TUI:**
```bash
cargo run --package busmaster-tui
```

**Terminal 2 - Send test messages (won't appear in TUI):**
```bash
# These messages will only loop back to the CLI process, not the TUI
cargo run --bin busmaster -- send --id 0x100 --data "11 22 33 44"
```

For CLI-TUI communication, wait for the PEAK driver (Task 3.4) which uses real CAN hardware.

## Logging

The TUI logs to `/tmp/busmaster-tui.log` for debugging.

View logs:
```bash
tail -f /tmp/busmaster-tui.log
```

## Troubleshooting

### Terminal not rendering correctly

Make sure your terminal supports:
- ANSI colors
- UTF-8 encoding
- Alternate screen buffer

Recommended terminals:
- iTerm2 (macOS)
- Terminal.app (macOS)
- Alacritty
- Kitty
- GNOME Terminal (Linux)
- Windows Terminal (Windows)

### TUI exits immediately

Check the log file:
```bash
cat /tmp/busmaster-tui.log
```

### No messages appearing

1. Make sure the engine is running (check status bar)
2. Try sending messages from another terminal
3. Check that the stub driver is working:
   ```bash
   cargo run --bin busmaster -- send --id 0x123 --data "01 02 03 04"
   ```

### Keyboard shortcuts not working

- Make sure your terminal is in focus
- Try pressing the key combinations again
- Check if your terminal is intercepting the keys

## Performance

- **Startup time:** < 1 second
- **Memory usage:** < 15MB
- **Message display:** Real-time, no lag
- **Max messages:** 1000 (auto-scrolling buffer)

## Comparison with CLI

| Feature | CLI | TUI |
|---------|-----|-----|
| Real-time display | ✅ | ✅ |
| Message history | ❌ | ✅ (1000 msgs) |
| Keyboard navigation | ❌ | ✅ |
| Message selection | ❌ | ✅ |
| Statistics | ✅ | ✅ |
| Filtering | ✅ | ⏳ (coming soon) |
| DBC loading | ✅ | ⏳ (coming soon) |
| Logging | ✅ | ⏳ (coming soon) |
| Interactive | ❌ | ✅ |
| Scriptable | ✅ | ❌ |

## Future Enhancements

- [ ] DBC database loading
- [ ] Signal value display
- [ ] Message filtering dialog
- [ ] ASC logging
- [ ] Multiple views (split screen)
- [ ] Message search
- [ ] Export to file
- [ ] Color coding by message type
- [ ] Graphical signal display

## See Also

- [CLI Documentation](../busmaster-cli/README.md)
- [Engine Documentation](../busmaster-engine/README.md)
- [Quick Start Guide](../../QUICKSTART_CLI.md)
