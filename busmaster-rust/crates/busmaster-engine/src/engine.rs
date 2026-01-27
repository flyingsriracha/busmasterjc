//! Main engine implementation

use crate::{
    error::{EngineError, Result},
    subscription::{create_channel, MessageEvent, Subscriber},
};
use busmaster_core::{CanFrame, MessageFilter};
use busmaster_db::dbc::DbcDatabase;
use busmaster_dil::CanDriver;
use busmaster_log::asc::AscWriter;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{broadcast, Mutex, RwLock},
    task::JoinHandle,
    time,
};
use tracing::{debug, error, info, warn};

/// Engine state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// Engine is stopped
    Stopped,
    /// Engine is starting
    Starting,
    /// Engine is running
    Running,
    /// Engine is stopping
    Stopping,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Subscription channel capacity
    pub subscription_capacity: usize,
    /// Message reception polling interval
    pub poll_interval: Duration,
    /// Enable automatic signal extraction
    pub auto_extract_signals: bool,
    /// Maximum number of messages to buffer
    pub message_buffer_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            subscription_capacity: 1000,
            poll_interval: Duration::from_millis(1),
            auto_extract_signals: true,
            message_buffer_size: 10000,
        }
    }
}

/// Main BUSMASTER engine
///
/// Coordinates all components and manages the message processing pipeline.
pub struct Engine {
    /// CAN driver
    driver: Arc<Mutex<Box<dyn CanDriver>>>,
    /// Configuration
    config: EngineConfig,
    /// Current state
    state: Arc<RwLock<EngineState>>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Message broadcaster
    broadcaster: broadcast::Sender<MessageEvent>,
    /// Database (optional)
    database: Arc<RwLock<Option<DbcDatabase>>>,
    /// Message filter (optional)
    filter: Arc<RwLock<Option<MessageFilter>>>,
    /// Logger (optional)
    logger: Arc<Mutex<Option<AscWriter>>>,
    /// Reception task handle
    rx_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Engine {
    /// Create a new engine with the given driver and configuration
    ///
    /// # Arguments
    ///
    /// * `driver` - The CAN driver to use
    /// * `config` - Engine configuration
    ///
    /// # Returns
    ///
    /// A new engine instance
    ///
    /// # Example
    ///
    /// ```no_run
    /// use busmaster_engine::{Engine, EngineConfig};
    /// use busmaster_hardware::StubDriver;
    ///
    /// let driver = Box::new(StubDriver::new());
    /// let config = EngineConfig::default();
    /// let engine = Engine::new(driver, config).unwrap();
    /// ```
    pub fn new(driver: Box<dyn CanDriver>, config: EngineConfig) -> Result<Self> {
        let (broadcaster, _) = create_channel(config.subscription_capacity);

        Ok(Self {
            driver: Arc::new(Mutex::new(driver)),
            config,
            state: Arc::new(RwLock::new(EngineState::Stopped)),
            running: Arc::new(AtomicBool::new(false)),
            broadcaster,
            database: Arc::new(RwLock::new(None)),
            filter: Arc::new(RwLock::new(None)),
            logger: Arc::new(Mutex::new(None)),
            rx_task: Arc::new(Mutex::new(None)),
        })
    }

    /// Get the current engine state
    pub async fn state(&self) -> EngineState {
        *self.state.read().await
    }

    /// Check if the engine is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Load a DBC database
    ///
    /// # Arguments
    ///
    /// * `dbc_content` - The DBC file content as a string
    ///
    /// # Returns
    ///
    /// Ok if the database was loaded successfully
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use busmaster_engine::{Engine, EngineConfig};
    /// # use busmaster_hardware::StubDriver;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let driver = Box::new(StubDriver::new());
    /// # let mut engine = Engine::new(driver, EngineConfig::default())?;
    /// let dbc = r#"
    /// VERSION ""
    /// BU_: ECU1
    /// BO_ 100 TestMsg: 8 ECU1
    ///  SG_ TestSig : 0|8@1+ (1,0) [0|255] "" ECU1
    /// "#;
    /// engine.load_database(dbc).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_database(&mut self, dbc_content: &str) -> Result<()> {
        info!("Loading DBC database");
        let db = busmaster_db::dbc::DbcParser::parse(dbc_content)
            .map_err(|e| EngineError::Database(e.to_string()))?;

        info!(
            "Loaded database with {} messages, {} nodes",
            db.messages.len(),
            db.nodes.len()
        );

        *self.database.write().await = Some(db);
        Ok(())
    }

