use anyhow::Result;
use std::env;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}

impl LogFormat {
    fn from_env() -> Self {
        match env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => Self::Json,
            "compact" => Self::Compact,
            _ => Self::Pretty,
        }
    }
}

fn init_pretty_logger(env_filter: EnvFilter) {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_ansi(true)
        .with_span_events(FmtSpan::CLOSE)
        .pretty();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

fn init_json_logger(env_filter: EnvFilter) {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

fn init_compact_logger(env_filter: EnvFilter) {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_ansi(true)
        .with_span_events(FmtSpan::CLOSE)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

pub fn init_logger() -> Result<()> {
    let log_format = LogFormat::from_env();

    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| {
        "info,tower_http=debug,axum=debug,hpyd_server=debug,hpyd_downloads=debug".to_string()
    });

    let env_filter = EnvFilter::try_new(&log_level)?;

    match log_format {
        LogFormat::Pretty => init_pretty_logger(env_filter),
        LogFormat::Json => init_json_logger(env_filter),
        LogFormat::Compact => init_compact_logger(env_filter),
    }

    tracing::info!(
        log_format = ?log_format,
        log_level = %log_level,
        "Logger initialized"
    );

    Ok(())
}

