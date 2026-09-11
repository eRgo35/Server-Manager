//! Logging: single truncated `latest.log` in the config dir plus stderr (spec §9.4).

use std::sync::Mutex;

use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;

use crate::state::Paths;

/// Initializes the global subscriber once. Level comes from `SM_LOG`
/// (`trace|debug|info|warn|error`, default `info`).
pub fn init_logging(paths: &Paths) {
    let level = level_filter();
    // `File::create` truncates: latest.log is overwritten on every run.
    let file = match std::fs::File::create(&paths.log) {
        Ok(file) => file,
        Err(_) => {
            eprintln!(
                "warning: could not open {} for logging",
                paths.log.display()
            );
            tracing_subscriber::fmt().with_max_level(level).init();
            return;
        }
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(Mutex::new(file))
                .with_ansi(false)
                .with_filter(level),
        )
        .with(tracing_subscriber::fmt::layer().with_filter(level))
        .init();
}

fn level_filter() -> LevelFilter {
    match std::env::var("SM_LOG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    }
}
