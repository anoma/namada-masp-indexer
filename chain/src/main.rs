pub mod appstate;
pub mod config;
pub mod entity;
pub mod services;

use std::collections::BTreeMap;
use std::env;
use std::future::poll_fn;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use namada_sdk::masp_primitives::transaction::Transaction as MaspTransaction;
use shared::block::Block;
use shared::client::Client;
use shared::error::{IntoMainError, MainError};
use shared::height::{BlockHeight, FollowingHeights, UnprocessedBlocks};
use shared::indexed_tx::MaspIndexedTx;
use shared::transaction::Transaction;
use shared::{exit_handle, retry};
use tendermint_rpc::HttpClient;
use tokio::signal;
use tokio::sync::{Semaphore, mpsc};
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
const DEFAULT_MAX_CONCURRENT_FETCHES: usize = 100;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let AppConfig {
        cometbft_url,
        database_url,
        interval,
        verbosity,
        starting_block_height,
        number_of_witness_map_roots_to_check,
        max_concurrent_fetches,
    } = AppConfig::parse();

    config::install_tracing_subscriber(verbosity);

    tracing::info!(version = VERSION_STRING, "Started the namada-masp-indexer");
    spawn_exit_handler();

    let app_state = AppState::new(database_url).await.into_db_error()?;

    run_migrations(&app_state).await?;

    let (last_block_height, mut commitment_tree, mut witness_map) =
        load_committed_state(&app_state, starting_block_height).await?;

    let mut tx_notes_index = TxNoteMap::default();
    let mut shielded_txs = BTreeMap::new();

    let client = Client::new(&cometbft_url);

    let retry_interval = Duration::from_millis(
        interval
            .map(|millis| millis * 1000)
            .unwrap_or(DEFAULT_INTERVAL * 1000),
    );

    let mut fetched_blocks = fetch_blocks_and_get_handle(
        last_block_height,
        max_concurrent_fetches,
        retry_interval,
        client.get(),
    );

    let mut unprocessed_blocks = UnprocessedBlocks::new(last_block_height);

    while let Some(block_data) =
        get_new_block_from_fetcher(&mut fetched_blocks).await
    {
        let received_block_height = block_data.header.height;

        // Sort the fetched block
        let Some(block_data) = unprocessed_blocks.next_to_process(block_data)
        else {
            tracing::info!(%received_block_height, "Queueing block to be processed");
            continue;
        };

        // Check if we can skip committing this block for now.
        // This is because the block is empty. We can make a
        // single remote procedure call to Postgres, when we
        // exit.
        if unprocessed_blocks.pre_commit_check_if_skip(&block_data) {
            tracing::info!(block_height = %block_data.header.height, "Skipping commit of empty block");
            continue;
        }

        tracing::info!(block_height = %block_data.header.height, "Dequeued block to be processed");

        // Build and commit MASP data at the block height
        if let ControlFlow::Break(()) =
            retry::every(retry_interval, async || {
                build_and_commit_masp_data_at_height(
                    block_data.clone(),
                    client.as_ref(),
                    &mut witness_map,
                    &mut commitment_tree,
                    &mut tx_notes_index,
                    &mut shielded_txs,
                    &app_state,
                    number_of_witness_map_roots_to_check,
                )
                .await
            })
            .await
        {
            break;
        }
    }

    if let Some(block_data) = unprocessed_blocks.finalize() {
        tracing::warn!(
            block_height = %block_data.header.height,
            "Single attempt at commiting data to db just before exiting"
        );

        // Make a feeble attempt at committing before exiting
        // for good
        build_and_commit_masp_data_at_height(
            block_data.clone(),
            client.as_ref(),
            &mut witness_map,
            &mut commitment_tree,
            &mut tx_notes_index,
            &mut shielded_txs,
            &app_state,
            number_of_witness_map_roots_to_check,
        )
        .await?;
    }

    Ok(())
}

