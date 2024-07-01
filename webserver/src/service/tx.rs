use itertools::Itertools;

use crate::appstate::AppState;
use crate::repository::tx::{TxRepository, TxRepositoryTrait};

#[derive(Clone)]
pub struct TxService {
    tx_repo: TxRepository,
}

impl TxService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            tx_repo: TxRepository::new(app_state),
        }
    }

    pub async fn get_txs(
        &self,
        from_block_height: u64,
        to_block_height: u64,
    ) -> anyhow::Result<impl IntoIterator<Item = (Vec<(u64, Vec<u8>)>, u64, u64)>>
    {
        Ok(self
            .tx_repo
            .get_txs(from_block_height as i32, to_block_height as i32)
            .await?
            .into_iter()
            // NB: the returned txs are guaranteed to be sorted
            // by their insertion order in the database, so
            // chunking should work as expected
            .chunk_by(|tx| {
                // NB: group batched txs by their slot in a block
                (tx.block_height, tx.block_index)
            })
            .into_iter()
            .map(|((block_height, block_index), tx_batch)| {
                let tx_batch: Vec<_> = tx_batch
                    .map(|tx| (tx.masp_tx_index as u64, tx.tx_bytes))
                    .collect();
                (tx_batch, block_height as u64, block_index as u64)
            })
            .collect::<Vec<_>>())
    }
}
