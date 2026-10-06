//! Logging to stderr and to a daily rolling file in the data directory.

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,naga=warn";

/// Initializes logging. Keep the returned guard alive until exit so buffered
/// file logs are flushed.
pub fn init(log_dir: Option<&Path>) -> Option<WorkerGuard> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let stderr = fmt::layer().with_writer(std::io::stderr).with_target(false);

    let (file_layer, guard) = match log_dir.map(|dir| (dir, std::fs::create_dir_all(dir))) {
        Some((dir, Ok(()))) => {
            let appender = tracing_appender::rolling::daily(dir, "truckpaint.log");
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let layer = fmt::layer().with_writer(writer).with_ansi(false);
            (Some(layer), Some(guard))
        }
        _ => (None, None),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(stderr)
        .with(file_layer)
        .init();
    guard
}