fn fetch_blocks_and_get_handle(
    last_block_height: Option<BlockHeight>,
    max_concurrent_fetches: usize,
    retry_interval: Duration,
    client: HttpClient,
) -> mpsc::UnboundedReceiver<Block> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut heights_to_process = FollowingHeights::after(last_block_height);

        let sem = Arc::new(Semaphore::new(if max_concurrent_fetches == 0 {
            DEFAULT_MAX_CONCURRENT_FETCHES
        } else {
            max_concurrent_fetches
        }));

        while let Some(block_height) = heights_to_process
            .next_height(&client, retry_interval)
            .await
        {
            let permit = sem
                .clone()
                .acquire_owned()
                .await
                .expect("Failed to acquire semaphore handle");

            let client = client.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                let _permit = permit;

                let ControlFlow::Continue(block_data) =
                    retry::every(retry_interval, async move || {
                        let mut checkpoint = Instant::now();

                        tracing::info!(
                            %block_height,
                            "Fetching block data from CometBFT"
                        );

                        let block_data =
                            cometbft_service::query_masp_txs_in_block(
                                &client,
                                block_height,
                            )
                            .await?;

                        with_time_taken(&mut checkpoint, |time_taken| {
                            tracing::info!(
                                time_taken,
                                %block_height,
                                "Acquired block data from CometBFT"
                            );
                        });

                        anyhow::Ok(block_data)
                    })
                    .await
                else {
                    return;
                };

                match tx.send(block_data) {
                    Err(_) if exit_handle::must_exit() => {}
                    Err(err) => panic!(
                        "Block data consumer has terminated unexpectedly: \
                         {err}"
                    ),
                    _ => {}
                }
            });
        }
    });

    rx
}

fn spawn_exit_handler() {
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Error receiving interrupt signal");
        tracing::info!("Ctrl-c received");
        exit_handle::exit();
    });
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
                    "Failed running migrations: {} ({}/5)",
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
    block_data: Block,
    client: &HttpClient,
    witness_map: &mut WitnessMap,
    commitment_tree: &mut CommitmentTree,
    tx_notes_index: &mut TxNoteMap,
    shielded_txs: &mut BTreeMap<MaspIndexedTx, MaspTransaction>,
    app_state: &AppState,
    number_of_witness_map_roots_to_check: usize,
) -> Result<(), MainError> {
    // NB: rollback changes from previous failed commit attempts
    witness_map.rollback();
    commitment_tree.rollback();
    tx_notes_index.clear();
    shielded_txs.clear();

    let conn_obj = app_state.get_db_connection().await.into_db_error()?;

    let mut checkpoint = Instant::now();

    let num_transactions = block_data.transactions.len();
    let block_height = block_data.header.height;

    tracing::info!(
        %block_height,
        num_transactions,
        "Attempting to process new masp transactions..."
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
        client,
        commitment_tree,
        witness_map,
        number_of_witness_map_roots_to_check,
    )
    .await?;

    let chain_state = ChainState::new(block_data.header.height);

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

async fn get_new_block_from_fetcher(
    blocks: &mut mpsc::UnboundedReceiver<Block>,
) -> Option<Block> {
    poll_fn(|cx| {
        if exit_handle::must_exit() {
            return Poll::Ready(None);
        }

        match blocks.poll_recv(cx) {
            poll @ Poll::Ready(None) if exit_handle::must_exit() => poll,
            Poll::Ready(None) => {
                panic!("The block fetching task has unexpectedly terminated")
            }
            poll => poll,
        }
    })
    .await
}

fn with_time_taken<F, T>(checkpoint: &mut Instant, callback: F) -> T
where
    F: FnOnce(f64) -> T,
{
    let last_checkpoint = std::mem::replace(checkpoint, Instant::now());
    let time_taken = last_checkpoint.elapsed().as_secs_f64();

    callback(time_taken)
}
