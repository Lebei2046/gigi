use std::env;
use std::path::PathBuf;

use gigi_logging::{init_logging_with_config, tracing::Level, LogConfig, LogOutput};

pub fn initialize() {
    // Get data directory
    let data_dir = env::var("GIGI_DATA_DIR").unwrap_or_else(|_| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gigi-dioxus")
            .to_string_lossy()
            .to_string()
    });

    // Expand ~ to home directory
    let data_dir_expanded = if data_dir.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            home.join(data_dir.strip_prefix('~').unwrap_or(""))
        } else {
            PathBuf::from(data_dir)
        }
    } else {
        PathBuf::from(data_dir)
    };

    let log_path = data_dir_expanded.join("gigi-dioxus.log");

    // Create parent directories if needed
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let config = LogConfig {
        output: LogOutput::Both(log_path.to_string_lossy().to_string()),
        level: Level::DEBUG,
        json: false,
        include_spans: false,
    };

    init_logging_with_config(config);
}

pub fn info<T: AsRef<str>>(message: T) {
    gigi_logging::info!("{}", message.as_ref());
}

pub fn warn<T: AsRef<str>>(message: T) {
    gigi_logging::warn!("{}", message.as_ref());
}

pub fn error<T: AsRef<str>>(message: T) {
    gigi_logging::error!("{}", message.as_ref());
}

pub fn debug<T: AsRef<str>>(message: T) {
    gigi_logging::debug!("{}", message.as_ref());
}
