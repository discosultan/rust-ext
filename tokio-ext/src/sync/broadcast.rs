use std::ops::Deref;

use tokio::sync::broadcast::error::SendError;
use tracing::warn;

#[must_use]
pub fn channel<T: Clone>(capacity: usize) -> (Sender<T>, tokio::sync::broadcast::Receiver<T>) {
    let (tx, rx) = tokio::sync::broadcast::channel(capacity);
    (Sender { capacity, tx }, rx)
}

pub use tokio::sync::broadcast::Receiver;

pub struct Sender<T> {
    capacity: usize,
    tx: tokio::sync::broadcast::Sender<T>,
}

impl<T> Deref for Sender<T> {
    type Target = tokio::sync::broadcast::Sender<T>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl<T> Sender<T> {
    #[track_caller]
    pub fn send_log_backpressure(&self, value: T) -> Result<usize, SendError<T>> {
        let capacity = self.capacity;
        let len = self.len();
        if len > capacity / 2 {
            warn!(
                len = len,
                capacity = capacity,
                "High channel usage in broadcast send."
            );
        }
        self.send(value)
    }
}
