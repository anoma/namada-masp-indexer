use crate::height::BlockHeight;
use crate::tx_index::{MaspTxIndex, TxIndex};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexedTx {
    /// The block height of the indexed tx
    pub block_height: BlockHeight,
    /// The index in the block of the tx
    pub block_index: TxIndex,
    /// The index pertaining to the order through
    /// which a masp tx is processed in Namada
    pub masp_tx_index: MaspTxIndex,
}
