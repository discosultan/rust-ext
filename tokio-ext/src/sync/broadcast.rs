use std::{ops::Deref, sync::atomic::AtomicU64};

use tokio::sync::broadcast::error::SendError;
use tracing::warn;

use super::log_throttle::should_log;

#[must_use]
pub fn channel<T: Clone>(capacity: usize) -> (Sender<T>, tokio::sync::broadcast::Receiver<T>) {
    let (tx, rx) = tokio::sync::broadcast::channel(capacity);
    (
        Sender {
            capacity,
            tx,
            next_log_ms: AtomicU64::new(0),
        },
        rx,
    )
}

pub use tokio::sync::broadcast::Receiver;

pub struct Sender<T> {
    capacity: usize,
    tx: tokio::sync::broadcast::Sender<T>,
    next_log_ms: AtomicU64,
}

impl<T> Deref for Sender<T> {
    type Target = tokio::sync::broadcast::Sender<T>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl<T> Sender<T> {
    /// Sends a value, logging on high channel usage. To keep the logging off
    /// the hot path, the warning is emitted at most once per second per
    /// channel.
    #[track_caller]
    pub fn send_log_backpressure(&self, value: T) -> Result<usize, SendError<T>> {
        let capacity = self.capacity;
        let len = self.len();
        if len > capacity / 2 && should_log(&self.next_log_ms) {
            warn!(
                len = len,
                capacity = capacity,
                "High channel usage in broadcast send."
            );
        }
        self.send(value)
    }
}
