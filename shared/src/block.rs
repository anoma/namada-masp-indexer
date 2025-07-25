use std::collections::BTreeMap;
use std::fmt::Display;

use namada_sdk::state::TxIndex as NamadaTxIndex;
use namada_tx::Tx as NamadaTx;
use namada_tx::event::MaspEvent;
use tendermint_rpc::endpoint::{block, block_results};

use crate::block_results::locate_masp_txs;
use crate::header::BlockHeader;
use crate::height::BlockHeight;
use crate::id::Id;
use crate::indexed_tx::MaspIndexedTx;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub hash: Id,
    pub header: BlockHeader,
    pub transactions: BTreeMap<MaspIndexedTx, Transaction>,
}

impl Block {
    pub fn empty_block(block_height: BlockHeight) -> Self {
        let mut block = Self::default();
        block.header.height = block_height;
        block
    }

    pub fn new(
        raw_block: block::Response,
        raw_results: block_results::Response,
    ) -> Result<Self, String> {
        let indexed_masp_txs = locate_masp_txs(&raw_results)?;

        let mut block = Block {
            hash: Id::from(raw_block.block_id.hash),
            header: BlockHeader::from(raw_block.block.header),
            transactions: BTreeMap::new(),
        };

        // Cache the last tx seen to avoid multiple deserializations
        let mut last_tx: Option<(NamadaTx, NamadaTxIndex)> = None;

        for MaspEvent {
            tx_index,
            kind,
            data,
        } in indexed_masp_txs
        {
            let tx = match &last_tx {
                Some((tx, idx)) if idx == &tx_index.block_index => tx,
                _ => {
                    let tx = NamadaTx::try_from_bytes(
                        raw_block.block.data[tx_index.block_index.0 as usize]
                            .as_ref(),
                    )
                    .map_err(|e| e.to_string())?;
                    last_tx = Some((tx, tx_index.block_index));

                    &last_tx.as_ref().unwrap().0
                }
            };

            let tx = Transaction::from_namada_tx(tx, &data)?;

            block.transactions.insert(
                MaspIndexedTx {
                    kind: kind.into(),
                    indexed_tx: tx_index.into(),
                },
                tx,
            );
        }

        Ok(block)
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block hash: {}, Block height: {}, Transactions {:#?}",
            self.hash,
            self.header.height,
            self.transactions
                .iter()
                .map(|(masp_indexed_tx, tx)| {
                    format!(
                        "Hash: {}, Batch index: {}",
                        tx.hash, masp_indexed_tx.indexed_tx.masp_tx_index
                    )
                })
                .collect::<Vec<String>>()
        )
    }
}
