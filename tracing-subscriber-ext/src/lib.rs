use std::path::Path;

use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

pub fn init_console() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var(EnvFilter::DEFAULT_ENV)
                .from_env_lossy(),
        )
        .init();
}

#[cfg(tokio_unstable)]
pub fn init_rolling_file(directory: impl AsRef<Path>) -> Result<(), SetGlobalDefaultError> {
    use tracing_subscriber::layer::{Layer, SubscriberExt};

    // Define the JSON logging layer with per-target filters.
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(tracing_appender::rolling::daily(directory, "log"))
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var(EnvFilter::DEFAULT_ENV)
                .from_env_lossy(),
        );

    // Create the console subscriber layer for tokio-console.
    let console_layer = console_subscriber::ConsoleLayer::builder().spawn();

    // Combine the layers into a subscriber.
    let subscriber = tracing_subscriber::registry()
        .with(console_layer)
        .with(json_layer);

    // Set the subscriber as the global default.
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[cfg(not(tokio_unstable))]
pub fn init_rolling_file(directory: impl AsRef<Path>) -> Result<(), SetGlobalDefaultError> {
    // Setup json logging to a file.
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var(EnvFilter::DEFAULT_ENV)
                .from_env_lossy(),
        )
        .with_writer(tracing_appender::rolling::daily(directory, "log"))
        .init();
    Ok(())
}
