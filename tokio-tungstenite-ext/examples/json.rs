use std::{error::Error, time::Duration};

use futures_util::{Sink, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_tungstenite_ext::{WebSocketSinkExt, WebSocketStreamExt};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable stdout logging through tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .with_env_var(EnvFilter::DEFAULT_ENV)
                .from_env_lossy(),
        )
        .init();

    // Connect to the websocket server.
    let url = "wss://echo.websocket.org/";
    let id = "json";
    let (ws_stream, _) = connect_async(url).await?;
    let ws_stream = ws_stream
        // Enable this if you want input and output messages traced as debug
        // events.
        .with_tracing(id);
    let (ws_write, mut ws_read) = ws_stream.split();

    // Create a task that periodically sends json messages to the server.
    tokio::spawn(periodically_send_json(ws_write));

    // The server initially sends a non-json text message. Ignore it.
    let text = ws_read
        .next_text()
        .await
        .ok_or(anyhow::anyhow!("Stream closed."))??;
    info!("Received text: {text}");

    // Stream json messages.
    while let Some(counter) = ws_read.next_json::<Counter>().await {
        info!("Received count: {}", counter??.count);
    }

    Ok(())
}

async fn periodically_send_json(
    mut ws_write: impl Sink<Message, Error: Error + Send + Sync + 'static> + Unpin,
) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(3));
    let mut count = 0;
    loop {
        interval.tick().await;
        count += 1;
        ws_write.send_json(&Counter { count }).await??;
    }
}

#[derive(Serialize, Deserialize)]
struct Counter {
    count: usize,
}
