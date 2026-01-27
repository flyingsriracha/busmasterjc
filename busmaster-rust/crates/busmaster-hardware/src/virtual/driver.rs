//! Virtual CAN Driver Implementation
//!
//! Implements the CanDriver trait for the virtual CAN bus.

use super::protocol::BusMessage;
use super::DEFAULT_SOCKET_PATH;
use busmaster_core::{BusmasterError, CanFrame, Result};
use busmaster_dil::{CanDriver, ChannelConfig, ChannelStatus, DeviceInfo};
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::UnixStream;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// Virtual CAN driver
pub struct VirtualDriver {
    socket_path: PathBuf,
    runtime: Arc<Runtime>,
    writer: Arc<Mutex<Option<OwnedWriteHalf>>>,
    rx_channel: Arc<Mutex<Option<mpsc::Receiver<(CanFrame, u8, u64)>>>>,
    rx_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    is_open: bool,
    client_id: String,
}

impl VirtualDriver {
    /// Create a new virtual driver
    pub fn new() -> Self {
        Self::with_socket_path(DEFAULT_SOCKET_PATH)
    }

    /// Create a new virtual driver with custom socket path
    pub fn with_socket_path<P: Into<PathBuf>>(path: P) -> Self {
        let client_id = format!("client-{}", std::process::id());
        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));

        Self {
            socket_path: path.into(),
            runtime,
            writer: Arc::new(Mutex::new(None)),
            rx_channel: Arc::new(Mutex::new(None)),
            rx_task: Arc::new(Mutex::new(None)),
            is_open: false,
            client_id,
        }
    }

    /// Connect to the virtual bus and return split stream halves
    async fn connect_async(&self) -> Result<(OwnedReadHalf, OwnedWriteHalf)> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(BusmasterError::Io)?;

        // Split into owned halves
        let (reader, mut writer) = stream.into_split();

        // Send connect message
        let msg = BusMessage::Connect {
            client_id: self.client_id.clone(),
        };

        Self::send_message_to_writer(&mut writer, &msg).await?;

        info!("Connected to virtual bus at {:?}", self.socket_path);

        Ok((reader, writer))
    }

    /// Send a message using the writer half
    async fn send_message_to_writer(writer: &mut OwnedWriteHalf, msg: &BusMessage) -> Result<()> {
        let bytes = msg.to_bytes().map_err(|e| BusmasterError::Protocol {
            message: e.to_string(),
        })?;
        let len_bytes = BusMessage::length_prefix(bytes.len());

        writer
            .write_all(&len_bytes)
            .await
            .map_err(BusmasterError::Io)?;
        writer.write_all(&bytes).await.map_err(BusmasterError::Io)?;
        writer.flush().await.map_err(BusmasterError::Io)?;

        Ok(())
    }

    /// Start receiving messages from the reader half
    fn start_receiver(&self, mut reader: OwnedReadHalf) {
        let (tx, rx) = mpsc::channel(1000);

        let task = self.runtime.spawn(async move {
            loop {
                // Read length prefix
                let mut len_bytes = [0u8; 4];
                match reader.read_exact(&mut len_bytes).await {
                    Ok(_) => {},
                    Err(e) => {
                        if e.kind() != ErrorKind::UnexpectedEof {
                            error!("Read length error: {}", e);
                        }
                        break;
                    },
                }

                let len = BusMessage::parse_length_prefix(&len_bytes);

                // Read message
                let mut msg_bytes = vec![0u8; len];
                match reader.read_exact(&mut msg_bytes).await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Read message error: {}", e);
                        break;
                    },
                }

                // Parse message
                match BusMessage::from_bytes(&msg_bytes) {
                    Ok(BusMessage::Frame {
                        frame,
                        channel,
                        timestamp,
                    }) => {
                        debug!("Received frame: ID={:03X}", frame.id());
                        if tx.send((frame, channel, timestamp)).await.is_err() {
                            break;
                        }
                    },
                    Ok(msg) => {
                        debug!("Received message: {:?}", msg);
                    },
                    Err(e) => {
                        error!("Parse error: {}", e);
                    },
                }
            }
        });

        self.runtime.block_on(async {
            *self.rx_channel.lock().await = Some(rx);
            *self.rx_task.lock().await = Some(task);
        });
    }
}

