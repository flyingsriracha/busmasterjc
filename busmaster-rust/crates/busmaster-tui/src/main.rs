//! BUSMASTER TUI Application
//!
//! Interactive terminal user interface for BUSMASTER

#![allow(dead_code)]
#![allow(clippy::while_let_loop)]

use busmaster_core::CanFrame;
use busmaster_engine::{Engine, EngineConfig, MessageEvent};
use busmaster_hardware::StubDriver;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tracing::info;

/// Application state
struct App {
    /// Running flag
    running: bool,
    /// Messages received
    messages: Vec<MessageData>,
    /// Selected message index
    selected: usize,
    /// Status message
    status: String,
    /// Statistics
    stats: Statistics,
    /// Current view
    view: View,
    /// Filter dialog state
    filter_dialog: Option<FilterDialog>,
}

/// Message data for display
#[derive(Clone)]
struct MessageData {
    timestamp: u64,
    channel: u8,
    id: u32,
    is_extended: bool,
    dlc: u8,
    data: Vec<u8>,
}

/// Statistics
struct Statistics {
    total_messages: usize,
    messages_per_second: f64,
    start_time: Instant,
    last_update: Instant,
}

/// Current view
#[derive(PartialEq)]
enum View {
    MessageList,
    Help,
}

/// Filter dialog state
struct FilterDialog {
    input: String,
    filter_type: FilterType,
}

#[derive(PartialEq)]
enum FilterType {
    IdRange,
    IdList,
}

impl App {
    fn new() -> Self {
        Self {
            running: true,
            messages: Vec::new(),
            selected: 0,
            status: "Ready".to_string(),
            stats: Statistics {
                total_messages: 0,
                messages_per_second: 0.0,
                start_time: Instant::now(),
                last_update: Instant::now(),
            },
            view: View::MessageList,
            filter_dialog: None,
        }
    }

    fn add_message(&mut self, msg: MessageData) {
        self.messages.push(msg);
        self.stats.total_messages += 1;

        // Keep only last 1000 messages
        if self.messages.len() > 1000 {
            self.messages.remove(0);
            if self.selected > 0 {
                self.selected -= 1;
            }
        }

        // Update statistics
        let now = Instant::now();
        let elapsed = now.duration_since(self.stats.last_update).as_secs_f64();
        if elapsed >= 1.0 {
            self.stats.messages_per_second = self.stats.total_messages as f64
                / now.duration_since(self.stats.start_time).as_secs_f64();
            self.stats.last_update = now;
        }
    }

