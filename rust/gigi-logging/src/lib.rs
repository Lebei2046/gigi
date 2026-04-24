//! Gigi Logging - Shared logging library for Gigi P2P ecosystem
//!
//! This library provides a consistent logging solution across all Gigi P2P components,
//! supporting flexible output options, log levels, and environment variable configuration.

use std::fs::File;
use std::path::Path;
use std::sync::Once;

use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, registry::Registry};

/// Log output destination
#[derive(Debug, Clone, PartialEq)]
pub enum LogOutput {
    /// Log to console
    Console,
    /// Log to file
    File(String),
    /// Log to both console and file
    Both(String),
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log output destination
    pub output: LogOutput,
    /// Log level
    pub level: Level,
    /// Whether to use JSON format
    pub json: bool,
    /// Whether to include span events
    pub include_spans: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            output: LogOutput::Console,
            level: Level::INFO,
            json: false,
            include_spans: false,
        }
    }
}

static INIT_ONCE: Once = Once::new();

/// Initialize logging with default configuration
///
/// Defaults to console output, INFO level, and text format.
///
/// # Examples
/// ```rust
/// use gigi_logging::init_logging;
///
/// init_logging();
/// ```
pub fn init_logging() {
    init_logging_with_config(LogConfig::default());
}

/// Initialize logging with custom configuration
///
/// # Examples
/// ```rust
/// use gigi_logging::{init_logging_with_config, LogConfig, LogOutput, Level};
///
/// let config = LogConfig {
///     output: LogOutput::File("gigi.log".to_string()),
///     level: Level::DEBUG,
///     json: true,
///     include_spans: true,
/// };
///
/// init_logging_with_config(config);
/// ```
pub fn init_logging_with_config(config: LogConfig) {
    INIT_ONCE.call_once(|| {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(format!("{} ,sqlx=warn", config.level)));

        // Create the subscriber based on config
        let subscriber: Box<dyn tracing::Subscriber + Send + Sync> = if config.json {
            match &config.output {
                LogOutput::File(path) | LogOutput::Both(path) => {
                    if let Ok(file) = File::create(Path::new(path)) {
                        Box::new(
                            tracing_subscriber::registry()
                                .with(env_filter)
                                .with(
                                    tracing_subscriber::fmt::layer()
                                        .with_level(true)
                                        .with_target(true)
                                        .with_thread_ids(true)
                                        .with_thread_names(true)
                                        .with_span_events(if config.include_spans {
                                            FmtSpan::ACTIVE | FmtSpan::CLOSE
                                        } else {
                                            FmtSpan::NONE
                                        })
                                        .json(),
                                )
                                .with(
                                    tracing_subscriber::fmt::layer()
                                        .with_writer(file)
                                        .with_level(true)
                                        .with_target(true)
                                        .with_thread_ids(true)
                                        .with_thread_names(true)
                                        .with_span_events(if config.include_spans {
                                            FmtSpan::ACTIVE | FmtSpan::CLOSE
                                        } else {
                                            FmtSpan::NONE
                                        })
                                        .json(),
                                ),
                        )
                    } else {
                        Box::new(
                            tracing_subscriber::registry().with(env_filter).with(
                                tracing_subscriber::fmt::layer()
                                    .with_level(true)
                                    .with_target(true)
                                    .with_thread_ids(true)
                                    .with_thread_names(true)
                                    .with_span_events(if config.include_spans {
                                        FmtSpan::ACTIVE | FmtSpan::CLOSE
                                    } else {
                                        FmtSpan::NONE
                                    })
                                    .json(),
                            ),
                        )
                    }
                }
                LogOutput::Console => Box::new(
                    tracing_subscriber::registry().with(env_filter).with(
                        tracing_subscriber::fmt::layer()
                            .with_level(true)
                            .with_target(true)
                            .with_thread_ids(true)
                            .with_thread_names(true)
                            .with_span_events(if config.include_spans {
                                FmtSpan::ACTIVE | FmtSpan::CLOSE
                            } else {
                                FmtSpan::NONE
                            })
                            .json(),
                    ),
                ),
            }
        } else {
            match &config.output {
                LogOutput::File(path) | LogOutput::Both(path) => {
                    if let Ok(file) = File::create(Path::new(path)) {
                        Box::new(
                            tracing_subscriber::registry()
                                .with(env_filter)
                                .with(
                                    tracing_subscriber::fmt::layer()
                                        .with_level(true)
                                        .with_target(true)
                                        .with_thread_ids(true)
                                        .with_thread_names(true)
                                        .with_span_events(if config.include_spans {
                                            FmtSpan::ACTIVE | FmtSpan::CLOSE
                                        } else {
                                            FmtSpan::NONE
                                        }),
                                )
                                .with(
                                    tracing_subscriber::fmt::layer()
                                        .with_writer(file)
                                        .with_level(true)
                                        .with_target(true)
                                        .with_thread_ids(true)
                                        .with_thread_names(true)
                                        .with_span_events(if config.include_spans {
                                            FmtSpan::ACTIVE | FmtSpan::CLOSE
                                        } else {
                                            FmtSpan::NONE
                                        }),
                                ),
                        )
                    } else {
                        Box::new(
                            tracing_subscriber::registry().with(env_filter).with(
                                tracing_subscriber::fmt::layer()
                                    .with_level(true)
                                    .with_target(true)
                                    .with_thread_ids(true)
                                    .with_thread_names(true)
                                    .with_span_events(if config.include_spans {
                                        FmtSpan::ACTIVE | FmtSpan::CLOSE
                                    } else {
                                        FmtSpan::NONE
                                    }),
                            ),
                        )
                    }
                }
                LogOutput::Console => Box::new(
                    tracing_subscriber::registry().with(env_filter).with(
                        tracing_subscriber::fmt::layer()
                            .with_level(true)
                            .with_target(true)
                            .with_thread_ids(true)
                            .with_thread_names(true)
                            .with_span_events(if config.include_spans {
                                FmtSpan::ACTIVE | FmtSpan::CLOSE
                            } else {
                                FmtSpan::NONE
                            }),
                    ),
                ),
            }
        };

        // Set the global default subscriber
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set up global tracing subscriber");

        // Redirect log crate logs to tracing
        tracing_log::LogTracer::init().expect("Failed to initialize log tracer");

        info!("Logging initialized with config: {:?}", config);
    });
}

