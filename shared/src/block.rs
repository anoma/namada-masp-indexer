use std::fmt::Display;

use namada_sdk::events::extend::IndexedMaspData;
use tendermint_rpc::endpoint::{block, block_results};

use crate::block_results::locate_masp_txs;
use crate::header::BlockHeader;
use crate::id::Id;
use crate::transaction::Transaction;

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
                .map(|(_, tx)| tx.to_string())
                .collect::<Vec<String>>()
        )
    }
}