impl Default for VirtualDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl CanDriver for VirtualDriver {
    fn name(&self) -> &str {
        "Virtual CAN"
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        Ok(vec![DeviceInfo::new("Virtual CAN Bus", "virtual-0", 1)])
    }

    fn open_channel(&mut self, _channel: u8, config: &ChannelConfig) -> Result<()> {
        if self.is_open {
            return Err(BusmasterError::Hardware {
                vendor: "Virtual".to_string(),
                message: "Channel already open".to_string(),
            });
        }

        // Connect to virtual bus
        let (reader, writer) = self.runtime.block_on(self.connect_async())?;

        // Store writer for sending
        self.runtime.block_on(async {
            *self.writer.lock().await = Some(writer);
        });

        // Start receiver task with reader
        self.start_receiver(reader);

        self.is_open = true;
        info!("Virtual channel opened with config: {:?}", config);

        Ok(())
    }

    fn close_channel(&mut self, _channel: u8) -> Result<()> {
        if !self.is_open {
            return Err(BusmasterError::Hardware {
                vendor: "Virtual".to_string(),
                message: "Channel not open".to_string(),
            });
        }

        // Stop receiver task
        self.runtime.block_on(async {
            if let Some(task) = self.rx_task.lock().await.take() {
                task.abort();
            }
        });

        // Send disconnect message and close writer
        self.runtime.block_on(async {
            if let Some(mut writer) = self.writer.lock().await.take() {
                let msg = BusMessage::Disconnect {
                    client_id: self.client_id.clone(),
                };
                let _ = Self::send_message_to_writer(&mut writer, &msg).await;
            }
        });

        self.is_open = false;
        info!("Virtual channel closed");

        Ok(())
    }

    fn send(&mut self, _channel: u8, frame: &CanFrame) -> Result<()> {
        if !self.is_open {
            return Err(BusmasterError::Hardware {
                vendor: "Virtual".to_string(),
                message: "Channel not open".to_string(),
            });
        }

        self.runtime.block_on(async {
            let mut writer_guard = self.writer.lock().await;

            if let Some(writer) = writer_guard.as_mut() {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|e| BusmasterError::Protocol {
                        message: e.to_string(),
                    })?
                    .as_micros() as u64;

                let msg = BusMessage::Frame {
                    frame: frame.clone(),
                    channel: 0,
                    timestamp,
                };

                Self::send_message_to_writer(writer, &msg).await?;
                debug!("Sent frame: ID={:03X}", frame.id());

                Ok(())
            } else {
                Err(BusmasterError::Hardware {
                    vendor: "Virtual".to_string(),
                    message: "Channel not open".to_string(),
                })
            }
        })
    }

    fn receive(&mut self, _channel: u8) -> Result<Option<CanFrame>> {
        if !self.is_open {
            return Err(BusmasterError::Hardware {
                vendor: "Virtual".to_string(),
                message: "Channel not open".to_string(),
            });
        }

        self.runtime.block_on(async {
            let mut rx_guard = self.rx_channel.lock().await;

            if let Some(rx) = rx_guard.as_mut() {
                match rx.try_recv() {
                    Ok((frame, _channel, _timestamp)) => Ok(Some(frame)),
                    Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                    Err(mpsc::error::TryRecvError::Disconnected) => Err(BusmasterError::Io(
                        IoError::new(ErrorKind::BrokenPipe, "Receiver disconnected"),
                    )),
                }
            } else {
                Ok(None)
            }
        })
    }

    fn channel_status(&self, _channel: u8) -> Result<ChannelStatus> {
        if self.is_open {
            Ok(ChannelStatus::Active)
        } else {
            Ok(ChannelStatus::Closed)
        }
    }
}

impl Drop for VirtualDriver {
    fn drop(&mut self) {
        if self.is_open {
            let _ = self.close_channel(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_creation() {
        let driver = VirtualDriver::new();
        assert_eq!(driver.socket_path, PathBuf::from(DEFAULT_SOCKET_PATH));
        assert_eq!(driver.name(), "Virtual CAN");
    }

    #[test]
    fn test_list_devices() {
        let driver = VirtualDriver::new();
        let devices = driver.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Virtual CAN Bus");
    }
}
