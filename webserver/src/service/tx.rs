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
    ) -> Vec<(Vec<u8>, u64, u64)> {
        self.tx_repo
            .get_txs(from_block_height as i32, to_block_height as i32)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|tx| {
                (tx.tx_bytes, tx.note_index as u64, tx.block_height as u64)
            })
            .collect()
    }
}