/// Get logger for a specific module
///
/// # Examples
/// ```rust
/// use gigi_logging::get_logger;
///
/// let logger = get_logger!("gigi-dns");
/// logger.info("DNS service started");
/// ```
#[macro_export]
macro_rules! get_logger {
    ($name:expr) => {
        $crate::tracing::info_span!($name)
    };
}

/// Log a debug message
///
/// # Examples
/// ```rust
/// use gigi_logging::debug;
///
/// debug!("Processing request: {:?}", request);
/// ```
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {
        $crate::tracing::debug!($($args)*);
    };
}

/// Log an info message
///
/// # Examples
/// ```rust
/// use gigi_logging::info;
///
/// info!("Service started on port: {}", port);
/// ```
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        $crate::tracing::info!($($args)*);
    };
}

/// Log a warning message
///
/// # Examples
/// ```rust
/// use gigi_logging::warn;
///
/// warn!("Connection timeout for peer: {}", peer_id);
/// ```
#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => {
        $crate::tracing::warn!($($args)*);
    };
}

/// Log an error message
///
/// # Examples
/// ```rust
/// use gigi_logging::error;
///
/// error!("Failed to connect: {}", error);
/// ```
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        $crate::tracing::error!($($args)*);
    };
}

/// Log a trace message
///
/// # Examples
/// ```rust
/// use gigi_logging::trace;
///
/// trace!("Entering function: {}", function_name);
/// ```
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {
        $crate::tracing::trace!($($args)*);
    };
}

// Re-export tracing for macro usage and instrument attribute
pub use tracing;
pub use tracing::instrument;
