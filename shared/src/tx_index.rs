use std::fmt::Display;

use namada_sdk::state::TxIndex as NamadaTxIndex;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxIndex(pub u32);

impl From<NamadaTxIndex> for TxIndex {
    fn from(value: NamadaTxIndex) -> Self {
        Self(value.0)
    }
}

/// The batch index in which a masp tx appears in a Namada tx event.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaspTxIndex(pub usize);

impl Display for MaspTxIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
