use namada_sdk::state::TxIndex as NamadaTxIndex;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxIndex(pub u32);

impl From<NamadaTxIndex> for TxIndex {
    fn from(value: NamadaTxIndex) -> Self {
        Self(value.0)
    }
}

/// The order in which a masp tx appears in a Namada tx event.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaspTxIndex(pub usize);

impl From<usize> for MaspTxIndex {
    fn from(masp_tx_index: usize) -> Self {
        Self(masp_tx_index)
    }
}
