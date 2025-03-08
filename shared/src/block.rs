use std::fmt::Display;

use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_sdk::events::extend::IndexedMaspData;
use tendermint_rpc::endpoint::{block, block_results};

use crate::block_results::locate_masp_txs;
use crate::header::BlockHeader;
use crate::id::Id;
use crate::indexed_tx::IndexedTx;
use crate::transaction::Transaction;
use crate::tx_index::{MaspTxIndex, TxIndex};

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub hash: Id,
    pub header: BlockHeader,
    pub transactions: Vec<(usize, Transaction)>,
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

        for IndexedMaspData {
            tx_index,
            masp_refs,
        } in indexed_masp_txs
        {
            let block_index = tx_index.0 as usize;
            let tx_bytes = &raw_block.block.data[block_index];
            let tx = Transaction::from_namada_tx(tx_bytes, &masp_refs.0)?;

            block.transactions.push((block_index, tx));
        }

        block
            .transactions
            .sort_unstable_by_key(|(tx_index, _)| *tx_index);

        Ok(block)
    }

    pub fn get_masp_tx(
        &self,
        indexed_tx: IndexedTx,
    ) -> Option<&NamadaMaspTransaction> {
        #[cold]
        fn unlikely<T, F: FnOnce() -> T>(f: F) -> T {
            f()
        }

        if self.header.height != indexed_tx.block_height {
            return unlikely(|| None);
        }

        let found_at_index = self
            .transactions
            .binary_search_by_key(
                &indexed_tx.block_index,
                |(block_index, _)| TxIndex(*block_index as _),
            )
            .ok()?;

        let (_, transaction) = match self.transactions.get(found_at_index) {
            Some(tx) => tx,
            None => unreachable!(),
        };

        transaction.masp_txs.get(indexed_tx.batch_index)
    }

    pub fn indexed_txs(&self) -> impl Iterator<Item = IndexedTx> + '_ {
        self.transactions.iter().flat_map(
            |(block_index, Transaction { masp_txs, .. })| {
                (0..masp_txs.len()).map(|batch_index| IndexedTx {
                    block_height: self.header.height,
                    block_index: TxIndex(*block_index as _),
                    masp_tx_index: MaspTxIndex(usize::MAX),
                    batch_index,
                })
            },
        )
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
                .map(|(_, tx)| tx.to_string())
                .collect::<Vec<String>>()
        )
    }
}
