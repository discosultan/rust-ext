use std::time::Duration;

use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite_ext::{DefaultConnector, DefaultPingFactory, WebSocketStreamExt};
use tracing::{info, level_filters::LevelFilter, warn};
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

    // Connect to the websocket server and augment functionality.
    let url = "wss://echo.websocket.org/";
    let id = "echo";
    let (ws_stream, _) = connect_async(url).await?;
    let mut ws_stream = ws_stream
        // Enable this if you want to refresh the connection (reconnect)
        // periodically.
        .with_refreshing(Duration::from_secs(5), DefaultConnector::new(url), id)
        // Enable this if you want input and output messages traced as debug
        // events.
        .with_tracing(id)
        // Enable this if you want the stream to send ping messages to the
        // server periodically.
        .with_heartbeat(Duration::from_secs(1), DefaultPingFactory::empty());

    // Stream messages.
    while let Some(res) = ws_stream.next().await {
        info!("Received message: {:?}", res?);
    }

    warn!("Stream closed.");

    Ok(())
}
