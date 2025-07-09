use std::collections::BTreeMap;
use std::fmt::Display;
use std::ops::ControlFlow;
use std::time::Duration;

use anyhow::Context;
use namada_core::chain::BlockHeight as NamadaBlockHeight;
use tendermint::block::Height;
use tendermint_rpc::{Client, HttpClient};

use crate::block::Block;
use crate::retry;

#[derive(Debug)]
pub struct UnprocessedBlocks {
    next_height: BlockHeight,
    buffer: BTreeMap<BlockHeight, Block>,
}

impl UnprocessedBlocks {
    pub const fn new(last_committed_height: Option<BlockHeight>) -> Self {
        Self {
            next_height: match last_committed_height {
                Some(BlockHeight(h)) => BlockHeight(h + 1),
                None => BlockHeight(1),
            },
            buffer: BTreeMap::new(),
        }
    }

    pub fn next_to_process(&mut self, incoming_block: Block) -> Option<Block> {
        if self.next_height == incoming_block.header.height {
            self.increment_next_height();
            return Some(incoming_block);
        }

        self.buffer
            .insert(incoming_block.header.height, incoming_block);

        let can_process_buffer_head =
            *self.buffer.first_entry()?.key() == self.next_height;

        if can_process_buffer_head {
            self.increment_next_height();
            self.buffer.pop_first().map(|(_, block)| block)
        } else {
            None
        }
    }

    fn increment_next_height(&mut self) {
        self.next_height =
            self.next_height.next().expect("Block height overflow");
    }
}

pub struct FollowingHeights {
    iter_height: BlockHeight,
    last_committed_height: BlockHeight,
}

impl FollowingHeights {
    pub const fn after(last_height: Option<BlockHeight>) -> Self {
        let h = match last_height {
            Some(h) => h,
            None => BlockHeight(0),
        };

        Self {
            iter_height: h,
            last_committed_height: h,
        }
    }

    pub async fn next_height(
        &mut self,
        http_client: &HttpClient,
        fetch_retry_interval: Duration,
    ) -> Option<BlockHeight> {
        let next_height =
            self.iter_height.next().expect("Block height overflow");
        self.iter_height = next_height;

        // NB: the next height might not have been committed
        // yet, and we must block
        while next_height > self.last_committed_height {
            // NB: the compiler likes to complain like a little
            // bitch if we don't clone the http client
            let http_client = http_client.clone();

            let ControlFlow::Continue(block) =
                retry::every(fetch_retry_interval, async move || {
                    http_client.latest_block().await.context(
                        "Failed to query Namada's last committed block",
                    )
                })
                .await
            else {
                return None;
            };

            self.last_committed_height =
                BlockHeight(block.block.header.height.value());
        }

        debug_assert!(self.iter_height <= self.last_committed_height);

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
