//! MVP Integration Tests
//!
//! End-to-end tests for the BUSMASTER MVP functionality.
//! These tests verify that all components work together correctly.

use busmaster_core::{CanFrame, FilterRule, MessageFilter};
use busmaster_db::DbcParser;
use busmaster_dil::{CanDriver, ChannelConfig, ChannelStatus};
use busmaster_engine::{Engine, EngineConfig};
use busmaster_hardware::StubDriver;
use busmaster_log::AscWriter;
use busmaster_platform::current_platform;
use std::fs;
use std::time::{Duration, Instant};

/// Test 3.6.1: End-to-end test with stub driver monitoring
#[test]
fn test_e2e_stub_driver_monitoring() {
    // Create stub driver with loopback
    let mut driver = StubDriver::new();
    let config = ChannelConfig::new(500_000).with_loopback(true);

    // Open channel
    driver
        .open_channel(0, &config)
        .expect("Failed to open channel");
    assert_eq!(driver.channel_status(0).unwrap(), ChannelStatus::Active);

    // Send a frame
    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
    driver.send(0, &frame).expect("Failed to send frame");

    // Receive the frame (loopback)
    let received = driver.receive(0).expect("Failed to receive");
    assert!(received.is_some());

    let received_frame = received.unwrap();
    assert_eq!(received_frame.id(), 0x123);
    assert_eq!(received_frame.data(), &[1, 2, 3, 4]);

    // Close channel
    driver.close_channel(0).expect("Failed to close channel");
    assert_eq!(driver.channel_status(0).unwrap(), ChannelStatus::Closed);
}

/// Test 3.6.2: End-to-end test with DBC signal extraction
#[test]
fn test_e2e_dbc_signal_extraction() {
    // Sample DBC content
    let dbc_content = r#"
VERSION ""

NS_ :

BS_:

BU_: ECU1 ECU2

BO_ 291 EngineData: 8 ECU1
 SG_ EngineSpeed : 0|16@1+ (0.25,0) [0|16383.75] "rpm" ECU2
 SG_ EngineTemp : 16|8@1+ (1,-40) [-40|215] "degC" ECU2
 SG_ ThrottlePos : 24|8@1+ (0.392157,0) [0|100] "%" ECU2
"#;

    // Parse DBC
    let db = DbcParser::parse(dbc_content).expect("Failed to parse DBC");

    // Verify message was parsed
    let msg = db.find_message(291).expect("Message not found");
    assert_eq!(msg.name, "EngineData");
    assert_eq!(msg.signals.len(), 3);

    // Create a frame with known data
    // EngineSpeed = 2000 rpm -> raw = 2000 / 0.25 = 8000 = 0x1F40
    // EngineTemp = 80°C -> raw = 80 + 40 = 120 = 0x78
    // ThrottlePos = 50% -> raw = 50 / 0.392157 ≈ 127 = 0x7F
    let frame = CanFrame::new_standard(291, &[0x40, 0x1F, 0x78, 0x7F, 0, 0, 0, 0]).unwrap();

    // Extract signals
    let engine_speed_sig = msg
        .signals
        .iter()
        .find(|s| s.name == "EngineSpeed")
        .unwrap();
    let signal_def = engine_speed_sig.to_signal_def();
    let value = signal_def
        .extract(frame.data())
        .expect("Failed to extract signal");

    // Verify extracted value (should be close to 2000 rpm)
    assert!(
        (value.physical_value - 2000.0).abs() < 1.0,
        "Expected ~2000, got {}",
        value.physical_value
    );
}

/// Test 3.6.3: End-to-end test with ASC logging
#[test]
fn test_e2e_asc_logging() {
    let log_path = "/tmp/busmaster_test_e2e.asc";

    // Clean up any existing file
    let _ = fs::remove_file(log_path);

    // Create ASC writer
    let mut writer = AscWriter::create(log_path).expect("Failed to create ASC writer");

    // Log some frames
    let frames = vec![
        CanFrame::new_standard(0x100, &[1, 2, 3, 4]).unwrap(),
        CanFrame::new_standard(0x200, &[5, 6, 7, 8]).unwrap(),
        CanFrame::new_extended(0x12345, &[0xAA, 0xBB]).unwrap(),
    ];

    for (i, frame) in frames.iter().enumerate() {
        let timestamp = Duration::from_millis((i as u64) * 1); // 1ms apart
        writer
            .log_frame(frame, timestamp, 0, true)
            .expect("Failed to log frame");
    }

    // Close the writer
    writer.close().expect("Failed to close writer");

    // Verify the file was created and has content
    let content = fs::read_to_string(log_path).expect("Failed to read log file");

    // Check header
    assert!(content.contains("date"), "Missing date in header");
    assert!(content.contains("base hex"), "Missing base hex in header");

    // Check frames were logged (IDs are in decimal in ASC format)
    assert!(content.contains("256"), "Missing frame 0x100 (256)");
    assert!(content.contains("512"), "Missing frame 0x200 (512)");
    assert!(content.contains("74565"), "Missing frame 0x12345 (74565)");

    // Clean up
    let _ = fs::remove_file(log_path);
}

