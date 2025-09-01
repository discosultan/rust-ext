use std::{
    pin::Pin,
    task::{Context, Poll, ready},
    time::Duration,
};

use futures_core::Stream;
use futures_util::{Sink, SinkExt, StreamExt};
use tokio::time::{self, Interval};
use tokio_tungstenite::tungstenite::{self, Bytes, Message};

enum State {
    Waiting,
    Scheduled,
    Flushing,
}

pub struct Heartbeat<S, F> {
    inner: S,
    interval: Interval,
    state: State,
    ping_factory: F,
}

impl<S, F> Heartbeat<S, F> {
    pub fn new(inner: S, interval: Duration, ping_factory: F) -> Self {
        let mut interval = time::interval(interval);
        // We reset because otherwise the interval ticks immediately.
        interval.reset();

        Self {
            inner,
            interval,
            ping_factory,
            state: State::Waiting,
        }
    }
}

impl<S, F> Stream for Heartbeat<S, F>
where
    S: Stream<Item = tungstenite::Result<Message>>
        + Sink<Message, Error = tungstenite::Error>
        + Unpin,
    F: PingFactory + Unpin,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // First check if the underlying stream has an item ready.
            if let Poll::Ready(item) = this.inner.poll_next_unpin(cx) {
                return Poll::Ready(item);
            }

            match this.state {
                State::Waiting => {
                    ready!(Pin::new(&mut this.interval).poll_tick(cx));
                    this.state = State::Scheduled;
                }
                State::Scheduled => {
                    ready!(this.inner.poll_ready_unpin(cx)?);
                    this.inner.start_send_unpin(this.ping_factory.create())?;
                    this.state = State::Flushing;
                }
                State::Flushing => {
                    ready!(this.inner.poll_flush_unpin(cx)?);
                    this.state = State::Waiting;
                }
            }
        }
    }
}

impl<S, F> Sink<Message> for Heartbeat<S, F>
where
    S: Sink<Message> + Unpin,
    F: Unpin,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().inner.poll_ready_unpin(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.get_mut().inner.start_send_unpin(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().inner.poll_flush_unpin(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().inner.poll_close_unpin(cx)
    }
}

pub trait PingFactory {
    fn create(&self) -> Message;
}

pub struct DefaultPingFactory {
    payload: Bytes,
}

impl DefaultPingFactory {
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Bytes::new())
    }

    #[must_use]
    pub fn new(payload: Bytes) -> Self {
        Self { payload }
    }
}

impl PingFactory for DefaultPingFactory {
    fn create(&self) -> Message {
        Message::Ping(self.payload.clone())
    }
}
