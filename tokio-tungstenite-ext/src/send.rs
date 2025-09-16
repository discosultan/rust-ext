use std::{marker::PhantomData, pin::Pin};

use futures_util::{
    Sink,
    future::Future,
    task::{Context, Poll},
};
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

/// Future for the [`send_json`](super::WebSocketSinkExt::send_json) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SendJson<'a, Si: ?Sized, T> {
    sink: &'a mut Si,
    item: Option<T>,
    phantom: PhantomData<T>,
}

impl<Si: Unpin + ?Sized, T> Unpin for SendJson<'_, Si, T> {}

impl<'a, Si: Sink<Message> + Unpin + ?Sized, T: Serialize> SendJson<'a, Si, T> {
    pub(super) fn new(sink: &'a mut Si, item: T) -> Self {
        Self {
            sink,
            item: Some(item),
            phantom: PhantomData,
        }
    }
}

impl<Si: Sink<Message> + Unpin + ?Sized, Item: Serialize> Future for SendJson<'_, Si, Item> {
    type Output = serde_json::Result<Result<(), Si::Error>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        let mut sink = Pin::new(&mut this.sink);
        if this.item.is_some() {
            if sink.as_mut().poll_ready(cx).is_pending() {
                return Poll::Pending;
            };
            let item = this.item.take().expect("polled after completion");
            let item = serde_json::to_string(&item)?;
            if let Err(err) = sink.as_mut().start_send(Message::Text(item.into())) {
                return Poll::Ready(Ok(Err(err)));
            };
        }
        if sink.poll_flush(cx).is_pending() {
            return Poll::Pending;
        }

        Poll::Ready(Ok(Ok(())))
    }
}
