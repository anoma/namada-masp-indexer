pub mod config;
pub mod result;
pub mod services;

use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use result::MainError;
use shared::height::BlockHeight;
use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};
use tendermint_rpc::HttpClient;
use tokio::signal;
use tokio_retry::{
    strategy::{jitter, FixedInterval},
    RetryIf,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{
    config::AppConfig, result::AsRpcError,
    services::cometbft as cometbft_service, services::rpc as rpc_service,
};

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    let log_level = match config.verbosity.log_level_filter() {
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

    tracing::info!("version: {}", env!("VERGEN_GIT_SHA").to_string());

    let client =
        Arc::new(HttpClient::new(config.tendermint_url.as_str()).unwrap());

    let retry_strategy = FixedInterval::from_millis(5000).map(jitter);
    let exit_handle = must_exit_handle();

    let last_block_height = 0;

    for block_height in last_block_height.. {
        if must_exit(&exit_handle) {
            break;
        }

        _ = RetryIf::spawn(
            retry_strategy.clone(),
            || async {
                let block_height = BlockHeight::from(block_height);

                tracing::info!(
                    "Attempting to process block: {}...",
                    block_height
                );

                if !rpc_service::is_block_committed(&client, &block_height)
                    .await
                    .into_rpc_error()?
                {
                    tracing::warn!(
                        "Block {} was not processed, retrying...",
                        block_height
                    );
                    return Err(MainError::Rpc);
                }

                tracing::info!("Querying epoch...");

                let epoch = rpc_service::get_epoch_at_block_height(
                    &client,
                    block_height,
                )
                .await
                .into_rpc_error()?;

                let tm_block_response_fut = async {
                    tracing::info!("Downloading new block...");
                    let tm_block_response =
                        cometbft_service::query_raw_block_at_height(
                            &client,
                            block_height,
                        )
                        .await
                        .into_rpc_error()?;
                    tracing::info!("Raw block downloaded!");
                    result::ok(tm_block_response)
                };

                let tm_block_results_response_fut = async {
                    tracing::info!("Query block results...");
                    let tm_block_results_response =
                        cometbft_service::query_raw_block_results_at_height(
                            &client,
                            block_height,
                        )
                        .await
                        .into_rpc_error()?;
                    tracing::info!("Block result downloaded!");
                    result::ok(tm_block_results_response)
                };

                let (tm_block_response, tm_block_results_response) = futures::try_join!(
                    tm_block_response_fut,
                    tm_block_results_response_fut,
                )?;

                Ok(())
            },
            |_: &MainError| !must_exit(&exit_handle),
        )
    }

    Ok(())
}

#[inline]
fn must_exit(handle: &AtomicBool) -> bool {
    handle.load(atomic::Ordering::Relaxed)
}

fn must_exit_handle() -> Arc<AtomicBool> {
    let handle = Arc::new(AtomicBool::new(false));
    let task_handle = Arc::clone(&handle);
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Error receiving interrupt signal");
        tracing::info!("Ctrl-c received");
        task_handle.store(true, atomic::Ordering::Relaxed);
    });
    handle
}
