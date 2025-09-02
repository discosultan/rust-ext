mod heartbeat;
mod next;
mod refreshing;
mod tracing;

use std::time::Duration;

use futures_core::Stream;
use serde::{Serialize, de::DeserializeOwned};
use tokio_tungstenite::tungstenite::{self, Message, Utf8Bytes};

pub use self::{heartbeat::*, next::*, refreshing::*, tracing::*};

pub trait WebSocketStreamExt {
    /// Periodically send ping messages with empty payload to the server.
    ///
    /// Note that this does not spawn any tasks. It is expected for the client
    /// to poll the stream for the functionality to take effect.
    fn with_heartbeat<F>(self, interval: Duration, ping_factory: F) -> Heartbeat<Self, F>
    where
        Self: Sized,
    {
        Heartbeat::new(self, interval, ping_factory)
    }

    /// Periodically reconnect to the server. During reconnection, duplicate
    /// messages may be received. It is up to the client to perform any
    /// deduplication if necessary.
    ///
    /// Note that this does not spawn any tasks. It is expected for the client
    /// to poll the stream for the functionality to take effect.
    fn with_refreshing<C>(
        self,
        interval: Duration,
        connector: C,
        id: impl Into<String>,
    ) -> Refreshing<Self, C>
    where
        Self: Sized,
    {
        Refreshing::new(self, interval, connector, id)
    }

    /// Trace request and response messages using [`tracing::debug!`].
    fn with_tracing(self, id: impl Into<String>) -> Tracing<Self>
    where
        Self: Sized,
    {
        Tracing::new(self, id)
    }

    /// Creates a future that resolves to the next [`Vec<u8>`] item in the
    /// stream.
    ///
    /// Note that because `next_bin` doesn't take ownership over the stream,
    /// the [`Stream`] type must be [`Unpin`]. If you want to use `next_bin`
    /// with a [`!Unpin`](Unpin) stream, you'll first have to pin the stream.
    /// This can be done by boxing the stream using [`Box::pin`] or pinning it
    /// to the stack using the `pin_mut!` macro from the `futures_util` crate.
    fn next_bin(&mut self) -> next::Binary<'_, Self>
    where
        Self: Unpin;

    /// Creates a future that resolves to the next [`String`] item in the
    /// stream.
    ///
    /// Note that because `next_text` doesn't take ownership over the stream,
    /// the [`Stream`] type must be [`Unpin`]. If you want to use `next_text`
    /// with a [`!Unpin`](Unpin) stream, you'll first have to pin the stream.
    /// This can be done by boxing the stream using [`Box::pin`] or pinning it
    /// to the stack using the `pin_mut!` macro from the `futures_util` crate.
    fn next_text(&mut self) -> next::Text<'_, Self>
    where
        Self: Unpin;
}

impl<S> WebSocketStreamExt for S
where
    S: Stream,
{
    fn next_bin(&mut self) -> next::Binary<'_, Self>
    where
        Self: Unpin,
    {
        next::Binary::new(self)
    }

    fn next_text(&mut self) -> next::Text<'_, Self>
    where
        Self: Unpin,
    {
        next::Text::new(self)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    #[error("Websocket stream closed.")]
    StreamClosed,
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Tunsgtenite(#[source] Box<tungstenite::Error>),
}

impl From<tungstenite::Error> for JsonError {
    fn from(err: tungstenite::Error) -> Self {
        JsonError::Tunsgtenite(Box::new(err))
    }
}

pub trait MessageStreamExt {
    /// Deserialize an instance of type `T` from a JSON
    /// [`tungstenite::Message`].
    ///
    /// # Errors
    ///
    /// This conversion can fail if either the websocket stream has closed,
    /// the result is [`tungstenite::Error`] or the underlying deserialization
    /// fails. See [`serde_json::from_str`] for more details regarding
    /// deserialization.
    fn json<T: DeserializeOwned>(self) -> Result<T, JsonError>;
}

impl MessageStreamExt for Option<tungstenite::Result<Utf8Bytes>> {
    fn json<T: DeserializeOwned>(self) -> Result<T, JsonError> {
        let Some(msg) = self else {
            return Err(JsonError::StreamClosed);
        };
        let value: T = serde_json::from_str(&msg?)?;
        Ok(value)
    }
}

pub trait MessageExt {
    /// Serialize the given data structure as a JSON [`tungstenite::Message`].
    ///
    /// # Errors
    ///
    /// This conversion can fail if the underlying serialization fails. See
    /// [`serde_json::to_string`] for more details.
    fn json<T: Serialize>(value: &T) -> Result<Message, serde_json::Error>;
}

impl MessageExt for Message {
    /// Create a new text WebSocket message from a json serializable.
    fn json<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        Ok(Message::Text(serde_json::to_string(value)?.into()))
    }
}
