//! Structured logging and diagnostics.
//!
//! Initializes the `tracing` subscriber with configurable log levels,
//! JSON or human-readable output, and optional file output.

use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

/// Logging configuration.
pub struct LogConfig {
    /// Minimum log level. Defaults to INFO.
    pub level: Level,
    /// Output JSON instead of human-readable format.
    pub json: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json: false,
        }
    }
}

/// Initialize the global tracing subscriber.
///
/// Call this once at app startup. Respects `GATEORIX_LOG` env var
/// for filtering (e.g. `GATEORIX_LOG=debug` or `GATEORIX_LOG=gateorix=trace`).
///
/// # Examples
///
/// ```no_run
/// use gateorix_host_core::logging::{init, LogConfig};
///
/// init(LogConfig::default());
/// ```
pub fn init(config: LogConfig) {
    let env_filter = EnvFilter::try_from_env("GATEORIX_LOG")
        .unwrap_or_else(|_| {
            EnvFilter::new(format!(
                "gateorix={level},gateorix_host_core={level},{level}",
                level = config.level
            ))
        });

    if config.json {
        fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .init();
    } else {
        fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .init();
    }
}
