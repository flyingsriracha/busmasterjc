//! BUSMASTER CLI Application

#![allow(clippy::too_many_arguments)]
#![allow(clippy::print_literal)]

use busmaster_core::{CanFrame, FilterRule, MessageFilter};
use busmaster_engine::{Engine, EngineConfig, MessageEvent};
use busmaster_hardware::StubDriver;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "busmaster")]
#[command(author, version, about = "BUSMASTER - Automotive Bus Monitor", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Monitor CAN bus traffic
    Monitor {
        /// Driver to use (stub, peak)
        #[arg(short, long, default_value = "stub")]
        driver: String,

        /// Channel number
        #[arg(short, long, default_value = "0")]
        channel: u8,

        /// Baudrate
        #[arg(short, long, default_value = "500000")]
        baudrate: u32,

        /// DBC file for signal decoding
        #[arg(long)]
        dbc: Option<PathBuf>,

        /// Log file path (ASC format)
        #[arg(long)]
        log: Option<PathBuf>,

        /// Filter by ID range (e.g., "0x100-0x1FF")
        #[arg(long)]
        filter_range: Option<String>,

        /// Filter by ID list (e.g., "0x100,0x200,0x300")
        #[arg(long)]
        filter_ids: Option<String>,

        /// Show signal values (requires --dbc)
        #[arg(long)]
        signals: bool,

        /// Maximum number of messages to display (0 = unlimited)
        #[arg(long, default_value = "0")]
        max_messages: usize,
    },

    /// Send a CAN message
    Send {
        /// Driver to use
        #[arg(short, long, default_value = "stub")]
        driver: String,

        /// Channel number
        #[arg(short, long, default_value = "0")]
        channel: u8,

        /// Message ID (hex, e.g., "0x123" or "123")
        #[arg(short, long)]
        id: String,

        /// Data bytes (hex, space or comma-separated, e.g., "01 02 03" or "01,02,03")
        #[arg(short = 'D', long)]
        data: String,

        /// Use extended ID format
        #[arg(short, long)]
        extended: bool,
    },

    /// List available hardware
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let result = match cli.command {
        Some(Commands::Monitor {
            driver,
            channel,
            baudrate,
            dbc,
            log,
            filter_range,
            filter_ids,
            signals,
            max_messages,
        }) => {
            monitor_command(
                driver,
                channel,
                baudrate,
                dbc,
                log,
                filter_range,
                filter_ids,
                signals,
                max_messages,
            )
            .await
        },
        Some(Commands::Send {
            driver,
            channel,
            id,
            data,
            extended,
        }) => send_command(driver, channel, id, data, extended).await,
        Some(Commands::List) => {
            list_command();
            Ok(())
        },
        None => {
            println!("BUSMASTER v{}", env!("CARGO_PKG_VERSION"));
            println!("Use --help for usage information");
            Ok(())
        },
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn monitor_command(
    driver_name: String,
    channel: u8,
    baudrate: u32,
    dbc_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    filter_range: Option<String>,
    filter_ids: Option<String>,
    show_signals: bool,
    max_messages: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Starting monitor: driver={}, channel={}, baudrate={}",
        driver_name, channel, baudrate
    );

    // Create driver
    let driver: Box<dyn busmaster_dil::CanDriver> = match driver_name.as_str() {
        "stub" => Box::new(StubDriver::new()),
        _ => {
            return Err(format!("Unknown driver: {}", driver_name).into());
        },
    };

    // Create engine
    let config = EngineConfig {
        subscription_capacity: 1000,
        poll_interval: Duration::from_millis(1),
        auto_extract_signals: show_signals,
        message_buffer_size: 10000,
    };

    let mut engine = Engine::new(driver, config)?;

    // Load DBC if provided
    if let Some(dbc_path) = dbc_path {
        info!("Loading DBC: {:?}", dbc_path);
        let dbc_content = std::fs::read_to_string(&dbc_path)?;
        engine.load_database(&dbc_content).await?;
        println!("✓ Loaded database: {:?}", dbc_path);
    }

    // Enable logging if provided
    if let Some(log_path) = log_path {
        info!("Enabling logging: {:?}", log_path);
        engine.enable_logging(log_path.clone()).await?;
        println!("✓ Logging to: {:?}", log_path);
    }

    // Apply filter if provided
    if filter_range.is_some() || filter_ids.is_some() {
        let mut filter = MessageFilter::new();

        if let Some(range_str) = filter_range {
            let (start, end) = parse_id_range(&range_str)?;
            filter = filter.add_rule(FilterRule::IdRange { start, end });
            println!("✓ Filter: ID range 0x{:X}-0x{:X}", start, end);
        }

        if let Some(ids_str) = filter_ids {
            let ids = parse_id_list(&ids_str)?;
            println!("✓ Filter: ID list ({} IDs)", ids.len());
            filter = filter.add_rule(FilterRule::IdList { ids });
        }

        engine.set_filter(filter).await;
    }

    // Subscribe to messages
    let mut subscriber = engine.subscribe();

    // Start engine
    engine.start().await?;
    println!("✓ Monitoring started (Ctrl+C to stop)");
    println!();
    println!(
        "{:<12} {:<4} {:<10} {:<3} {}",
        "Time", "Ch", "ID", "DLC", "Data"
    );
    println!("{}", "-".repeat(60));

    // Process messages
    let mut count = 0;
    let start_time = std::time::Instant::now();

    loop {
        match subscriber.recv().await {
            Ok(MessageEvent::FrameReceived {
                frame,
                channel: ch,
                timestamp,
            }) => {
                display_frame(&frame, ch, timestamp);
                count += 1;

                if max_messages > 0 && count >= max_messages {
                    break;
                }
            },
            Ok(MessageEvent::FrameTransmitted { .. }) => {
                // Ignore transmitted frames in monitor mode
            },
            Ok(MessageEvent::Error { message }) => {
                error!("Error: {}", message);
            },
            Err(e) => {
                error!("Subscription error: {}", e);
                break;
            },
        }
    }

    // Stop engine
    engine.stop().await?;

    let elapsed = start_time.elapsed();
    println!();
    println!(
        "Received {} messages in {:.2}s",
        count,
        elapsed.as_secs_f64()
    );
    println!(
        "Average rate: {:.1} msg/s",
        count as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}

async fn send_command(
    driver_name: String,
    channel: u8,
    id_str: String,
    data_str: String,
    extended: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Sending message: driver={}, channel={}, id={}, data={}",
        driver_name, channel, id_str, data_str
    );

    // Parse ID
    let id = parse_id(&id_str)?;

    // Parse data
    let data = parse_data(&data_str)?;

    // Create frame
    let frame = if extended {
        CanFrame::new_extended(id, &data)?
    } else {
        CanFrame::new_standard(id, &data)?
    };

    // Create driver
    let driver: Box<dyn busmaster_dil::CanDriver> = match driver_name.as_str() {
        "stub" => Box::new(StubDriver::new()),
        _ => {
            return Err(format!("Unknown driver: {}", driver_name).into());
        },
    };

    // Create engine
    let config = EngineConfig::default();
    let mut engine = Engine::new(driver, config)?;

    // Start engine
    engine.start().await?;

    // Send frame
    engine.send_frame(&frame, channel).await?;

    println!(
        "✓ Sent: ID=0x{:X} DLC={} Data={}",
        frame.id(),
        frame.dlc(),
        format_data(frame.data())
    );

    // Stop engine
    engine.stop().await?;

    Ok(())
}

