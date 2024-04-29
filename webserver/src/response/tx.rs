use serde::{Deserialize, Serialize};
use shared::height::BlockHeight;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TxResponse {
    pub txs: Vec<Tx>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Tx {
    pub bytes: Vec<u8>,
    pub index: u64,
    pub block_height: u64,
}

impl TxResponse {
    pub fn new(txs: Vec<(Vec<u8>, u64, u64)>) -> Self {
        Self {
            txs: txs
                .into_iter()
                .map(|(bytes, index, block_height)| Tx {
                    bytes,
                    index,
                    block_height,
                })
                .collect(),
        }
    }
}
