pub mod config;
pub mod result;
pub mod services;

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;

use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use namada_core::masp_primitives::merkle_tree::{
    CommitmentTree, IncrementalWitness,
};
use namada_core::masp_primitives::sapling::Node;
use result::MainError;
use shared::extracted_masp_tx::ExtractedMaspTx;
use shared::height::BlockHeight;
use shared::indexed_tx::IndexedTx;
use shared::tx_index::TxIndex;
use tendermint_rpc::HttpClient;
use tokio::signal;
use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::RetryIf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::config::AppConfig;
use crate::result::AsRpcError;
use crate::services::masp::{extract_masp_tx, update_witness_map};
use crate::services::{cometbft as cometbft_service, rpc as rpc_service};

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

    // TODO: Load up from persistence
    let last_block_height = 0;
    let mut witness_map = HashMap::<usize, IncrementalWitness<Node>>::default();
    let mut commitment_tree = CommitmentTree::<Node>::empty();
    let mut tx_note_map = BTreeMap::<IndexedTx, usize>::default();

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
                let mut shielded_txs = BTreeMap::new();
                let height = tm_block_response.header.height;
                for (idx, tx_event) in tm_block_results_response
                    .end_events
                    .into_iter()
                    .filter_map(|event| {
                        event
                            .attributes
                            .is_valid_masp_tx
                            .map(|ix| (ix as usize, event))
                    })
                {
                    let tx = &tm_block_response.transactions[idx];
                    let ExtractedMaspTx {
                        fee_unshielding,
                        inner_tx,
                    } = extract_masp_tx(tx, &tx_event);
                    fee_unshielding.and_then(|(_, masp_transaction)| {
                        let indexed_tx = IndexedTx {
                            height,
                            index: TxIndex(idx as u32),
                            is_fee_unshielding: true,
                        };
                        update_witness_map(
                            &mut commitment_tree,
                            &mut tx_note_map,
                            &mut witness_map,
                            indexed_tx,
                            &masp_transaction,
                        )?;
                        shielded_txs.insert(indexed_tx, masp_transaction)
                    });
                    inner_tx.and_then(|(_, masp_transaction)| {
                        let indexed_tx = IndexedTx {
                            height,
                            index: TxIndex(idx as u32),
                            is_fee_unshielding: true,
                        };
                        update_witness_map(
                            &mut commitment_tree,
                            &mut tx_note_map,
                            &mut witness_map,
                            indexed_tx,
                            &masp_transaction,
                        )?;
                        shielded_txs.insert(indexed_tx, masp_transaction)
                    });
                }
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
