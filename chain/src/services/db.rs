use std::collections::{BTreeMap, HashMap};

use anyhow::{anyhow, Context};
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
use namada_sdk::borsh::{BorshDeserialize, BorshSerializeExt};
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use namada_sdk::masp_primitives::sapling::Node;
use namada_sdk::masp_primitives::transaction::Transaction;
use orm::schema::{self, chain_state, commitment_tree, witness};
use orm::tree::TreeDb;
use orm::tx::TxInsertDb;
use orm::witness::WitnessDb;
use shared::height::BlockHeight;
use shared::indexed_tx::IndexedTx;

use crate::entity::chain_state::ChainState;
use crate::entity::commitment_tree::CommitmentTree;
use crate::entity::tx_note_map::TxNoteMap;
use crate::entity::witness_map::WitnessMap;
use crate::result::ContextDbInteractError;

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("../orm/migrations/");

pub async fn run_migrations(conn: Object) -> anyhow::Result<()> {
    conn.interact(|transaction_conn| {
        transaction_conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(move |_| anyhow::anyhow!("Failed to run db migrations"))
            .map(|_| ())
    })
    .await
    .context_db_interact_error()??;

    Ok(())
}

pub async fn get_last_synched_block(
    conn: Object,
) -> anyhow::Result<Option<BlockHeight>> {
    let block_height = conn
        .interact(move |conn| {
            chain_state::dsl::chain_state
                .select(max(chain_state::dsl::block_height))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .unwrap_or(None);

    Ok(block_height
        .map(|height| Some(BlockHeight::from(height)))
        .unwrap_or(None))
}

pub async fn get_last_commitment_tree(
    conn: Object,
    height: BlockHeight,
) -> anyhow::Result<Option<CommitmentTree>> {
    let tree = conn
        .interact(move |conn| {
            commitment_tree::dsl::commitment_tree
                .filter(commitment_tree::dsl::block_height.eq(height.0 as i32))
                .select(TreeDb::as_select())
                .first(conn)
        })
        .await
        .context_db_interact_error()
        .context("Failed to read block max height in db")?
        .ok();

    if let Some(tree) = tree {
        if let Ok(tree) = CommitmentTree::try_from(tree) {
            Ok(Some(tree))
        } else {
            Err(anyhow!("Can't deserialize commitment tree"))
        }
    } else {
        Ok(None)
    }
}

pub async fn get_last_witness_map(
    conn: Object,
    height: BlockHeight,
) -> anyhow::Result<WitnessMap> {
    let witnesses: Vec<WitnessDb> = conn
        .interact(move |conn| {
            witness::dsl::witness
                .filter(witness::dsl::block_height.eq(height.0 as i32))
                .select(WitnessDb::as_select())
                .get_results(conn)
                .unwrap_or_default()
        })
        .await
        .context_db_interact_error()
        .context("Failed to read block max height in db")?;

    let witnesses = witnesses
        .into_iter()
        .try_fold(HashMap::new(), |mut acc, witness| {
            let witness_node = IncrementalWitness::<Node>::try_from_slice(
                &witness.witness_bytes,
            )?;
            acc.insert(witness.witness_idx as usize, witness_node);
            Ok::<_, std::io::Error>(acc)
        })
        .unwrap_or_default();

    Ok(WitnessMap::new(witnesses))
}

#[allow(clippy::too_many_arguments)]
pub async fn commit(
    conn: &Object,
    chain_state: ChainState,
    commitment_tree: CommitmentTree,
    witness_map: WitnessMap,
    notes_map: TxNoteMap,
    shielded_txs: BTreeMap<IndexedTx, Transaction>,
) -> anyhow::Result<()> {
    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                let commitment_tree_db =
                    commitment_tree.into_db(chain_state.block_height);
                diesel::insert_into(schema::commitment_tree::table)
                    .values(&commitment_tree_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert commitment tree into db")?;

                let witness_map_db =
                    witness_map.into_db(chain_state.block_height);
                diesel::insert_into(schema::witness::table)
                    .values(&witness_map_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert witness map into db")?;

                let notes_map_db = notes_map.into_db();
                diesel::insert_into(schema::notes_map::table)
                    .values(&notes_map_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert notes map into db")?;

                let shielded_txs_db = shielded_txs
                    .iter()
                    .map(|(index, tx)| TxInsertDb {
                        note_index: index.block_index.0 as i32,
                        tx_bytes: tx.serialize_to_vec(),
                        block_height: index.block_height.0 as i32,
                        masp_tx_index: index.masp_tx_index.0 as i32,
                    })
                    .collect::<Vec<TxInsertDb>>();
                diesel::insert_into(schema::tx::table)
                    .values(&shielded_txs_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert shielded txs into db")?;

                let chain_state_db = chain_state.into_db();
                diesel::insert_into(schema::chain_state::table)
                    .values(&chain_state_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert last chain state into db")?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()?
    .context("Commit block db transaction error")
}
