pub mod appstate;
pub mod config;
pub mod entity;
pub mod services;

use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use shared::block::Block;
use shared::error::{IntoMainError, MainError};
use shared::height::{BlockHeight, FollowingHeights};
use shared::indexed_tx::IndexedTx;
use tendermint_rpc::HttpClient;
use tendermint_rpc::client::CompatMode;
use tokio::signal;
use tokio::time::sleep;
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
    rpc as rpc_service,
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

    for block_height in FollowingHeights::after(last_block_height) {
        if must_exit(&exit_handle) {
            break;
        }

        _ = RetryIf::spawn(
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
                )
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

async fn build_and_commit_masp_data_at_height(
    block_height: BlockHeight,
    exit_handle: &AtomicBool,
    client: Arc<HttpClient>,
    witness_map: WitnessMap,
    commitment_tree: CommitmentTree,
    app_state: AppState,
    chain_state: ChainState,
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

    if !rpc_service::is_block_committed(&client, &block_height)
        .await
        .into_rpc_error()?
    {
        tracing::warn!(
            %block_height,
            "Block was not processed, retrying..."
        );
        return Err(MainError);
    }

    let block_data = {
        tracing::info!(
            %block_height,
            "Fetching block data from CometBFT"
        );
        let block_data =
            cometbft_service::query_masp_txs_in_block(&client, block_height)
                .await
                .into_rpc_error()?;
        tracing::info!(
            %block_height,
            "Acquired block data from CometBFT"
        );
        block_data
    };

    let mut shielded_txs = Vec::new();
    let mut tx_notes_index = TxNoteMap::default();

    tracing::info!(
        %block_height,
        num_transactions = block_data.transactions.len(),
        "Processing new masp transactions...",
    );

    let ordered_txs =
        lookup_valid_commitment_tree(&client, &commitment_tree, &block_data)
            .await?;

    commitment_tree.rollback();

    for (new_masp_tx_index, mut indexed_tx) in
        ordered_txs.into_iter().enumerate()
    {
        let masp_tx = block_data.get_masp_tx(indexed_tx).unwrap();

        indexed_tx.masp_tx_index = new_masp_tx_index.into();

        masp_service::update_witness_map(
            &commitment_tree,
            &mut tx_notes_index,
            &witness_map,
            indexed_tx,
            masp_tx,
        )
        .into_masp_error()?;

        shielded_txs.push((indexed_tx, masp_tx.clone()));
    }

    query_witness_map_anchor_existence(&client, &witness_map).await?;

    db_service::commit(
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

async fn lookup_valid_commitment_tree(
    client: &HttpClient,
    commitment_tree: &CommitmentTree,
    block: &Block,
) -> Result<Vec<IndexedTx>, MainError> {
    use itertools::Itertools;

    let all_indexed_txs: Vec<_> = block.indexed_txs().collect();

    let mut correct_order = Vec::with_capacity(all_indexed_txs.len());
    let mut fee_unshields = HashSet::with_capacity(all_indexed_txs.len());

    // Guess the set of fee unshieldings at the current height
    let fee_unshield_sets = all_indexed_txs.iter().copied().powerset();

    for fee_unshield_set in fee_unshield_sets {
        // Start a new attempt at guessing the root of
        // the commitment tree
        commitment_tree.rollback();
        correct_order.clear();
        fee_unshields.clear();

        tracing::info!(
            ?fee_unshield_set,
            "Checking subset of masp fee unshields to build cmt tree"
        );

        for indexed_tx in fee_unshield_set {
            let masp_tx = block.get_masp_tx(indexed_tx).unwrap();

            masp_service::update_commitment_tree(commitment_tree, masp_tx)
                .into_masp_error()?;

            correct_order.push(indexed_tx);
            fee_unshields.insert(indexed_tx);
        }

        for indexed_tx in all_indexed_txs
            .iter()
            .copied()
            // We filter fee unshields out of this loop
            .filter(|indexed_tx| !fee_unshields.contains(indexed_tx))
        {
            let masp_tx = block.get_masp_tx(indexed_tx).unwrap();

            masp_service::update_commitment_tree(commitment_tree, masp_tx)
                .into_masp_error()?;

            correct_order.push(indexed_tx);
        }

        if cometbft_service::query_commitment_tree_anchor_existence(
            client,
            commitment_tree.root(),
        )
        .await
        .into_masp_error()?
        {
            return Ok(correct_order);
        }
    }

    Err(anyhow::anyhow!(
        "Couldn't find a valid permutation of fee unshieldings"
    ))
    .into_masp_error()
}

async fn query_witness_map_anchor_existence(
    client: &Arc<HttpClient>,
    witness_map: &WitnessMap,
) -> Result<(), MainError> {
    // NB: before we commit, let's check the roots
    // of the inner commitment trees in the witness map
    const NUMBER_OF_ROOTS_TO_CHECK: usize = 20;

    let witness_map_roots = witness_map.roots(NUMBER_OF_ROOTS_TO_CHECK);

    futures::future::try_join_all(witness_map_roots.into_iter().map(
        |(note_index, witness_map_root)| {
            let client = Arc::clone(client);
            async move {
                let exists =
                    cometbft_service::query_commitment_tree_anchor_existence(
                        &client,
                        witness_map_root,
                    )
                    .await?;

                Ok((note_index, exists))
            }
        },
    ))
    .await
    .into_rpc_error()?
    .into_iter()
    .try_for_each(|(note_index, anchor_exists)| {
        if !anchor_exists {
            anyhow::bail!("No anchor could be found for witness {note_index}");
        }
        Ok(())
    })
    .into_masp_error()?;

    Ok(())
}
