use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use futures_core::Stream;
use futures_util::{Sink, SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::{self, Message};
use tracing::debug;

pub struct Tracing<S> {
    inner: S,
    id: String,
}

impl<S> Tracing<S> {
    pub fn new(inner: S, id: impl Into<String>) -> Self {
        Self {
            inner,
            id: id.into(),
        }
    }
}

impl<S> Stream for Tracing<S>
where
    S: Stream<Item = tungstenite::Result<Message>>
        + Sink<Message, Error = tungstenite::Error>
        + Unpin,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let item = ready!(this.inner.poll_next_unpin(cx));
        debug!(id = this.id, item = ?item, "Received websocket message.");
        Poll::Ready(item)
    }
}

impl<S> Sink<Message> for Tracing<S>
where
    S: Sink<Message> + Unpin,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().inner.poll_ready_unpin(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        let this = self.get_mut();
        debug!(id = this.id, item = ?item, "Sending websocket message.");
        this.inner.start_send_unpin(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().inner.poll_flush_unpin(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().inner.poll_close_unpin(cx)
    }
}
