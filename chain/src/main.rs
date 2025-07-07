pub mod appstate;
pub mod config;
pub mod entity;
pub mod services;

use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use namada_sdk::queries::RPC;
use shared::error::{IntoMainError, MainError};
use shared::height::{BlockHeight, FollowingHeights};
use shared::transaction::Transaction;
use tendermint_rpc::HttpClient;
use tendermint_rpc::client::CompatMode;
use tokio::signal;
use tokio::time::{Instant, sleep};
use tokio_retry::RetryIf;
use tokio_retry::strategy::{FixedInterval, jitter};

use crate::appstate::AppState;
use crate::config::AppConfig;
use crate::entity::chain_state::ChainState;
use crate::entity::commitment_tree::CommitmentTree;
use crate::entity::tx_notes_index::TxNoteMap;
use crate::entity::witness_map::WitnessMap;
use crate::services::{
    cometbft as cometbft_service, db as db_service, masp as masp_service,
};

const VERSION_STRING: &str = env!("VERGEN_GIT_SHA");
const DEFAULT_INTERVAL: u64 = 5;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let AppConfig {
        cometbft_url,
        database_url,
        interval,
        verbosity,
        starting_block_height,
        number_of_witness_map_roots_to_check,
    } = AppConfig::parse();

    config::install_tracing_subscriber(verbosity);

    tracing::info!(version = VERSION_STRING, "Started the namada-masp-indexer");
    let exit_handle = must_exit_handle();

    let app_state = AppState::new(database_url).await.into_db_error()?;

    run_migrations(&app_state).await?;

    let (last_block_height, commitment_tree, witness_map) =
        load_committed_state(&app_state, starting_block_height).await?;

    let client = HttpClient::builder(cometbft_url.as_str().parse().unwrap())
        .compat_mode(CompatMode::V0_37)
        .build()
        .unwrap();
    let client = Arc::new(client);

    let internal = interval
        .map(|millis| millis * 1000)
        .unwrap_or(DEFAULT_INTERVAL * 1000);
    let retry_strategy = FixedInterval::from_millis(internal).map(jitter);

    let mut latest_known_block_height =
        last_block_height.unwrap_or(BlockHeight::from(0));

    for block_height in FollowingHeights::after(last_block_height) {
        if must_exit(&exit_handle) {
            break;
        }

        // If the latest known block height is less than the block height:
        // 1) We need to fetch the latest block height
        // 2) If the latest block height is less than the block height, we need to wait for the block to be committed
        if latest_known_block_height.0 < block_height.0 {
            let new_latest_block_height = RetryIf::spawn(
                retry_strategy.clone(),
                || need_to_wait_for_block(client.clone(), block_height),
                |_: &MainError| !must_exit(&exit_handle),
            )
            .await;

            // Update the latest known block height with the result
            if let Ok(latest_height) = new_latest_block_height {
                latest_known_block_height = latest_height;
            }

            tracing::info!(
                %latest_known_block_height,
                "Latest known block height"
            );
        }

        // Build and commit MASP data at the block height
        let _ = RetryIf::spawn(
            retry_strategy.clone(),
            || {
                let client = client.clone();
                let witness_map = witness_map.clone();
                let commitment_tree = commitment_tree.clone();
                let app_state = app_state.clone();
                let chain_state = ChainState::new(block_height);

                build_and_commit_masp_data_at_height(
                    block_height,
                    &exit_handle,
                    client,
                    witness_map,
                    commitment_tree,
                    app_state,
                    chain_state,
                    number_of_witness_map_roots_to_check,
                )
            },
            |_: &MainError| !must_exit(&exit_handle),
        )
        .await;
    }

    Ok(())
}

