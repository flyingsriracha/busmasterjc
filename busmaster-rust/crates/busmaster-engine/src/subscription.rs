//! Message subscription and pub/sub system

use busmaster_core::CanFrame;
use tokio::sync::broadcast;

/// A message event that can be subscribed to
#[derive(Debug, Clone)]
pub enum MessageEvent {
    /// A CAN frame was received
    FrameReceived {
        /// The received frame
        frame: CanFrame,
        /// The channel it was received on
        channel: u8,
        /// Timestamp in microseconds
        timestamp: u64,
    },
    /// A CAN frame was transmitted
    FrameTransmitted {
        /// The transmitted frame
        frame: CanFrame,
        /// The channel it was transmitted on
        channel: u8,
        /// Timestamp in microseconds
        timestamp: u64,
    },
    /// An error occurred
    Error {
        /// Error message
        message: String,
    },
}

/// A subscriber to message events
pub type Subscriber = broadcast::Receiver<MessageEvent>;

/// Creates a new broadcast channel for message events
pub(crate) fn create_channel(capacity: usize) -> (broadcast::Sender<MessageEvent>, Subscriber) {
    broadcast::channel(capacity)
}
