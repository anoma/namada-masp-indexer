use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TxResponse {
    pub txs: Vec<Tx>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Tx {
    pub block_height: u64,
    pub block_index: u64,
    pub batch: Vec<TxSlot>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TxSlot {
    pub masp_tx_index: u64,
    pub bytes: Vec<u8>,
}

impl TxResponse {
    pub fn new(
        txs: impl IntoIterator<Item = (Vec<(u64, Vec<u8>)>, u64, u64)>,
    ) -> Self {
        Self {
            txs: txs
                .into_iter()
                .map(|(batch, block_height, block_index)| Tx {
                    batch: batch
                        .into_iter()
                        .map(|(masp_tx_index, bytes)| TxSlot {
                            masp_tx_index,
                            bytes,
                        })
                        .collect(),
                    block_height,
                    block_index,
                })
                .collect(),
        }
    }
}
