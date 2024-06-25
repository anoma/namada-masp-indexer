use std::fmt::Display;

use tendermint_rpc::endpoint::block::Response as TendermintBlock;

use crate::header::BlockHeader;
use crate::id::Id;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub hash: Id,
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl From<TendermintBlock> for Block {
    fn from(value: TendermintBlock) -> Self {
        Block {
            hash: Id::from(value.block_id.hash),
            header: BlockHeader::from(value.block.header),
            transactions: value
                .block
                .data
                .iter()
                .filter_map(|tx_bytes| {
                    Transaction::try_from(tx_bytes.as_ref()).ok()
                })
                .collect(),
        }
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
                .map(|tx| tx.to_string())
                .collect::<Vec<String>>()
        )
    }
}

impl From<&TendermintBlock> for Block {
    fn from(value: &TendermintBlock) -> Self {
        Block::from(value.clone())
    }
}