    fn scroll_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn scroll_down(&mut self) {
        if self.selected < self.messages.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn page_up(&mut self) {
        self.selected = self.selected.saturating_sub(10);
    }

    fn page_down(&mut self) {
        self.selected = (self.selected + 10).min(self.messages.len().saturating_sub(1));
    }

    fn clear_messages(&mut self) {
        self.messages.clear();
        self.selected = 0;
        self.stats.total_messages = 0;
        self.stats.start_time = Instant::now();
        self.stats.last_update = Instant::now();
        self.status = "Messages cleared".to_string();
    }

    fn toggle_view(&mut self) {
        self.view = match self.view {
            View::MessageList => View::Help,
            View::Help => View::MessageList,
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let test_mode = args.contains(&"--test".to_string());

    // Initialize logging to file
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::fs::File::create("/tmp/busmaster-tui.log")?)
        .init();

    info!("Starting BUSMASTER TUI");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Create engine
    let driver = Box::new(StubDriver::new());
    let config = EngineConfig {
        subscription_capacity: 1000,
        poll_interval: Duration::from_millis(1),
        auto_extract_signals: false,
        message_buffer_size: 10000,
    };

    let mut engine = Engine::new(driver, config)?;

    // Subscribe to messages
    let mut subscriber = engine.subscribe();

    // Start engine
    engine.start().await?;

    if test_mode {
        app.status = "Test mode: Generating random messages".to_string();
    } else {
        app.status = "Monitoring started".to_string();
    }

    // Create channel for UI events
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn task to receive messages
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            match subscriber.recv().await {
                Ok(event) => {
                    if tx_clone.send(event).await.is_err() {
                        break;
                    }
                },
                Err(_) => break,
            }
        }
    });

    // Spawn test message generator if in test mode
    if test_mode {
        tokio::spawn(async move {
            let mut counter = 0u32;
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Generate random test messages
                let id = 0x100 + (counter % 256);
                let data = vec![
                    (counter & 0xFF) as u8,
                    ((counter >> 8) & 0xFF) as u8,
                    ((counter >> 16) & 0xFF) as u8,
                    ((counter >> 24) & 0xFF) as u8,
                ];

                // Send via engine
                if let Ok(frame) = CanFrame::new_standard(id, &data) {
                    // Inject directly into the message channel
                    let event = MessageEvent::FrameReceived {
                        frame,
                        channel: 0,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as u64,
                    };
                    let _ = tx.send(event).await;
                }

                counter += 1;
            }
        });
    }

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;

        // Handle input with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.running = false;
                        break;
                    },
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.running = false;
                        break;
                    },
                    KeyCode::Char('h') | KeyCode::F(1) => {
                        app.toggle_view();
                    },
                    KeyCode::Char('c') => {
                        app.clear_messages();
                    },
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.scroll_up();
                    },
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.scroll_down();
                    },
                    KeyCode::PageUp => {
                        app.page_up();
                    },
                    KeyCode::PageDown => {
                        app.page_down();
                    },
                    KeyCode::Home => {
                        app.selected = 0;
                    },
                    KeyCode::End => {
                        app.selected = app.messages.len().saturating_sub(1);
                    },
                    _ => {},
                }
            }
        }

        // Process messages
        while let Ok(event) = rx.try_recv() {
            match event {
                MessageEvent::FrameReceived {
                    frame,
                    channel,
                    timestamp,
                } => {
                    app.add_message(MessageData {
                        timestamp,
                        channel,
                        id: frame.id(),
                        is_extended: frame.is_extended(),
                        dlc: frame.dlc(),
                        data: frame.data().to_vec(),
                    });
                },
                MessageEvent::Error { message } => {
                    app.status = format!("Error: {}", message);
                },
                _ => {},
            }
        }

        if !app.running {
            break;
        }
    }

    // Cleanup
    engine.stop().await?;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    info!("BUSMASTER TUI stopped");
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    match app.view {
        View::MessageList => draw_message_list(f, app),
        View::Help => draw_help(f),
    }
}

fn draw_message_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Message list
            Constraint::Length(3), // Statistics
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("BUSMASTER - CAN Bus Monitor")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Message list
    draw_message_table(f, app, chunks[1]);

    // Statistics
    let stats_text = format!(
        " Total: {} | Rate: {:.1} msg/s | Selected: {}/{}",
        app.stats.total_messages,
        app.stats.messages_per_second,
        app.selected + 1,
        app.messages.len()
    );
    let stats = Paragraph::new(stats_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Statistics"));
    f.render_widget(stats, chunks[2]);

    // Status bar
    let status_text = format!(
        " {} | Press 'h' for help, 'q' to quit, 'c' to clear",
        app.status
    );
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[3]);
}

fn draw_message_table(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Time", "Ch", "ID", "DLC", "Data"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, msg)| {
            let time = format!("{:.3}", msg.timestamp as f64 / 1000000.0);
            let id = if msg.is_extended {
                format!("{:08X}", msg.id)
            } else {
                format!("{:03X}", msg.id)
            };
            let data = msg
                .data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                time,
                msg.channel.to_string(),
                id,
                msg.dlc.to_string(),
                data,
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(4),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Messages (↑↓ to scroll, PgUp/PgDn, Home/End)"),
    );

    f.render_widget(table, area);
}

fn draw_help(f: &mut Frame) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "BUSMASTER TUI - Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("  ↑/k         - Scroll up"),
        Line::from("  ↓/j         - Scroll down"),
        Line::from("  PgUp        - Page up"),
        Line::from("  PgDn        - Page down"),
        Line::from("  Home        - Go to first message"),
        Line::from("  End         - Go to last message"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("  c           - Clear messages"),
        Line::from("  h / F1      - Toggle help"),
        Line::from("  q / Esc     - Quit"),
        Line::from("  Ctrl+C      - Force quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Features:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("  • Real-time CAN bus monitoring"),
        Line::from("  • Message history (last 1000 messages)"),
        Line::from("  • Message rate statistics"),
        Line::from("  • Keyboard navigation"),
        Line::from("  • Message selection and highlighting"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Status:",
            Style::default().fg(Color::Green),
        )]),
        Line::from("  Driver: Stub (virtual CAN device)"),
        Line::from("  Mode: Loopback"),
        Line::from("  Logging: /tmp/busmaster-tui.log"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press 'h' or F1 to return to message list",
            Style::default().fg(Color::Cyan),
        )]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::White));

    f.render_widget(help, f.size());
}
