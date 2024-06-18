pub mod appstate;
pub mod config;
pub mod entity;
pub mod result;
pub mod services;

use std::collections::BTreeMap;
use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;

use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use result::MainError;
use shared::height::BlockHeight;
use shared::indexed_tx::IndexedTx;
use shared::tx_index::TxIndex;
use tendermint_rpc::HttpClient;
use tokio::signal;
use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::RetryIf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::appstate::AppState;
use crate::config::AppConfig;
use crate::entity::chain_state::ChainState;
use crate::entity::tx_note_map::TxNoteMap;
use crate::result::{AsDbError, AsRpcError};
use crate::services::masp::update_witness_map;
use crate::services::{
    cometbft as cometbft_service, db as db_service, rpc as rpc_service,
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

    let app_state = AppState::new(config.database_url).into_db_error()?;

    let last_block_height = db_service::get_last_synched_block(
        app_state.get_db_connection().await.into_db_error()?,
    )
    .await
    .into_db_error()?
    .unwrap_or_default()
    .next();

    let commitment_tree = db_service::get_last_commitment_tree(
        app_state.get_db_connection().await.into_db_error()?,
        last_block_height,
    )
    .await
    .into_db_error()?
    .unwrap_or_default();

    let witness_map = db_service::get_last_witness_map(
        app_state.get_db_connection().await.into_db_error()?,
        last_block_height,
    )
    .await
    .into_db_error()?;

    for block_height in last_block_height.0.. {
        if must_exit(&exit_handle) {
            break;
        }

        _ = RetryIf::spawn(
            retry_strategy.clone(),
            || {
                let client = client.clone();
                let witness_map = witness_map.clone();
                let commitment_tree = commitment_tree.clone();
                let tx_note_map = TxNoteMap::default();
                let app_state = app_state.clone();

                async move {
                    let conn_obj = app_state
                        .clone()
                        .get_db_connection()
                        .await
                        .into_db_error()?;

                    let block_height = BlockHeight::from(block_height);
                    let chain_state = ChainState::new(block_height);

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

                    let tm_block_response_fut = async {
                        tracing::info!("Downloading new block...");
                        let tm_block_response = cometbft_service::query_block(
                            &client,
                            block_height,
                        )
                        .await
                        .into_rpc_error()?;
                        tracing::info!("Block downloaded!");
                        result::ok(tm_block_response)
                    };

                    let tm_block_response = tm_block_response_fut.await?;

                    let mut shielded_txs = BTreeMap::new();
                    let height = tm_block_response.header.height;

                    tracing::info!(
                        "Processing {} transactions...",
                        tm_block_response.transactions.len()
                    );

                    for (idx, batch_tx) in
                        tm_block_response.transactions.into_iter().enumerate()
                    {
                        for (masp_tx, _memo) in batch_tx.masp_tx {
                            // TODO: handle fee unshielding

                            let inner_tx = masp_tx.0;
                            let indexed_tx = IndexedTx {
                                height,
                                index: TxIndex(idx as u32),
                                is_fee_unshielding: false,
                            };

                            update_witness_map(
                                commitment_tree.clone(),
                                tx_note_map.clone(),
                                witness_map.clone(),
                                indexed_tx,
                                &inner_tx,
                            )
                            .map_err(MainError::Masp)?;

                            shielded_txs.insert(indexed_tx, inner_tx);
                        }

                        db_service::commit_masp(
                            &conn_obj,
                            chain_state,
                            commitment_tree.clone(),
                            witness_map.clone(),
                            tx_note_map.clone(),
                            shielded_txs.clone(),
                        )
                        .await
                        .into_db_error()?;

                        tracing::info!(
                            "Done committing block {}!",
                            block_height
                        );
                    }

                    db_service::commit_state(&conn_obj, chain_state)
                        .await
                        .into_db_error()?;
                    
                    tracing::info!("Done processing block height {}!", height);

                    Ok(())
                }
            },
            |_: &MainError| !must_exit(&exit_handle),
        )
        .await
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
