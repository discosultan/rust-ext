use std::panic::Location;

use tokio::sync::mpsc::{
    PermitIterator, Sender,
    error::{SendError, TrySendError},
};
use tracing::warn;

pub trait SenderExt<'a, T: 'a> {
    fn send_log_backpressure(&self, value: T) -> impl Future<Output = Result<(), SendError<T>>>;

    fn try_send_discard_full_log_backpressure(&self, value: T) -> Result<(), SendError<T>>;

    fn reserve_many_log_backpressure(
        &'a self,
        size: usize,
    ) -> impl Future<Output = Result<PermitIterator<'a, T>, SendError<()>>>;
}

impl<'a, T: 'a> SenderExt<'a, T> for Sender<T> {
    #[track_caller]
    fn send_log_backpressure(&self, value: T) -> impl Future<Output = Result<(), SendError<T>>> {
        let caller = Location::caller();
        send_log_backpressure_impl(self, value, caller)
    }

    #[track_caller]
    fn try_send_discard_full_log_backpressure(&self, value: T) -> Result<(), SendError<T>> {
        let capacity = self.max_capacity();
        let len = capacity - self.capacity();
        if len > capacity / 2 {
            warn!(
                len = len,
                capacity = capacity,
                "High channel usage in mpsc try send."
            );
        }
        match self.try_send(value) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                warn!("Discarded value in mpsc try send.");
                Ok(())
            }
            Err(TrySendError::Closed(value)) => Err(SendError(value)),
        }
    }

    #[track_caller]
    fn reserve_many_log_backpressure(
        &'a self,
        size: usize,
    ) -> impl Future<Output = Result<PermitIterator<'a, T>, SendError<()>>> {
        let caller = Location::caller();
        reserve_many_log_backpressure_impl(self, size, caller)
    }
}

async fn send_log_backpressure_impl<T>(
    sender: &Sender<T>,
    value: T,
    caller: &'static Location<'static>,
) -> Result<(), SendError<T>> {
    let capacity = sender.max_capacity();
    let len = capacity - sender.capacity();
    if len > capacity / 2 {
        warn!(
            len = len,
            capacity = capacity,
            file = caller.file(),
            line = caller.line(),
            "High channel usage in mpsc send."
        );
    }
    sender.send(value).await
}

async fn reserve_many_log_backpressure_impl<'a, T: 'a>(
    sender: &'a Sender<T>,
    size: usize,
    caller: &'static Location<'static>,
) -> Result<PermitIterator<'a, T>, SendError<()>> {
    let capacity = sender.max_capacity();
    let len = capacity - sender.capacity();
    if len > capacity / 2 {
        warn!(
            len = len,
            capacity = capacity,
            file = caller.file(),
            line = caller.line(),
            "High channel usage in mpsc reserve many."
        );
    }
    sender.reserve_many(size).await
}
