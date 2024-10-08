#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(long, env, default_value = "5000")]
    pub port: u16,

    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub rps: Option<u64>,
}
