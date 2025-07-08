pub mod appstate;
pub mod config;
pub mod entity;
pub mod services;

use std::collections::BTreeMap;
use std::env;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use namada_sdk::masp_primitives::transaction::Transaction as MaspTransaction;
use shared::error::{IntoMainError, MainError};
use shared::height::{BlockHeight, FollowingHeights};
use shared::indexed_tx::MaspIndexedTx;
use shared::retry;
use shared::transaction::Transaction;
use tendermint_rpc::HttpClient;
use tendermint_rpc::client::CompatMode;
use tokio::signal;
use tokio::time::{Instant, sleep};

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

    let (last_block_height, mut commitment_tree, mut witness_map) =
        load_committed_state(&app_state, starting_block_height).await?;

    let mut tx_notes_index = TxNoteMap::default();
    let mut shielded_txs = BTreeMap::new();

    let client = HttpClient::builder(cometbft_url.as_str().parse().unwrap())
        .compat_mode(CompatMode::V0_37)
        .build()
        .unwrap();
    let client = Arc::new(client);

    let retry_interval = Duration::from_millis(
        interval
            .map(|millis| millis * 1000)
            .unwrap_or(DEFAULT_INTERVAL * 1000),
    );

    let mut heights_to_process = FollowingHeights::after(last_block_height);

    while let Some(block_height) = heights_to_process
        .next_height(&client, retry_interval, || must_exit(&exit_handle))
        .await
        .into_rpc_error()?
    {
        if must_exit(&exit_handle) {
            break;
        }

        // Build and commit MASP data at the block height
        if let ControlFlow::Break(()) = retry::every(
            retry_interval,
            async || {
                let client = client.clone();
                let app_state = app_state.clone();
                let chain_state = ChainState::new(block_height);

                build_and_commit_masp_data_at_height(
                    block_height,
                    &exit_handle,
                    client,
                    &mut witness_map,
                    &mut commitment_tree,
                    &mut tx_notes_index,
                    &mut shielded_txs,
                    app_state,
                    chain_state,
                    number_of_witness_map_roots_to_check,
                )
                .await
            },
            async |_| {
                if must_exit(&exit_handle) {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )
        .await
        {
            break;
        }
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
    witness_map: &mut WitnessMap,
    commitment_tree: &mut CommitmentTree,
    tx_notes_index: &mut TxNoteMap,
    shielded_txs: &mut BTreeMap<MaspIndexedTx, MaspTransaction>,
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
    tx_notes_index.clear();
    shielded_txs.clear();

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

    tracing::info!(
        %block_height,
        num_transactions,
        "Processing new masp transactions...",
    );

    for (masp_indexed_tx, Transaction { masp_tx, .. }) in
        block_data.transactions.into_iter()
    {
        masp_service::update_witness_map(
            commitment_tree,
            tx_notes_index,
            witness_map,
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
        commitment_tree,
        witness_map,
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

        // SAFETY: Artificially extend the lifetime of `witness_map`
        // to be able to pass it to `tokio::task::spawn_blocking`.
        // The reference does not escape its intended scope,
        // therefore the `'static`, in spite of being
        // somewhat sketchy, is safe.
        let witness_map: &'static WitnessMap =
            unsafe { std::mem::transmute(witness_map) };

        let witness_map_check_fut = async move {
            tokio::task::spawn_blocking(move || {
                masp_service::query_witness_map_anchor_existence(
                    witness_map,
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
