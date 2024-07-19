use clap_verbosity_flag::{InfoLevel, LevelFilter, Verbosity};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(long, env)]
    pub database_url: String,

    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}

pub fn install_tracing_subscriber(verbosity: Verbosity<InfoLevel>) {
    let log_level = match verbosity.log_level_filter() {
        LevelFilter::Off => None,
        LevelFilter::Error => Some(Level::ERROR),
        LevelFilter::Warn => Some(Level::WARN),
        LevelFilter::Info => Some(Level::INFO),
        LevelFilter::Debug => Some(Level::DEBUG),
        LevelFilter::Trace => Some(Level::TRACE),
    };
    if let Some(log_level) = log_level {
        let subscriber =
            FmtSubscriber::builder().with_max_level(log_level).finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}
