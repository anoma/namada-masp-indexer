use crate::height::BlockHeight;
use crate::tx_index::{MaspTxIndex, TxIndex};

/// The type of a MASP transaction
#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub enum MaspTxKind {
    /// A MASP transaction used for fee payment
    FeePayment,
    /// A general MASP transfer
    #[default]
    Transfer,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaspIndexedTx {
    /// The masp tx kind, fee-payment or transfer
    pub kind: MaspTxKind,
    /// The pointer to the inner tx carrying this masp tx
    pub indexed_tx: IndexedTx,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexedTx {
    /// The block height of the indexed tx
    pub block_height: BlockHeight,
    /// The index pertaining to the order through
    /// which a masp tx is included in a transaction batch
    pub masp_tx_index: MaspTxIndex,
    /// The index in the block of the tx
    pub block_index: TxIndex,
}

impl From<namada_tx::IndexedTx> for IndexedTx {
    fn from(value: namada_tx::IndexedTx) -> Self {
        Self {
            block_height: value.height.0.into(),
            block_index: TxIndex(value.index.0),
            masp_tx_index: MaspTxIndex(value.batch_index.unwrap() as usize),
        }
    }
}

impl From<namada_tx::event::MaspEventKind> for MaspTxKind {
    fn from(value: namada_tx::event::MaspEventKind) -> Self {
        match value {
            namada_tx::event::MaspEventKind::FeePayment => {
                MaspTxKind::FeePayment
            }
            namada_tx::event::MaspEventKind::Transfer => MaspTxKind::Transfer,
        }
    }
}
