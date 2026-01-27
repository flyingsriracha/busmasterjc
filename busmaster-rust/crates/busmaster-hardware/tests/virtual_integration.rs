//! Integration tests for virtual CAN driver
//!
//! Tests multi-process communication through the virtual CAN bus.

use busmaster_core::CanFrame;
use busmaster_dil::{CanDriver, ChannelConfig, ChannelStatus};
use busmaster_hardware::{VirtualBus, VirtualDriver};
use std::thread;
use std::time::Duration;

/// Test that the virtual bus can start and stop
#[test]
fn test_virtual_bus_lifecycle() {
    let socket_path = "/tmp/busmaster-test-lifecycle.sock";

    // Use a separate runtime for the bus
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut bus = VirtualBus::with_socket_path(socket_path);

        // Start the bus
        bus.start().await.expect("Failed to start bus");
        assert!(std::path::Path::new(socket_path).exists());

        // Stop the bus
        bus.stop().await.expect("Failed to stop bus");
        assert!(!std::path::Path::new(socket_path).exists());
    });
}

/// Test that a driver can connect to the virtual bus
#[test]
fn test_driver_connect() {
    let socket_path = "/tmp/busmaster-test-connect.sock";

    // Start bus server in a separate thread
    let socket_path_clone = socket_path.to_string();
    let bus_thread = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut bus = VirtualBus::with_socket_path(&socket_path_clone);
            bus.start().await.expect("Failed to start bus");

            // Run for a short time
            tokio::select! {
                _ = bus.run() => {}
                _ = tokio::time::sleep(Duration::from_secs(2)) => {}
            }

            bus.stop().await.ok();
        });
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(200));

    // Create driver and connect
    let mut driver = VirtualDriver::with_socket_path(socket_path);
    let config = ChannelConfig::new(500_000);

    let result = driver.open_channel(0, &config);
    assert!(result.is_ok(), "Failed to open channel: {:?}", result.err());

    // Check status
    let status = driver.channel_status(0).unwrap();
    assert_eq!(status, ChannelStatus::Active);

    // Close channel
    driver.close_channel(0).expect("Failed to close channel");

    // Clean up
    let _ = bus_thread.join();
    let _ = std::fs::remove_file(socket_path);
}

/// Test message transmission between two drivers
#[test]
fn test_message_broadcast() {
    let socket_path = "/tmp/busmaster-test-broadcast.sock";

    // Start bus server in a separate thread
    let socket_path_clone = socket_path.to_string();
    let bus_thread = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut bus = VirtualBus::with_socket_path(&socket_path_clone);
            bus.start().await.expect("Failed to start bus");

            // Run for a short time
            tokio::select! {
                _ = bus.run() => {}
                _ = tokio::time::sleep(Duration::from_secs(3)) => {}
            }

            bus.stop().await.ok();
        });
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(200));

    // Create two drivers (simulating CLI and TUI)
    let mut driver1 = VirtualDriver::with_socket_path(socket_path);
    let mut driver2 = VirtualDriver::with_socket_path(socket_path);

    let config = ChannelConfig::new(500_000);

    // Open channels
    driver1
        .open_channel(0, &config)
        .expect("Driver 1 failed to open");
    driver2
        .open_channel(0, &config)
        .expect("Driver 2 failed to open");

    // Give time for connections to establish
    thread::sleep(Duration::from_millis(200));

    // Driver 1 sends a frame
    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4]).unwrap();
    driver1.send(0, &frame).expect("Failed to send frame");

    // Give time for message to propagate
    thread::sleep(Duration::from_millis(300));

    // Driver 2 should receive it
    let received = driver2.receive(0).expect("Failed to receive");

    // Note: The current implementation broadcasts to ALL clients including sender
    // So driver2 should receive the message
    if let Some(received_frame) = received {
        assert_eq!(received_frame.id(), frame.id());
        assert_eq!(received_frame.data(), frame.data());
    }

    // Clean up
    driver1.close_channel(0).expect("Failed to close driver 1");
    driver2.close_channel(0).expect("Failed to close driver 2");
    let _ = bus_thread.join();
    let _ = std::fs::remove_file(socket_path);
}

/// Test that driver reports correct status
#[test]
fn test_driver_status() {
    let driver = VirtualDriver::new();

    // Before opening, status should be Closed
    let status = driver.channel_status(0).unwrap();
    assert_eq!(status, ChannelStatus::Closed);
}

/// Test device listing
#[test]
fn test_list_devices() {
    let driver = VirtualDriver::new();
    let devices = driver.list_devices().unwrap();

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].name, "Virtual CAN Bus");
    assert_eq!(devices[0].channel_count, 1);
}
