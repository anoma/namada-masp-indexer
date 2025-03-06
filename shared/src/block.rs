use std::fmt::Display;

use namada_sdk::state::TxIndex as NamadaTxIndex;
use namada_tx::Tx as NamadaTx;
use namada_tx::event::MaspEvent;
use tendermint_rpc::endpoint::{block, block_results};

use crate::block_results::locate_masp_txs;
use crate::header::BlockHeader;
use crate::id::Id;
use crate::indexed_tx::IndexedTx;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub hash: Id,
    pub header: BlockHeader,
    pub transactions: Vec<(IndexedTx, Transaction)>,
}

impl Block {
    pub fn new(
        raw_block: block::Response,
        raw_results: block_results::Response,
    ) -> Result<Self, String> {
        let indexed_masp_txs = locate_masp_txs(&raw_results);

        let mut block = Block {
            hash: Id::from(raw_block.block_id.hash),
            header: BlockHeader::from(raw_block.block.header),
            transactions: Vec::with_capacity(raw_block.block.data.len()),
        };

        // Cache the last tx seen to avoid multiple deserializations
        let mut last_tx: Option<(NamadaTx, NamadaTxIndex)> = None;

        for MaspEvent {
            tx_index,
            kind: _,
            data,
        } in indexed_masp_txs
        {
            let tx = match &last_tx {
                Some((tx, idx)) if idx == &tx_index.index => tx,
                _ => {
                    let tx = NamadaTx::try_from_bytes(
                        raw_block.block.data[tx_index.index.0 as usize]
                            .as_ref(),
                    )
                    .map_err(|e| e.to_string())?;
                    last_tx = Some((tx, tx_index.index));

                    &last_tx.as_ref().unwrap().0
                }
            };

            let tx = Transaction::from_namada_tx(tx, &data)?;

            block.transactions.push((tx_index.into(), tx));
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
                .map(|(indexed_tx, tx)| format!(
                    "{}: batch index: {}",
                    tx, indexed_tx.masp_tx_index
                ))
                .collect::<Vec<String>>()
        )
    }
}
