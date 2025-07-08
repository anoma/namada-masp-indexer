use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, anyhow};
use deadpool_diesel::postgres::Object;
use diesel::connection::DefaultLoadingMode as DbDefaultLoadingMode;
use diesel::dsl::max;
use diesel::{
    ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use diesel_migrations::{
    EmbeddedMigrations, MigrationHarness, embed_migrations,
};
use namada_sdk::borsh::{BorshDeserialize, BorshSerializeExt};
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use namada_sdk::masp_primitives::sapling::Node;
use namada_sdk::masp_primitives::transaction::Transaction;
use orm::schema::{self, chain_state, commitment_tree, witness};
use orm::tree::TreeDb;
use orm::tx::TxInsertDb;
use orm::witness::WitnessDb;
use shared::error::ContextDbInteractError;
use shared::height::BlockHeight;
use shared::indexed_tx::MaspIndexedTx;
use tokio::time::Instant;

use crate::entity::chain_state::ChainState;
use crate::entity::commitment_tree::CommitmentTree;
use crate::entity::tx_notes_index::TxNoteMap;
use crate::entity::witness_map::WitnessMap;
use crate::with_time_taken;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../orm/migrations/");

pub async fn run_migrations(conn: Object) -> anyhow::Result<()> {
    tracing::debug!("Running db migrations...");

    conn.interact(|transaction_conn| {
        transaction_conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| {
                anyhow!("Failed to run db migrations: {}", e.to_string())
            })?;
        anyhow::Ok(())
    })
    .await
    .context_db_interact_error()??;

    tracing::debug!("Finished running db migrations");

    Ok(())
}

