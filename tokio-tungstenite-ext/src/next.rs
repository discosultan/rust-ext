use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use futures_core::{FusedFuture, FusedStream, Future, Stream};
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::{self, Bytes, Message, Utf8Bytes};

/// Future for the [`next_bin`](super::WebSocketStreamExt::next_bin) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Binary<'a, St: ?Sized> {
    stream: &'a mut St,
}

impl<St: ?Sized + Unpin> Unpin for Binary<'_, St> {}

impl<'a, St: ?Sized + Stream + Unpin> Binary<'a, St> {
    pub(super) fn new(stream: &'a mut St) -> Self {
        Self { stream }
    }
}

impl<St: ?Sized + FusedStream<Item = tungstenite::Result<Message>> + Unpin> FusedFuture
    for Binary<'_, St>
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St: ?Sized + Stream<Item = tungstenite::Result<Message>> + Unpin> Future for Binary<'_, St> {
    type Output = Option<tungstenite::Result<Bytes>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let item = ready!(self.stream.poll_next_unpin(cx));

        let Some(item) = item else {
            return Poll::Ready(None);
        };

        match item {
            Ok(item) => match item {
                Message::Binary(item) => Poll::Ready(Some(Ok(item))),
                _ => Self::poll(self, cx),
            },
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}

/// Future for the [`next_text`](super::WebSocketStreamExt::next_text) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Text<'a, St: ?Sized> {
    stream: &'a mut St,
}

impl<St: ?Sized + Unpin> Unpin for Text<'_, St> {}

impl<'a, St: ?Sized + Stream + Unpin> Text<'a, St> {
    pub(super) fn new(stream: &'a mut St) -> Self {
        Self { stream }
    }
}

impl<St: ?Sized + FusedStream<Item = tungstenite::Result<Message>> + Unpin> FusedFuture
    for Text<'_, St>
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St: ?Sized + Stream<Item = tungstenite::Result<Message>> + Unpin> Future for Text<'_, St> {
    type Output = Option<tungstenite::Result<Utf8Bytes>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let item = ready!(self.stream.poll_next_unpin(cx));

        let Some(item) = item else {
            return Poll::Ready(None);
        };

        match item {
            Ok(item) => match item {
                Message::Text(item) => Poll::Ready(Some(Ok(item))),
                _ => Self::poll(self, cx),
            },
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}
