use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(long, env)]
    pub cometbft_url: String,

    #[clap(long, env)]
    pub database_url: String,

    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}
