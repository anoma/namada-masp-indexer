use std::fmt::Display;

use namada_core::storage::BlockHeight as NamadaBlockHeight;
use tendermint::block::Height;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockHeight(pub u64);

impl From<u64> for BlockHeight {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i32> for BlockHeight {
    fn from(value: i32) -> Self {
        Self(value as u64)
    }
}

impl From<BlockHeight> for Height {
    fn from(value: BlockHeight) -> Self {
        Height::from(value.0 as u32) // safe dont touch
    }
}

impl From<Height> for BlockHeight {
    fn from(value: Height) -> Self {
        Self(value.into())
    }
}

impl From<NamadaBlockHeight> for BlockHeight {
    fn from(value: NamadaBlockHeight) -> Self {
        Self(value.0)
    }
}

impl From<BlockHeight> for NamadaBlockHeight {
    fn from(value: BlockHeight) -> Self {
        NamadaBlockHeight(value.0)
    }
}

impl Display for BlockHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
