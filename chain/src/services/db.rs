use std::collections::BTreeMap;

use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::masp_primitives::transaction::Transaction;
use orm::schema::{self, chain_state};
use orm::tx::TxInsertDb;
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
) -> anyhow::Result<BlockHeight> {
    let block_height = conn
        .interact(move |conn| {
            chain_state::dsl::chain_state
                .select(max(chain_state::dsl::block_height))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(block_height
        .map(BlockHeight::from)
        .unwrap_or_else(|| BlockHeight::from(0)))
}

#[allow(clippy::too_many_arguments)]
pub async fn commit(
    conn: Object,
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
                let chain_state_db = chain_state.into_db();
                let commitment_tree_db =
                    commitment_tree.into_db(chain_state.block_height);
                let witness_map_db =
                    witness_map.into_db(chain_state.block_height);
                let notes_map_db = notes_map.into_db(chain_state.block_height);

                let shielded_txs_db = shielded_txs
                    .iter()
                    .map(|(index, tx)| TxInsertDb {
                        note_index: index.index.0 as i32,
                        tx_bytes: tx.serialize_to_vec(),
                        block_height: index.height.0 as i32,
                    })
                    .collect::<Vec<TxInsertDb>>();

                diesel::insert_into(schema::chain_state::table)
                    .values(&chain_state_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update crawler state in db")?;

                diesel::insert_into(schema::commitment_tree::table)
                    .values(&commitment_tree_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update crawler state in db")?;

                diesel::insert_into(schema::witness::table)
                    .values(&witness_map_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update crawler state in db")?;

                diesel::insert_into(schema::notes_map::table)
                    .values(&notes_map_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update crawler state in db")?;

                diesel::insert_into(schema::tx::table)
                    .values(&shielded_txs_db)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update crawler state in db")?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()?
    .context("Commit block db transaction error")
}
