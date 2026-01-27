//! Virtual CAN Bus Server
//!
//! Manages the virtual CAN bus and broadcasts messages to all connected clients.

use super::protocol::BusMessage;
use super::{DEFAULT_SOCKET_PATH, MAX_CONNECTIONS, MESSAGE_BUFFER_SIZE};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, error, info, warn};

/// Virtual CAN bus server
pub struct VirtualBus {
    socket_path: PathBuf,
    tx: broadcast::Sender<Vec<u8>>,
    listener: Option<UnixListener>,
    running: Arc<Mutex<bool>>,
}

impl VirtualBus {
    /// Create a new virtual bus
    pub fn new() -> Self {
        Self::with_socket_path(DEFAULT_SOCKET_PATH)
    }

    /// Create a new virtual bus with custom socket path
    pub fn with_socket_path<P: Into<PathBuf>>(path: P) -> Self {
        let (tx, _rx) = broadcast::channel(MESSAGE_BUFFER_SIZE);

        Self {
            socket_path: path.into(),
            tx,
            listener: None,
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the virtual bus server
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        // Remove existing socket file if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        // Create Unix socket listener
        let listener = UnixListener::bind(&self.socket_path)?;
        info!("Virtual CAN bus started at {:?}", self.socket_path);

        self.listener = Some(listener);
        *self.running.lock().await = true;

        Ok(())
    }

    /// Stop the virtual bus server
    pub async fn stop(&mut self) -> Result<(), std::io::Error> {
        *self.running.lock().await = false;

        // Remove socket file
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        info!("Virtual CAN bus stopped");
        Ok(())
    }

    /// Run the server loop (accepts connections and handles clients)
    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        let listener = self.listener.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "Bus not started")
        })?;

        let mut connection_count = 0;

        loop {
            // Check if we should stop
            if !*self.running.lock().await {
                break;
            }

            // Accept new connection
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    if connection_count >= MAX_CONNECTIONS {
                        warn!("Max connections reached, rejecting new connection");
                        continue;
                    }

                    connection_count += 1;
                    let client_id = format!("client-{}", connection_count);
                    info!("New connection: {}", client_id);

                    // Spawn task to handle this client
                    let tx = self.tx.clone();
                    let rx = self.tx.subscribe();
                    let running = self.running.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, client_id, tx, rx, running).await {
                            error!("Client handler error: {}", e);
                        }
                    });
                },
                Err(e) => {
                    error!("Accept error: {}", e);
                },
            }
        }

        Ok(())
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

impl Drop for VirtualBus {
    fn drop(&mut self) {
        // Clean up socket file
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }
}

/// Handle a single client connection
async fn handle_client(
    stream: UnixStream,
    client_id: String,
    tx: broadcast::Sender<Vec<u8>>,
    mut rx: broadcast::Receiver<Vec<u8>>,
    running: Arc<Mutex<bool>>,
) -> Result<(), std::io::Error> {
    debug!("{}: Handler started", client_id);

    // Split stream into owned halves for reading and writing
    let (mut reader, mut writer) = stream.into_split();

    // Spawn task to receive messages from other clients
    let client_id_clone = client_id.clone();
    let receive_task = tokio::spawn(async move {
        while *running.lock().await {
            match rx.recv().await {
                Ok(bytes) => {
                    // Write length prefix
                    let len_bytes = BusMessage::length_prefix(bytes.len());
                    if writer.write_all(&len_bytes).await.is_err() {
                        break;
                    }

                    // Write message
                    if writer.write_all(&bytes).await.is_err() {
                        break;
                    }

                    if writer.flush().await.is_err() {
                        break;
                    }
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("{}: Lagged by {} messages", client_id_clone, n);
                },
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                },
            }
        }
    });

    // Read messages from this client and broadcast to others
    loop {
        // Read length prefix (4 bytes)
        let mut len_bytes = [0u8; 4];
        match reader.read_exact(&mut len_bytes).await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("{}: Client disconnected", client_id);
                break;
            },
            Err(e) => {
                error!("{}: Read error: {}", client_id, e);
                break;
            },
        }

        let len = BusMessage::parse_length_prefix(&len_bytes);

        // Read message
        let mut msg_bytes = vec![0u8; len];
        match reader.read_exact(&mut msg_bytes).await {
            Ok(_) => {},
            Err(e) => {
                error!("{}: Read message error: {}", client_id, e);
                break;
            },
        }

        // Parse message
        match BusMessage::from_bytes(&msg_bytes) {
            Ok(msg) => {
                debug!("{}: Received message: {:?}", client_id, msg);

                // Broadcast to all other clients
                if tx.send(msg_bytes).is_err() {
                    warn!("{}: No receivers", client_id);
                }
            },
            Err(e) => {
                error!("{}: Parse error: {}", client_id, e);
            },
        }
    }

    // Clean up
    receive_task.abort();
    info!("{}: Connection closed", client_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bus_start_stop() {
        let socket_path = "/tmp/busmaster-test-bus.sock";
        let mut bus = VirtualBus::with_socket_path(socket_path);

        bus.start().await.unwrap();
        assert!(std::path::Path::new(socket_path).exists());

        bus.stop().await.unwrap();
        assert!(!std::path::Path::new(socket_path).exists());
    }
}
