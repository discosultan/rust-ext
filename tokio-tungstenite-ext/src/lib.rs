mod heartbeat;
mod next;
mod refreshing;
mod tracing;

use std::time::Duration;

use futures_util::Stream;
use tokio_tungstenite::tungstenite;

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

    #[cfg(feature = "serde")]
    fn next_json<T>(&mut self) -> next::Json<'_, Self, T>
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

    #[cfg(feature = "serde")]
    fn next_json<T>(&mut self) -> next::Json<'_, Self, T>
    where
        Self: Unpin,
    {
        next::Json::new(self)
    }
}

#[expect(
    clippy::result_large_err,
    reason = "We want to wrap `tunsgtenite::Error` which is large."
)]
pub trait ResultExt<T> {
    /// `tokio-tungstenite` stream yields `None` if the internal websocket error
    /// matches `tungstenite::Error::AlreadyClosed | tungstenite::Error::ConnectionClosed`.
    /// It represents a graceful exist. This fn removes the optionality by
    /// undoing the conversion back to `tungstenite::Error::AlreadyClosed`.
    fn ok_or_already_closed(self) -> tungstenite::Result<T>;

    /// `tokio-tungstenite` stream yields `None` if the internal websocket error
    /// matches `tungstenite::Error::AlreadyClosed | tungstenite::Error::ConnectionClosed`.
    /// It represents a graceful exist. This fn removes the optionality by
    /// undoing the conversion back to `tungstenite::Error::ConnectionClosed`.
    fn ok_or_connection_closed(self) -> tungstenite::Result<T>;
}

impl<T> ResultExt<T> for Option<tungstenite::Result<T>> {
    fn ok_or_already_closed(self) -> tungstenite::Result<T> {
        match self {
            Some(res) => res,
            None => Err(tungstenite::Error::AlreadyClosed),
        }
    }

    fn ok_or_connection_closed(self) -> tungstenite::Result<T> {
        match self {
            Some(res) => res,
            None => Err(tungstenite::Error::ConnectionClosed),
        }
    }
}

#[cfg(feature = "serde")]
pub use serde::*;

#[cfg(feature = "serde")]
mod serde {
    use ::serde::Serialize;
    use futures_util::{Sink, SinkExt, sink::Send};
    use tokio_tungstenite::tungstenite::Message;

    pub trait WebSocketSinkExt: Sink<Message> {
        /// Serializes `item` as json and returns a future that completes after the
        /// given item has been fully processed into the sink, including flushing.
        fn send_json<T: Serialize>(
            &mut self,
            item: T,
        ) -> serde_json::Result<Send<'_, Self, Message>>
        where
            Self: Unpin,
        {
            let msg = Message::json(&item)?;
            Ok(self.send(msg))
        }
    }

    impl<S> WebSocketSinkExt for S where S: Sink<Message> + ?Sized {}

    pub trait MessageExt {
        /// Serialize the given data structure as a JSON [`tungstenite::Message`].
        ///
        /// # Errors
        ///
        /// This conversion can fail if the underlying serialization fails. See
        /// [`serde_json::to_string`] for more details.
        fn json<T: Serialize>(value: &T) -> serde_json::Result<Message>;
    }

    impl MessageExt for Message {
        /// Create a new text WebSocket message from a json serializable.
        fn json<T: Serialize>(value: &T) -> serde_json::Result<Self> {
            Ok(Message::Text(serde_json::to_string(value)?.into()))
        }
    }
}