    /// Set the message filter
    ///
    /// # Arguments
    ///
    /// * `filter` - The message filter to use
    pub async fn set_filter(&mut self, filter: MessageFilter) {
        info!("Setting message filter with {} rules", filter.rule_count());
        *self.filter.write().await = Some(filter);
    }

    /// Clear the message filter
    pub async fn clear_filter(&mut self) {
        info!("Clearing message filter");
        *self.filter.write().await = None;
    }

    /// Enable logging to an ASC file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ASC log file
    ///
    /// # Returns
    ///
    /// Ok if logging was enabled successfully
    pub async fn enable_logging(&mut self, path: PathBuf) -> Result<()> {
        info!("Enabling logging to {:?}", path);
        let writer = AscWriter::create(path)?;
        *self.logger.lock().await = Some(writer);
        Ok(())
    }

    /// Disable logging
    pub async fn disable_logging(&mut self) -> Result<()> {
        info!("Disabling logging");
        let mut logger = self.logger.lock().await;
        if let Some(writer) = logger.take() {
            writer.close()?;
        }
        Ok(())
    }

    /// Subscribe to message events
    ///
    /// # Returns
    ///
    /// A subscriber that receives message events
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use busmaster_engine::{Engine, EngineConfig};
    /// # use busmaster_hardware::StubDriver;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let driver = Box::new(StubDriver::new());
    /// # let mut engine = Engine::new(driver, EngineConfig::default())?;
    /// let mut subscriber = engine.subscribe();
    /// engine.start().await?;
    ///
    /// while let Ok(event) = subscriber.recv().await {
    ///     println!("Event: {:?}", event);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe(&self) -> Subscriber {
        self.broadcaster.subscribe()
    }

    /// Start the engine
    ///
    /// This starts the message reception loop and begins processing messages.
    ///
    /// # Returns
    ///
    /// Ok if the engine started successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the engine is already running or if the driver fails to start.
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(EngineError::AlreadyRunning);
        }

        info!("Starting engine");
        *self.state.write().await = EngineState::Starting;

        // Open driver channel
        let mut driver = self.driver.lock().await;
        let config = busmaster_dil::ChannelConfig::default();
        driver.open_channel(0, &config)?;
        drop(driver);

        // Set running flag
        self.running.store(true, Ordering::Relaxed);

        // Start reception task
        let task = self.spawn_reception_task();
        *self.rx_task.lock().await = Some(task);

        *self.state.write().await = EngineState::Running;
        info!("Engine started");

        Ok(())
    }

    /// Stop the engine
    ///
    /// This stops the message reception loop and closes the driver.
    ///
    /// # Returns
    ///
    /// Ok if the engine stopped successfully
    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running() {
            return Err(EngineError::NotRunning);
        }

        info!("Stopping engine");
        *self.state.write().await = EngineState::Stopping;

        // Stop reception task
        self.running.store(false, Ordering::Relaxed);

        if let Some(task) = self.rx_task.lock().await.take() {
            task.abort();
            let _ = task.await;
        }

        // Close driver channel
        let mut driver = self.driver.lock().await;
        driver.close_channel(0)?;
        drop(driver);

        // Close logger
        self.disable_logging().await?;

        *self.state.write().await = EngineState::Stopped;
        info!("Engine stopped");

        Ok(())
    }

    /// Spawn the message reception task
    fn spawn_reception_task(&self) -> JoinHandle<()> {
        let driver = Arc::clone(&self.driver);
        let running = Arc::clone(&self.running);
        let broadcaster = self.broadcaster.clone();
        let filter = Arc::clone(&self.filter);
        let logger = Arc::clone(&self.logger);
        let database = Arc::clone(&self.database);
        let poll_interval = self.config.poll_interval;
        let auto_extract = self.config.auto_extract_signals;

        tokio::spawn(async move {
            debug!("Reception task started");

            while running.load(Ordering::Relaxed) {
                // Receive messages from driver
                let frame_opt = {
                    let mut driver = driver.lock().await;
                    match driver.receive(0) {
                        Ok(frame_opt) => frame_opt,
                        Err(e) => {
                            error!("Error receiving frames: {}", e);
                            let _ = broadcaster.send(MessageEvent::Error {
                                message: e.to_string(),
                            });
                            time::sleep(poll_interval).await;
                            continue;
                        },
                    }
                };

                let frame = match frame_opt {
                    Some(f) => f,
                    None => {
                        time::sleep(poll_interval).await;
                        continue;
                    },
                };

                debug!("Received frame");

                // Apply filter
                let filter_guard = filter.read().await;
                if let Some(f) = filter_guard.as_ref() {
                    if !f.matches(&frame, 0) {
                        continue;
                    }
                }
                drop(filter_guard);

                // Log frame
                let mut logger_guard = logger.lock().await;
                if let Some(writer) = logger_guard.as_mut() {
                    let timestamp = Duration::from_micros(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as u64,
                    );
                    if let Err(e) = writer.log_frame(&frame, timestamp, 0, false) {
                        warn!("Error logging frame: {}", e);
                    }
                }
                drop(logger_guard);

                // Extract signals if database is loaded
                if auto_extract {
                    let db_guard = database.read().await;
                    if let Some(db) = db_guard.as_ref() {
                        if let Some(msg) = db.find_message(frame.id()) {
                            for signal in &msg.signals {
                                let signal_def = signal.to_signal_def();
                                match signal_def.extract(frame.data()) {
                                    Ok(value) => {
                                        debug!(
                                            "Extracted signal {}: {}",
                                            signal.name, value.physical_value
                                        );
                                    },
                                    Err(e) => {
                                        warn!("Error extracting signal {}: {}", signal.name, e);
                                    },
                                }
                            }
                        }
                    }
                    drop(db_guard);
                }

                // Broadcast event
                let event = MessageEvent::FrameReceived {
                    frame,
                    channel: 0,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_micros() as u64,
                };

                if broadcaster.send(event).is_err() {
                    // No subscribers, that's ok
                }

                time::sleep(poll_interval).await;
            }

            debug!("Reception task stopped");
        })
    }

    /// Send a CAN frame
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to send
    /// * `channel` - The channel to send on
    ///
    /// # Returns
    ///
    /// Ok if the frame was sent successfully
    pub async fn send_frame(&self, frame: &CanFrame, channel: u8) -> Result<()> {
        if !self.is_running() {
            return Err(EngineError::NotRunning);
        }

        let mut driver = self.driver.lock().await;
        driver.send(channel, frame)?;

        // Broadcast event
        let event = MessageEvent::FrameTransmitted {
            frame: frame.clone(),
            channel,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        };

        let _ = self.broadcaster.send(event);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use busmaster_hardware::StubDriver;

    #[tokio::test]
    async fn test_engine_creation() {
        let driver = Box::new(StubDriver::new());
        let config = EngineConfig::default();
        let engine = Engine::new(driver, config);
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_engine_state() {
        let driver = Box::new(StubDriver::new());
        let engine = Engine::new(driver, EngineConfig::default()).unwrap();
        assert_eq!(engine.state().await, EngineState::Stopped);
        assert!(!engine.is_running());
    }

    #[tokio::test]
    async fn test_load_database() {
        let driver = Box::new(StubDriver::new());
        let mut engine = Engine::new(driver, EngineConfig::default()).unwrap();

        let dbc = r#"VERSION ""
BU_: ECU1
BO_ 100 TestMsg: 8 ECU1
 SG_ TestSig : 0|8@1+ (1,0) [0|255] "" ECU1
"#;

        let result = engine.load_database(dbc).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_stop() {
        let driver = Box::new(StubDriver::new());
        let mut engine = Engine::new(driver, EngineConfig::default()).unwrap();

        // Start engine
        let result = engine.start().await;
        assert!(result.is_ok());
        assert!(engine.is_running());
        assert_eq!(engine.state().await, EngineState::Running);

        // Stop engine
        let result = engine.stop().await;
        assert!(result.is_ok());
        assert!(!engine.is_running());
        assert_eq!(engine.state().await, EngineState::Stopped);
    }

    #[tokio::test]
    async fn test_double_start() {
        let driver = Box::new(StubDriver::new());
        let mut engine = Engine::new(driver, EngineConfig::default()).unwrap();

        engine.start().await.unwrap();
        let result = engine.start().await;
        assert!(matches!(result, Err(EngineError::AlreadyRunning)));

        engine.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        let driver = Box::new(StubDriver::new());
        let mut engine = Engine::new(driver, EngineConfig::default()).unwrap();

        let result = engine.stop().await;
        assert!(matches!(result, Err(EngineError::NotRunning)));
    }

    #[tokio::test]
    async fn test_subscription() {
        let driver = Box::new(StubDriver::new());
        let engine = Engine::new(driver, EngineConfig::default()).unwrap();

        let _subscriber = engine.subscribe();
        // Just verify we can create a subscriber
    }

    #[tokio::test]
    async fn test_filter() {
        let driver = Box::new(StubDriver::new());
        let mut engine = Engine::new(driver, EngineConfig::default()).unwrap();

        let filter = MessageFilter::new();
        engine.set_filter(filter).await;
        engine.clear_filter().await;
    }
}