pub async fn get_last_synced_block(
    conn: Object,
) -> anyhow::Result<Option<BlockHeight>> {
    tracing::debug!("Reading last synced height from db");

    let block_height = conn
        .interact(move |conn| {
            chain_state::dsl::chain_state
                .select(max(chain_state::dsl::block_height))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()??;

    let block_height = block_height.map(BlockHeight::from);
    tracing::debug!(?block_height, "Read last synced height from db");

    Ok(block_height)
}

pub async fn get_last_commitment_tree(
    conn: Object,
) -> anyhow::Result<Option<CommitmentTree>> {
    tracing::debug!("Reading last commitment tree from db");

    let maybe_tree = conn
        .interact(move |conn| {
            diesel::alias!(commitment_tree as commitment_tree_alias: CommitmentTreeAlias);

            let max_block_height = commitment_tree_alias
                .select(max(commitment_tree_alias.field(commitment_tree::dsl::block_height)))
                .single_value();

            let tree = commitment_tree::dsl::commitment_tree
                .filter(
                    commitment_tree::dsl::block_height
                        .nullable()
                        .eq(max_block_height),
                )
                .select(TreeDb::as_select())
                .first(conn)
                .optional()
                .context("Failed to read commitment tree from db")?;
            anyhow::Ok(tree)
        })
        .await
        .context_db_interact_error()??;

    tracing::debug!(
        present_in_db = maybe_tree.is_some(),
        "Read last commitment tree from db"
    );

    let maybe_tree = maybe_tree
        .map(|tree| {
            tracing::debug!("Deserializing commitment tree from db");
            let deserialized = tree.try_into().context(
                "Failed to deserialize commitment tree from db row data",
            )?;
            tracing::debug!("Deserialized commitment tree from db");
            tracing::trace!(commitment_tree = ?deserialized, "Commitment tree data");
            anyhow::Ok(deserialized)
        })
        .transpose()?;

    anyhow::Ok(maybe_tree)
}

pub async fn get_last_witness_map(conn: Object) -> anyhow::Result<WitnessMap> {
    tracing::debug!("Reading last witness map from db");

    let witnesses = conn
        .interact(move |conn| {
            diesel::alias!(witness as witness_alias: WitnessMapAlias);

            let max_block_height = witness_alias
                .select(max(witness_alias.field(witness::dsl::block_height)))
                .single_value();

            witness::dsl::witness
                .filter(
                    witness::dsl::block_height.nullable().eq(max_block_height),
                )
                .select(WitnessDb::as_select())
                .load_iter::<_, DbDefaultLoadingMode>(conn)
                .context("Failed to query note witnesses from db")?
                .try_fold(HashMap::new(), |mut accum, maybe_witness| {
                    tracing::debug!("Reading new witness map entry from db");
                    let witness = maybe_witness.context(
                        "Failed to get note witnesses row data from db",
                    )?;
                    tracing::debug!("Read new witness map entry from db");
                    let witness_node =
                        IncrementalWitness::<Node>::try_from_slice(
                            &witness.witness_bytes,
                        )
                        .context(
                            "Failed to deserialize note witness from db",
                        )?;
                    let note_index = usize::try_from(witness.witness_idx)
                        .with_context(|| {
                            let db_note_index = witness.witness_idx;
                            format!(
                                "Failed to convert note index {db_note_index} \
                                 from witnesses db table"
                            )
                        })?;
                    tracing::debug!("Deserialized new witness map entry");
                    tracing::trace!(
                        ?note_index,
                        ?witness_node,
                        "Witness map entry data"
                    );
                    accum.insert(note_index, witness_node);
                    tracing::trace!("Inserted data into witness map");
                    anyhow::Ok(accum)
                })
        })
        .await
        .context_db_interact_error()??;

    tracing::debug!("Read and deserialized witness map from db");

    Ok(WitnessMap::new(witnesses))
}

#[allow(clippy::too_many_arguments)]
pub fn commit(
    checkpoint: &mut Instant,
    conn: &Object,
    chain_state: ChainState,
    commitment_tree: &mut CommitmentTree,
    witness_map: WitnessMap,
    notes_index: TxNoteMap,
    shielded_txs: BTreeMap<MaspIndexedTx, Transaction>,
) -> anyhow::Result<()> {
    tracing::info!(
        block_height = %chain_state.block_height,
        "Beginning block commit"
    );

    tokio::task::block_in_place(|| {
        commit_inner(
            conn,
            chain_state,
            commitment_tree,
            witness_map,
            notes_index,
            shielded_txs,
        )
    })?;

    with_time_taken(checkpoint, |time_taken| {
        tracing::info!(
            block_height = %chain_state.block_height,
            time_taken,
            "Committed new block"
        );
    });

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn commit_inner(
    pool_conn: &Object,
    chain_state: ChainState,
    commitment_tree: &mut CommitmentTree,
    witness_map: WitnessMap,
    notes_index: TxNoteMap,
    shielded_txs: BTreeMap<MaspIndexedTx, Transaction>,
) -> anyhow::Result<()> {
    let mut conn = pool_conn
        .lock()
        .expect("Database connection pool mutex has been poisoned");

    conn.build_transaction()
        .read_write()
        .run(|transaction_conn| {
            if let Some(commitment_tree_db) =
                commitment_tree.into_db(chain_state.block_height)
            {
                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committing commitment tree"
                );

                diesel::insert_into(schema::commitment_tree::table)
                    .values(&commitment_tree_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert commitment tree into db")?;

                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committed commitment tree"
                );
            }

            if let Some(witness_map_db) =
                witness_map.into_db(chain_state.block_height)
            {
                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committing witness map"
                );

                diesel::insert_into(schema::witness::table)
                    .values(&witness_map_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert witness map into db")?;

                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committed witness map"
                );
            }

            if !notes_index.is_empty() {
                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committing notes map"
                );

                let notes_index_db = notes_index.into_db();
                diesel::insert_into(schema::notes_index::table)
                    .values(&notes_index_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert notes map into db")?;

                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committed notes map"
                );
            }

            if !shielded_txs.is_empty() {
                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committing shielded txs"
                );

                let shielded_txs_db = shielded_txs
                    .iter()
                    .map(|(MaspIndexedTx { kind, indexed_tx }, tx)| {
                        let is_masp_fee_payment = matches!(
                            kind,
                            shared::indexed_tx::MaspTxKind::FeePayment
                        );

                        TxInsertDb {
                            block_index: indexed_tx.block_index.0 as i32,
                            tx_bytes: tx.serialize_to_vec(),
                            block_height: indexed_tx.block_height.0 as i32,
                            masp_tx_index: indexed_tx.masp_tx_index.0 as i32,
                            is_masp_fee_payment,
                        }
                    })
                    .collect::<Vec<TxInsertDb>>();
                diesel::insert_into(schema::tx::table)
                    .values(&shielded_txs_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert shielded txs into db")?;

                tracing::debug!(
                    block_height = %chain_state.block_height,
                    "Pre-committed shielded txs"
                );
            }

            let chain_state_db = chain_state.into_db();
            diesel::insert_into(schema::chain_state::table)
                .values(&chain_state_db)
                .on_conflict(schema::chain_state::dsl::id)
                .do_update()
                .set(
                    schema::chain_state::block_height
                        .eq(chain_state_db.block_height),
                )
                .execute(transaction_conn)
                .context("Failed to insert last chain state into db")?;

            tracing::debug!(
                block_height = %chain_state.block_height,
                "All data was successfully pre-committed, committing..."
            );

            anyhow::Ok(())
        })
        .with_context(|| {
            format!(
                "Failed to commit block at height={}",
                chain_state.block_height
            )
        })?;

    Ok(())
}
