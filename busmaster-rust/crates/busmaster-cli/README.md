# BUSMASTER CLI

Command-line interface for BUSMASTER - Automotive Bus Monitor

## Installation

```bash
cargo install --path .
```

Or run directly from source:

```bash
cargo run --bin busmaster -- [COMMAND]
```

## Usage

### List Available Drivers

```bash
busmaster list
```

Output:
```
Available drivers:

  stub    - Virtual CAN device (loopback)
            Always available for testing

  peak    - PEAK USB/PCIe devices
            Requires PCAN hardware (not yet implemented)

  vector  - Vector CANcaseXL/CANcardXL devices
            Requires Vector hardware (not yet implemented)
```

### Monitor CAN Bus Traffic

Basic monitoring with stub driver:

```bash
busmaster monitor --driver stub
```

Monitor with DBC database for signal decoding:

```bash
busmaster monitor --driver stub --dbc path/to/database.dbc
```

Monitor with logging to ASC file:

```bash
busmaster monitor --driver stub --log output.asc
```

Monitor with ID range filter (only show IDs 0x100-0x1FF):

```bash
busmaster monitor --driver stub --filter-range 0x100-0x1FF
```

Monitor with ID list filter:

```bash
busmaster monitor --driver stub --filter-ids 0x100,0x200,0x300
```

Monitor with signal value display:

```bash
busmaster monitor --driver stub --dbc database.dbc --signals
```

Monitor with maximum message count:

```bash
busmaster monitor --driver stub --max-messages 100
```

Complete example with all options:

```bash
busmaster monitor \
  --driver stub \
  --channel 0 \
  --baudrate 500000 \
  --dbc database.dbc \
  --log output.asc \
  --filter-range 0x100-0x1FF \
  --signals \
  --max-messages 1000
```

### Send CAN Messages

Send a standard ID message:

```bash
busmaster send --id 0x123 --data "01 02 03 04"
```

Send with comma-separated data:

```bash
busmaster send --id 0x123 --data "01,02,03,04"
```

Send an extended ID message:

```bash
busmaster send --id 0x12345678 --data "01 02 03 04" --extended
```

Send on specific channel:

```bash
busmaster send --channel 1 --id 0x123 --data "01 02 03 04"
```

Send with different driver:

```bash
busmaster send --driver peak --id 0x123 --data "01 02 03 04"
```

### Verbose Output

Enable verbose logging for debugging:

```bash
busmaster --verbose monitor --driver stub
```

## Examples

### Example 1: Basic CAN Monitoring

Monitor CAN traffic with the stub driver (no hardware required):

```bash
# Terminal 1: Start monitoring
busmaster monitor --driver stub

# Terminal 2: Send some messages
busmaster send --id 0x100 --data "11 22 33 44"
busmaster send --id 0x200 --data "AA BB CC DD"
busmaster send --id 0x300 --data "01 02 03 04 05 06 07 08"
```

Expected output in Terminal 1:
```
✓ Monitoring started (Ctrl+C to stop)

Time         Ch   ID         DLC Data
------------------------------------------------------------
    1234.567    0 0x100        4 11 22 33 44
    1235.123    0 0x200        4 AA BB CC DD
    1235.789    0 0x300        8 01 02 03 04 05 06 07 08
```

### Example 2: Filtered Monitoring with Logging

Monitor only specific IDs and log to file:

```bash
busmaster monitor \
  --driver stub \
  --filter-range 0x100-0x1FF \
  --log session.asc
```

This will:
- Only show messages with IDs between 0x100 and 0x1FF
- Log all matching messages to `session.asc` in Vector ASC format

### Example 3: Signal Decoding

Create a simple DBC file (`test.dbc`):

```dbc
VERSION ""

BU_: ECU1 ECU2

BO_ 256 EngineData: 8 ECU1
 SG_ EngineSpeed : 0|16@1+ (0.25,0) [0|16383.75] "rpm" ECU2
 SG_ EngineTemp : 16|8@1+ (1,-40) [-40|215] "degC" ECU2

BO_ 512 VehicleSpeed: 8 ECU1
 SG_ Speed : 0|16@1+ (0.01,0) [0|655.35] "km/h" ECU2
```

Monitor with signal decoding:

```bash
busmaster monitor --driver stub --dbc test.dbc --signals
```

Send test data:

```bash
# Engine speed = 2000 rpm (2000/0.25 = 8000 = 0x1F40)
# Engine temp = 90°C (90+40 = 130 = 0x82)
busmaster send --id 0x100 --data "40 1F 82 00 00 00 00 00"

# Vehicle speed = 100 km/h (100/0.01 = 10000 = 0x2710)
busmaster send --id 0x200 --data "10 27 00 00 00 00 00 00"
```

### Example 4: Testing with Loopback

The stub driver supports loopback mode for testing:

```bash
# Start monitoring in one terminal
busmaster monitor --driver stub --max-messages 10

# Send messages from another terminal
for i in {1..10}; do
  busmaster send --id 0x$i --data "00 00 00 $i"
  sleep 0.1
done
```

## Output Format

### Monitor Output

```
Time         Ch   ID         DLC Data
------------------------------------------------------------
    1234.567    0 0x100        4 11 22 33 44
    1235.123    0 0x200        8 AA BB CC DD EE FF 00 11
```

- **Time**: Timestamp in milliseconds
- **Ch**: Channel number
- **ID**: CAN message ID (0x000-0x7FF for standard, 0x00000000-0x1FFFFFFF for extended)
- **DLC**: Data Length Code (0-8 bytes)
- **Data**: Message data bytes in hexadecimal

### Send Output

```
✓ Sent: ID=0x123 DLC=4 Data=01 02 03 04
```

## Error Handling

### Unknown Driver

```bash
$ busmaster monitor --driver unknown
Error: Unknown driver: unknown
```

### Invalid ID Format

```bash
$ busmaster send --id invalid --data "01 02"
Error: invalid digit found in string
```

### Invalid Data Format

```bash
$ busmaster send --id 0x123 --data "GG HH"
Error: invalid digit found in string
```

## Tips

1. **Use stub driver for testing**: No hardware required, perfect for development
2. **Filter early**: Use `--filter-range` or `--filter-ids` to reduce noise
3. **Log everything**: Use `--log` to capture sessions for later analysis
4. **Decode signals**: Use `--dbc` with `--signals` to see physical values
5. **Limit output**: Use `--max-messages` to capture a specific number of frames

## Troubleshooting

### No messages appearing

- Check that the driver is correct (`--driver stub` for testing)
- Verify the channel number (`--channel 0` is default)
- Check if filters are too restrictive

### Permission denied (hardware drivers)

- Ensure you have permissions to access the CAN hardware
- On Linux, you may need to add your user to the `dialout` group
- On macOS, you may need to grant USB access permissions

### DBC file not loading

- Verify the DBC file path is correct
- Check that the DBC file is valid (use `--verbose` for details)
- Ensure the file uses standard DBC format

## See Also

- [BUSMASTER Engine Documentation](../busmaster-engine/README.md)
- [DBC Parser Documentation](../busmaster-db/README.md)
- [ASC Logger Documentation](../busmaster-log/README.md)