/// Test 3.6.4: End-to-end test with message filtering
#[test]
fn test_e2e_message_filtering() {
    // Create filter for IDs 0x100-0x1FF
    let filter = MessageFilter::new().add_rule(FilterRule::IdRange {
        start: 0x100,
        end: 0x1FF,
    });

    // Test frames
    let frames = vec![
        (CanFrame::new_standard(0x050, &[1]).unwrap(), false), // Should be blocked
        (CanFrame::new_standard(0x100, &[2]).unwrap(), true),  // Should pass
        (CanFrame::new_standard(0x150, &[3]).unwrap(), true),  // Should pass
        (CanFrame::new_standard(0x1FF, &[4]).unwrap(), true),  // Should pass
        (CanFrame::new_standard(0x200, &[5]).unwrap(), false), // Should be blocked
        (CanFrame::new_standard(0x300, &[6]).unwrap(), false), // Should be blocked
    ];

    for (frame, expected) in frames {
        let result = filter.matches(&frame, 0);
        assert_eq!(
            result,
            expected,
            "Filter mismatch for ID 0x{:03X}",
            frame.id()
        );
    }
}

/// Test 3.6.5: Performance test - 1000 msg/sec throughput
#[test]
fn test_performance_throughput() {
    let mut driver = StubDriver::new();
    let config = ChannelConfig::new(500_000).with_loopback(true);
    driver
        .open_channel(0, &config)
        .expect("Failed to open channel");

    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    let message_count = 1000;

    // Measure send throughput
    let start = Instant::now();
    for _ in 0..message_count {
        driver.send(0, &frame).expect("Failed to send");
    }
    let send_duration = start.elapsed();

    // Measure receive throughput
    let start = Instant::now();
    let mut received_count = 0;
    while let Ok(Some(_)) = driver.receive(0) {
        received_count += 1;
        if received_count >= message_count {
            break;
        }
    }
    let receive_duration = start.elapsed();

    driver.close_channel(0).expect("Failed to close channel");

    // Calculate throughput
    let send_rate = message_count as f64 / send_duration.as_secs_f64();
    let receive_rate = received_count as f64 / receive_duration.as_secs_f64();

    println!("Send throughput: {:.0} msg/sec", send_rate);
    println!("Receive throughput: {:.0} msg/sec", receive_rate);

    // Should achieve at least 1000 msg/sec
    assert!(
        send_rate >= 1000.0,
        "Send throughput too low: {:.0} msg/sec",
        send_rate
    );
    assert!(
        receive_rate >= 1000.0,
        "Receive throughput too low: {:.0} msg/sec",
        receive_rate
    );
}

/// Test 3.6.6: Stability test - continuous operation
/// Note: This is a shorter version for CI; full 1-hour test should be run manually
#[test]
fn test_stability_continuous_operation() {
    let mut driver = StubDriver::new();
    let config = ChannelConfig::new(500_000).with_loopback(true);
    driver
        .open_channel(0, &config)
        .expect("Failed to open channel");

    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
    let test_duration = Duration::from_secs(5); // 5 seconds for CI
    let start = Instant::now();
    let mut message_count = 0;

    while start.elapsed() < test_duration {
        // Send
        driver.send(0, &frame).expect("Failed to send");
        message_count += 1;

        // Receive
        while let Ok(Some(_)) = driver.receive(0) {
            // Drain receive buffer
        }

        // Small delay to prevent busy loop
        std::thread::sleep(Duration::from_micros(100));
    }

    driver.close_channel(0).expect("Failed to close channel");

    println!(
        "Processed {} messages in {:?}",
        message_count, test_duration
    );
    assert!(message_count > 0, "No messages processed");
}

/// Test 3.6.7: Memory test - verify reasonable memory usage
/// Note: This is a basic test; full memory profiling should be done separately
#[test]
fn test_memory_usage() {
    // Create multiple drivers and frames to test memory behavior
    let mut drivers: Vec<StubDriver> = Vec::new();
    let config = ChannelConfig::new(500_000);

    // Create 10 drivers
    for _ in 0..10 {
        let mut driver = StubDriver::new();
        driver
            .open_channel(0, &config)
            .expect("Failed to open channel");
        drivers.push(driver);
    }

    // Send many frames
    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    for driver in &mut drivers {
        for _ in 0..1000 {
            driver.send(0, &frame).expect("Failed to send");
        }
    }

    // Clean up
    for mut driver in drivers {
        driver.close_channel(0).expect("Failed to close channel");
    }

    // If we got here without OOM, the test passes
    // Actual memory measurement would require external tools
}

/// Test platform integration
#[test]
fn test_platform_integration() {
    let platform = current_platform();

    // Test timestamp
    let t1 = platform.timestamp_us();
    std::thread::sleep(Duration::from_millis(10));
    let t2 = platform.timestamp_us();

    assert!(t2 > t1, "Timestamp should increase");
    assert!(t2 - t1 >= 10_000, "Should have elapsed at least 10ms");

    // Test USB enumeration (should not panic)
    let devices = platform.list_usb_devices();
    assert!(devices.is_ok(), "USB enumeration should not fail");
}

/// Test engine with stub driver
#[tokio::test]
async fn test_engine_integration() {
    let driver = Box::new(StubDriver::new());
    let config = EngineConfig::default();
    let mut engine = Engine::new(driver, config).expect("Failed to create engine");

    // Start engine
    engine.start().await.expect("Failed to start engine");

    // Subscribe to messages
    let _subscriber = engine.subscribe();

    // Let it run briefly
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stop engine
    engine.stop().await.expect("Failed to stop engine");
}