// returns latest block height if sufficiently new.
// Else returns an error, to signal to caller to retry
async fn need_to_wait_for_block(
    client: Arc<HttpClient>,
    block_height: BlockHeight,
) -> Result<BlockHeight, MainError> {
    let latest_known_block_height_option = RPC
        .shell()
        .last_block(&*client)
        .await
        .context("Failed to query Namada's last committed block")
        .into_rpc_error()?;
    match latest_known_block_height_option {
        Some(b) => {
            let new_latest_known_block_height: BlockHeight = b.height.into();
            let need_to_wait = block_height.0 > new_latest_known_block_height.0;
            if need_to_wait {
                Err(MainError)
            } else {
                Ok(new_latest_known_block_height)
            }
        }
        None => Err(MainError),
    }
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

async fn run_migrations(app_state: &AppState) -> Result<(), MainError> {
    let mut max_retries = env::var("DATABASE_MAX_MIGRATION_RETRY")
        .unwrap_or_else(|_| 5.to_string())
        .parse::<u64>()
        .unwrap_or(5_u64);
    loop {
        let migration_res = db_service::run_migrations(
            app_state.get_db_connection().await.into_db_error()?,
        )
        .await;

        match &migration_res {
            Ok(_) => {
                return migration_res
                    .context("Failed to run db migrations")
                    .into_db_error();
            }
            Err(e) => {
                tracing::debug!(
                    "Failed runnign migrations: {} ({}/5)",
                    e.to_string(),
                    max_retries
                );
                if max_retries == 0 {
                    return migration_res
                        .context("Failed to run db migrations")
                        .into_db_error();
                }
                max_retries -= 1;
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

async fn load_committed_state(
    app_state: &AppState,
    starting_block_height: Option<u64>,
) -> Result<(Option<BlockHeight>, CommitmentTree, WitnessMap), MainError> {
    tracing::info!("Loading last committed state from db...");

    let last_block_height = db_service::get_last_synced_block(
        app_state.get_db_connection().await.into_db_error()?,
    )
    .await
    .into_db_error()?;

    let last_block_height = std::cmp::max(
        last_block_height,
        starting_block_height.map(BlockHeight::from),
    );

    let commitment_tree = db_service::get_last_commitment_tree(
        app_state.get_db_connection().await.into_db_error()?,
    )
    .await
    .into_db_error()?
    .unwrap_or_default();

    let witness_map = db_service::get_last_witness_map(
        app_state.get_db_connection().await.into_db_error()?,
    )
    .await
    .into_db_error()?;

    let commitment_tree_len = commitment_tree.size();
    let witness_map_len = witness_map.size();

    if commitment_tree_len == 0 && witness_map_len != 0
        || commitment_tree_len != 0 && witness_map_len == 0
    {
        return Err(anyhow::anyhow!(
            "Invalid database state: Commitment tree size is \
             {commitment_tree_len}, and witness map size is {witness_map_len}"
        ))
        .into_db_error();
    }
    tracing::info!(?last_block_height, "Last state has been loaded");

    shared::error::ok((last_block_height, commitment_tree, witness_map))
}

#[allow(clippy::too_many_arguments)]
async fn build_and_commit_masp_data_at_height(
    block_height: BlockHeight,
    exit_handle: &AtomicBool,
    client: Arc<HttpClient>,
    witness_map: WitnessMap,
    commitment_tree: CommitmentTree,
    app_state: AppState,
    chain_state: ChainState,
    number_of_witness_map_roots_to_check: usize,
) -> Result<(), MainError> {
    if must_exit(exit_handle) {
        return Ok(());
    }

    // NB: rollback changes from previous failed commit attempts
    witness_map.rollback();
    commitment_tree.rollback();

    let conn_obj = app_state.get_db_connection().await.into_db_error()?;

    tracing::info!(
        %block_height,
        "Attempting to process new block"
    );

    let mut checkpoint = Instant::now();

    let (block_data, num_transactions) = {
        tracing::info!(
            %block_height,
            "Fetching block data from CometBFT"
        );
        let block_data =
            cometbft_service::query_masp_txs_in_block(&client, block_height)
                .await
                .into_rpc_error()?;
        with_time_taken(&mut checkpoint, |time_taken| {
            tracing::info!(
                time_taken,
                %block_height,
                "Acquired block data from CometBFT"
            );
        });
        let num_transactions = block_data.transactions.len();
        (block_data, num_transactions)
    };

    let mut shielded_txs = BTreeMap::new();
    let mut tx_notes_index = TxNoteMap::default();

    tracing::info!(
        %block_height,
        num_transactions,
        "Processing new masp transactions...",
    );

    for (masp_indexed_tx, Transaction { masp_tx, .. }) in
        block_data.transactions.into_iter()
    {
        masp_service::update_witness_map(
            &commitment_tree,
            &mut tx_notes_index,
            &witness_map,
            masp_indexed_tx,
            &masp_tx,
        )
        .into_masp_error()?;

        shielded_txs.insert(masp_indexed_tx, masp_tx);
    }

    with_time_taken(&mut checkpoint, |time_taken| {
        tracing::info!(
            %block_height,
            num_transactions,
            time_taken,
            "Processed new masp transactions",
        );
    });

    validate_masp_state(
        &mut checkpoint,
        &client,
        &commitment_tree,
        &witness_map,
        number_of_witness_map_roots_to_check,
    )
    .await?;

    db_service::commit(
        &mut checkpoint,
        &conn_obj,
        chain_state,
        commitment_tree,
        witness_map,
        tx_notes_index,
        shielded_txs,
    )
    .await
    .into_db_error()?;

    Ok(())
}

async fn validate_masp_state(
    checkpoint: &mut Instant,
    client: &HttpClient,
    commitment_tree: &CommitmentTree,
    witness_map: &WitnessMap,
    number_of_witness_map_roots_to_check: usize,
) -> Result<(), MainError> {
    if commitment_tree.is_dirty() && number_of_witness_map_roots_to_check > 0 {
        tracing::info!("Validating MASP state...");

        let tree_root = tokio::task::block_in_place(|| commitment_tree.root());

        let commitment_tree_check_fut = async {
            cometbft_service::query_commitment_tree_anchor_existence(
                client, tree_root,
            )
            .await
            .into_rpc_error()
        };

        let witness_map = witness_map.clone();
        let witness_map_check_fut = async move {
            tokio::task::spawn_blocking(move || {
                masp_service::query_witness_map_anchor_existence(
                    &witness_map,
                    tree_root,
                    number_of_witness_map_roots_to_check,
                )
                .into_masp_error()
            })
            .await
            .context("Failed to join Tokio task")
            .into_tokio_join_error()?
        };

        futures::try_join!(commitment_tree_check_fut, witness_map_check_fut)?;

        with_time_taken(checkpoint, |time_taken| {
            tracing::info!(time_taken, "Validated MASP state");
        });
    }

    Ok(())
}

fn with_time_taken<F, T>(checkpoint: &mut Instant, callback: F) -> T
where
    F: FnOnce(f64) -> T,
{
    let last_checkpoint = std::mem::replace(checkpoint, Instant::now());
    let time_taken = last_checkpoint.elapsed().as_secs_f64();

    callback(time_taken)
}
