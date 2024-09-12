use std::fmt::Display;

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
    ) -> Self {
        let indexed_masp_txs = locate_masp_txs(&raw_results);

        let mut block = Block {
            hash: Id::from(raw_block.block_id.hash),
            header: BlockHeader::from(raw_block.block.header),
            transactions: Vec::with_capacity(raw_block.block.data.len()),
        };

        for (block_index, masp_tx_refs) in indexed_masp_txs.locations {
            let tx_bytes = &raw_block.block.data[block_index];

            let tx = match Transaction::from_namada_tx(tx_bytes, &masp_tx_refs)
            {
                Some(tx) => tx,
                None => {
                    tracing::warn!(
                        block_hash = %block.hash,
                        block_index,
                        "Invalid Namada transaction in block"
                    );
                    continue;
                }
            };

            block.transactions.push((block_index, tx));
        }

        block
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