fn list_command() {
    println!("Available drivers:");
    println!();
    println!("  stub    - Virtual CAN device (loopback)");
    println!("            Always available for testing");
    println!();
    println!("  peak    - PEAK USB/PCIe devices");
    println!("            Requires PCAN hardware (not yet implemented)");
    println!();
    println!("  vector  - Vector CANcaseXL/CANcardXL devices");
    println!("            Requires Vector hardware (not yet implemented)");
}

fn display_frame(frame: &CanFrame, channel: u8, timestamp: u64) {
    let time_ms = timestamp as f64 / 1000.0;
    let id_str = if frame.is_extended() {
        format!("0x{:08X}", frame.id())
    } else {
        format!("0x{:03X}", frame.id())
    };

    println!(
        "{:>12.3} {:>4} {:>10} {:>3} {}",
        time_ms,
        channel,
        id_str,
        frame.dlc(),
        format_data(frame.data())
    );
}

fn format_data(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_id(id_str: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let id_str = id_str
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    Ok(u32::from_str_radix(id_str, 16)?)
}

fn parse_data(data_str: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let data_str = data_str.replace(',', " ");
    let parts: Vec<&str> = data_str.split_whitespace().collect();

    let mut data = Vec::new();
    for part in parts {
        let part = part.trim_start_matches("0x").trim_start_matches("0X");
        data.push(u8::from_str_radix(part, 16)?);
    }

    Ok(data)
}

fn parse_id_range(range_str: &str) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid range format. Use: 0x100-0x1FF".into());
    }

    let start = parse_id(parts[0])?;
    let end = parse_id(parts[1])?;

    if start > end {
        return Err("Invalid range: start > end".into());
    }

    Ok((start, end))
}

fn parse_id_list(ids_str: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = ids_str.split(',').collect();
    let mut ids = Vec::new();

    for part in parts {
        ids.push(parse_id(part.trim())?);
    }

    Ok(ids)
}
