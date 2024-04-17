use crate::height::BlockHeight;
use crate::tx_index::TxIndex;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexedTx {
    /// The block height of the indexed tx
    pub height: BlockHeight,
    /// The index in the block of the tx
    pub index: TxIndex,
    /// A transaction can have up to two shielded transfers.
    /// This indicates if the wrapper contained a shielded transfer.
    pub is_fee_unshielding: bool,
}
