use std::fmt::Display;

use namada_core::chain::BlockHeight as NamadaBlockHeight;
use tendermint::block::Height;

pub struct FollowingHeights(BlockHeight);

impl FollowingHeights {
    pub const fn after(last_height: Option<BlockHeight>) -> Self {
        match last_height {
            Some(h) => FollowingHeights(h),
            None => FollowingHeights(BlockHeight(0)),
        }
    }
}

impl Iterator for FollowingHeights {
    type Item = BlockHeight;

    fn next(&mut self) -> Option<Self::Item> {
        let next_height = self.0.next()?;
        self.0 = next_height;
        Some(next_height)
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockHeight(pub u64);

impl BlockHeight {
    pub fn next(&self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

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
