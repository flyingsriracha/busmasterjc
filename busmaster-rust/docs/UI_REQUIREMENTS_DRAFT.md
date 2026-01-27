# BUSMASTER Rust - UI/UX Requirements Draft

**Status:** DRAFT - Evolving as features are implemented  
**Last Updated:** January 26, 2026  
**Target:** Rival CANoe, CANape, ETAS INCA

---

## Design Philosophy

### Core Principles
1. **Professional but Approachable** - Automotive engineers need power, but shouldn't need a PhD to use it
2. **Information Density** - Show lots of data without overwhelming (think Bloomberg Terminal meets modern design)
3. **Workflow-Centric** - Organize around what users DO, not what the software CAN do
4. **Real-time First** - Everything updates live, no refresh buttons
5. **Dark Mode Default** - Engineers work long hours, be kind to their eyes

### Target Users
- Automotive test engineers
- ECU calibration engineers  
- Vehicle diagnostics technicians
- Embedded software developers
- Quality assurance engineers

---

## Main Application Layout

### Window Structure (Dockable Panels)
```
┌─────────────────────────────────────────────────────────────────────┐
│  Menu Bar  │  Toolbar (Connect/Disconnect, Start/Stop, Quick Actions) │
├────────────┼────────────────────────────────────────────────────────┤
│            │                                                         │
│  Project   │              Main Content Area                          │
│  Explorer  │         (Message View / Signal Graph /                  │
│            │          Diagnostics / Calibration)                     │
│  ─────────│                                                         │
│            │                                                         │
│  Database  │                                                         │
│  Browser   │                                                         │
│            │                                                         │
│  ─────────│─────────────────────────────────────────────────────────│
│            │              Signal Watch / Properties                  │
│  ECU       │                                                         │
│  Explorer  │                                                         │
│            │                                                         │
├────────────┴────────────────────────────────────────────────────────┤
│  Status Bar: Connection Status │ Bus Load │ Message Rate │ Errors   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Feature-Specific UI Requirements

### 1. Database Browser (DBC/DBF/LDF/ARXML/ODX/A2L)

**Requirements from Parser Implementation:**
- Tree view showing database hierarchy
- For DBC/DBF: Messages → Signals
- For ARXML: Packages → Elements (I-Signals, PDUs, Frames, Clusters)
- For LDF: Nodes → Frames → Signals, Schedule Tables
- For ODX: DiagLayers → Services → Parameters, DTCs
- For A2L: Modules → Measurements/Characteristics/Functions

**UI Elements:**
- [ ] Collapsible tree with icons per element type
- [ ] Quick search/filter across all loaded databases
- [ ] Drag-drop signals to Signal Watch panel
- [ ] Right-click context menu: "Add to Watch", "Show in Graph", "Copy Path"
- [ ] Tooltip on hover showing full signal details (factor, offset, unit, range)
- [ ] Multi-database support (load multiple DBC files simultaneously)

**Visual Design:**
- Color-coded icons: 📦 Package, 📨 Message/Frame, 📊 Signal, 🔧 Parameter
- Grayed out items that aren't currently active on the bus
- Bold items that have been recently updated

---

### 2. Message View (Real-time CAN/LIN/Ethernet Traffic)

**Requirements from Protocol Implementation:**
- Support CAN, CAN FD, LIN, J1939, DoIP, SOME/IP
- Show raw and decoded views
- Filtering by ID, direction, channel

**UI Elements:**
- [ ] Virtual scrolling table (handle 100k+ messages)
- [ ] Columns: Timestamp, Channel, ID, Name, DLC, Data, Direction
- [ ] Expandable rows showing decoded signals
- [ ] Color coding: TX=blue, RX=green, Error=red
- [ ] Freeze/unfreeze scrolling
- [ ] Quick filter bar at top
- [ ] Column customization (show/hide, reorder)

**Advanced Features:**
- [ ] Message highlighting rules (user-defined)
- [ ] Delta time display (time since last occurrence)
- [ ] Cycle time monitoring with warnings
- [ ] Bit-level data visualization

---

### 3. Signal Watch Panel

**Requirements from Signal Extraction:**
- Real-time signal value display
- Physical value with unit
- Raw value option
- Min/Max tracking

**UI Elements:**
- [ ] Compact list view with signal name, value, unit
- [ ] Sparkline mini-graphs inline
- [ ] Value change highlighting (flash on update)
- [ ] Grouping by message or custom groups
- [ ] Quick graph button per signal
- [ ] Edit value (for TX signals)

**Layout Options:**
- List view (compact)
- Grid view (larger, with gauges)
- Dashboard view (customizable widgets)

---

### 4. Signal Graphing

**Requirements from Data Types:**
- Time-series plotting
- Multiple signals on same graph
- Zoom/pan
- Cursors for measurement

**UI Elements:**
- [ ] Multi-axis support (left/right Y axes)
- [ ] Legend with show/hide toggles
- [ ] Time range selector (last 10s, 1min, 5min, custom)
- [ ] Cursor tool with delta measurement
- [ ] Export to PNG/CSV
- [ ] Trigger/marker support

**Performance:**
- Handle 100+ signals at 1ms resolution
- GPU-accelerated rendering (egui + wgpu)

---

### 5. Diagnostics Panel (UDS/OBD-II/KWP2000)

**Requirements from UDS/OBD Implementation:**
- Service execution
- DTC reading/clearing
- Data identifier read/write
- Security access
- Routine control

**UI Elements:**
- [ ] Service browser (tree of available services)
- [ ] Request builder with parameter inputs
- [ ] Response viewer with decoded values
- [ ] DTC list with severity icons and descriptions
- [ ] Freeze frame data viewer
- [ ] Session/security state indicator

**Workflow:**
- One-click "Read All DTCs"
- Guided security unlock flow
- Service sequence recording/playback

---

### 6. ECU Explorer (Auto-Detection)

**Requirements from ECU Database:**
- Show detected ECUs
- Display manufacturer, family, variant
- Link to A2L files
- AI-assisted identification

**UI Elements:**
- [ ] Card view of detected ECUs
- [ ] ECU icon based on type (Engine, Transmission, ABS, etc.)
- [ ] Confidence indicator for AI detection
- [ ] "Identify" button to trigger scan
- [ ] Link to associated A2L/ODX files
- [ ] VIN decoder display

---

### 7. Calibration View (CANape/INCA Rival)

**Requirements from Calibration Module:**
- Measurement display
- Parameter editing
- Memory page management
- Dataset comparison
- Flash programming

**UI Elements:**
- [ ] Measurement table with live values
- [ ] Parameter editor with validation
- [ ] Curve/Map 2D/3D editors
- [ ] Working page vs Reference page comparison
- [ ] Flash progress dialog
- [ ] Dataset manager (load/save/compare)

**Advanced:**
- [ ] A2L-driven automatic layout
- [ ] Characteristic group editing
- [ ] Hex file viewer/editor
- [ ] Checksum calculator

---

### 8. Logging Configuration

**Requirements from Log Module:**
- ASC, BLF, PCAP, MDF4 formats
- Trigger conditions
- File splitting

**UI Elements:**
- [ ] Format selector with options per format
- [ ] Trigger configuration (start/stop conditions)
- [ ] File path with auto-naming templates
- [ ] Size/time-based splitting options
- [ ] Recording indicator in status bar

---

### 9. Filter Configuration

**Requirements from Filter Module:**
- ID range, mask, list filters
- Direction filtering
- Channel filtering
- Combinable with AND/OR

**UI Elements:**
- [ ] Visual filter builder (no code required)
- [ ] Filter presets (save/load)
- [ ] Quick enable/disable toggles
- [ ] Filter statistics (passed/blocked counts)

---

## Cross-Cutting UI Concerns

### Keyboard Shortcuts
- `Ctrl+Space` - Connect/Disconnect
- `Ctrl+L` - Start/Stop Logging
- `Ctrl+F` - Focus filter bar
- `Ctrl+G` - Open signal graph
- `F5` - Refresh/Rescan
- `Esc` - Clear selection / Close dialog

### Theming
- Dark mode (default)
- Light mode
- High contrast mode
- Custom accent colors

### Accessibility
- Screen reader support
- Keyboard-only navigation
- Configurable font sizes
- Color-blind friendly palettes

### Performance Targets
- 60 FPS UI rendering
- <100ms response to user actions
- Handle 10,000 msg/sec display
- <500MB memory for typical session

---

## Platform-Specific Considerations

### macOS (Primary)
- Native menu bar integration
- Touch Bar support (if applicable)
- Retina display support
- System dark mode detection

### Windows (Future)
- Native title bar option
- System tray integration
- Jump list support

### Linux (Future)
- Wayland and X11 support
- System theme integration

---

## Competitive Analysis Notes

### CANoe Strengths to Match
- Comprehensive protocol support ✓ (building)
- Simulation capabilities (Phase 5)
- CAPL scripting (consider Lua alternative)

### CANape Strengths to Match
- A2L integration ✓ (done)
- XCP measurement ✓ (done)
- Calibration workflow (building)
- Curve/Map editors (needed)

### INCA Strengths to Match
- ECU flash programming (building)
- Dataset management (building)
- Measurement automation (needed)

### SavvyCAN Strengths to Match
- Reverse engineering tools (Phase 4)
- Signal discovery (Phase 4)
- Open source community

---

## Next Steps

1. Create wireframes for main window layout
2. Design database browser component
3. Design message view with virtual scrolling
4. Prototype signal graph with egui
5. User testing with automotive engineers

---

**Note:** This document will be updated as more features are implemented. Each parser and protocol implementation should consider how its data will be visualized in the GUI.
