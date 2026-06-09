use std::path::Path;

use tracing::subscriber::SetGlobalDefaultError;
use tracing_appender::non_blocking::WorkerGuard;
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

/// Initializes JSON logging to a daily-rolling file, written off-thread.
///
/// Returns a [`WorkerGuard`] that must be kept alive for the duration of the
/// program. Dropping the guard flushes and stops the background writer
/// thread; logs emitted afterwards are discarded.
#[cfg(tokio_unstable)]
pub fn init_rolling_file(directory: impl AsRef<Path>) -> Result<WorkerGuard, SetGlobalDefaultError> {
    use tracing_subscriber::layer::{Layer, SubscriberExt};

    // Write to the file from a dedicated thread so that emitting a log event
    // never blocks the calling (e.g. tokio worker) thread on file I/O.
    let (writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(directory, "log"));

    // Define the JSON logging layer with per-target filters.
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(writer)
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
    Ok(guard)
}

/// Initializes JSON logging to a daily-rolling file, written off-thread.
///
/// Returns a [`WorkerGuard`] that must be kept alive for the duration of the
/// program. Dropping the guard flushes and stops the background writer
/// thread; logs emitted afterwards are discarded.
#[cfg(not(tokio_unstable))]
pub fn init_rolling_file(directory: impl AsRef<Path>) -> Result<WorkerGuard, SetGlobalDefaultError> {
    // Write to the file from a dedicated thread so that emitting a log event
    // never blocks the calling (e.g. tokio worker) thread on file I/O.
    let (writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(directory, "log"));

    // Setup json logging to a file.
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var(EnvFilter::DEFAULT_ENV)
                .from_env_lossy(),
        )
        .with_writer(writer)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(guard)
}
