use std::{
    pin::Pin,
    task::{Context, Poll, ready},
    time::Duration,
};

use futures_util::{FutureExt, Sink, SinkExt, Stream, StreamExt, future::BoxFuture};
use tokio::time::{self, Interval, Sleep};
use tokio_tungstenite::{
    WebSocketStream, connect_async_with_config,
    tungstenite::{
        self, Message, client::IntoClientRequest, handshake::client::Response,
        protocol::WebSocketConfig,
    },
};
use tracing::debug;

enum State<S> {
    Waiting,
    Refreshing {
        connection: BoxFuture<'static, tungstenite::Result<(S, Response)>>,
    },
    Stitching {
        stream: S,
        sleep: Pin<Box<Sleep>>,
    },
}

pub struct Refreshing<S, C> {
    inner: S,
    interval: Interval,
    connector: C,
    id: String,
    state: State<S>,
}

impl<S, C> Refreshing<S, C> {
    pub fn new(inner: S, interval: Duration, connector: C, id: impl Into<String>) -> Self {
        let mut interval = time::interval(interval);
        // We reset because otherwise the interval ticks immediately.
        interval.reset();

        Self {
            inner,
            interval,
            connector,
            id: id.into(),
            state: State::Waiting,
        }
    }
}

impl<S, C> Stream for Refreshing<S, C>
where
    S: Stream<Item = tungstenite::Result<Message>>
        + Sink<Message, Error = tungstenite::Error>
        + Unpin,
    C: Connector<Connection = S> + Send + Unpin,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // Check if the underlying stream has an item ready.
            if let Poll::Ready(item) = this.inner.poll_next_unpin(cx) {
                return Poll::Ready(item);
            }

            match &mut this.state {
                State::Waiting => {
                    // Wait for the interval to tick.
                    ready!(Pin::new(&mut this.interval).poll_tick(cx));
                    let connection = this.connector.connect();
                    this.state = State::Refreshing { connection };
                    debug!(id = this.id, "Refreshing websocket connection.");
                }
                State::Refreshing { connection } => {
                    // Poll the connection future.
                    let (stream, _) = ready!(connection.poll_unpin(cx)?);
                    let sleep = tokio::time::sleep(Duration::from_secs(1));
                    this.state = State::Stitching {
                        stream,
                        sleep: Box::pin(sleep),
                    };
                    debug!(
                        id = this.id,
                        "New connection established but streaming still from old connection."
                    );
                }
                State::Stitching { stream, sleep } => {
                    // Wait for the sleep to complete.
                    ready!(sleep.as_mut().poll(cx));
                    std::mem::swap(&mut this.inner, stream);
                    this.state = State::Waiting;
                    debug!(
                        id = this.id,
                        "Switching over to stream from new connection."
                    );
                }
            }
        }
    }
}

impl<S, F> Sink<Message> for Refreshing<S, F>
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

pub trait Connector {
    type Connection;

    fn connect(&self) -> BoxFuture<'static, tungstenite::Result<(Self::Connection, Response)>>;
}

#[derive(Clone)]
pub struct DefaultConnector<R> {
    request: R,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
}

impl<R> DefaultConnector<R> {
    pub fn new(request: R) -> Self {
        Self::new_with_config(request, None, false)
    }

    pub fn new_with_config(
        request: R,
        config: Option<WebSocketConfig>,
        disable_nagle: bool,
    ) -> Self {
        Self {
            request,
            config,
            disable_nagle,
        }
    }
}

impl<R> Connector for DefaultConnector<R>
where
    R: IntoClientRequest + Unpin + Send + Clone + 'static,
{
    type Connection = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

    fn connect(&self) -> BoxFuture<'static, tungstenite::Result<(Self::Connection, Response)>> {
        connect_async_with_config(self.request.clone(), self.config, self.disable_nagle).boxed()
    }
}
