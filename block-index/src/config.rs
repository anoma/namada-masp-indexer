use std::num::NonZeroU64;

use clap_verbosity_flag::{InfoLevel, LevelFilter, Verbosity};
use tracing::Level;
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::FmtSubscriber;

#[derive(clap::Parser)]
pub struct AppConfig {
    /// Link to the Postgres database
    #[clap(long, env)]
    pub database_url: String,

    /// How often (in seconds) a new block index is built
    #[clap(long, env)]
    pub interval: Option<NonZeroU64>,

    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}

pub fn install_tracing_subscriber(
    verbosity: Verbosity<InfoLevel>,
    non_blocking_logger: NonBlocking,
) {
    let log_level = match verbosity.log_level_filter() {
        LevelFilter::Off => None,
        LevelFilter::Error => Some(Level::ERROR),
        LevelFilter::Warn => Some(Level::WARN),
        LevelFilter::Info => Some(Level::INFO),
        LevelFilter::Debug => Some(Level::DEBUG),
        LevelFilter::Trace => Some(Level::TRACE),
    };
    if let Some(log_level) = log_level {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(log_level)
            .with_ansi(false)
            .with_writer(non_blocking_logger)
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}
