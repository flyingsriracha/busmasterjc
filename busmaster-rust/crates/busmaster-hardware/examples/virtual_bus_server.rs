//! Virtual CAN Bus Server Example
//!
//! This example starts a virtual CAN bus server that allows multiple
//! processes to communicate through a shared virtual bus.
//!
//! # Usage
//!
//! ```bash
//! # Start the server
//! cargo run --package busmaster-hardware --example virtual_bus_server
//!
//! # In another terminal, connect with a driver
//! # (CLI/TUI integration coming soon)
//! ```

use busmaster_hardware::VirtualBus;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚌 Starting Virtual CAN Bus Server...");
    println!();
    println!("Socket: /tmp/busmaster-virtual-can.sock");
    println!("Max connections: 10");
    println!();
    println!("Press Ctrl+C to stop the server.");
    println!();

    // Create and start the virtual bus
    let mut bus = VirtualBus::new();
    bus.start().await?;

    println!("✅ Server started! Waiting for connections...");
    println!();

    // Run the server until Ctrl+C
    tokio::select! {
        result = bus.run() => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            println!();
            println!("🛑 Shutting down...");
        }
    }

    // Clean up
    bus.stop().await?;
    println!("✅ Server stopped.");

    Ok(())
}
