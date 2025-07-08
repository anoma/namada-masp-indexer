use std::fmt::Display;
use std::ops::ControlFlow;
use std::time::Duration;

use anyhow::Context;
use namada_core::chain::BlockHeight as NamadaBlockHeight;
use namada_sdk::queries::RPC;
use tendermint::block::Height;
use tendermint_rpc::HttpClient;

use crate::retry;

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
        mut must_exit: impl FnMut() -> bool,
    ) -> anyhow::Result<Option<BlockHeight>> {
        let next_height =
            self.iter_height.next().context("Block height overflow")?;
        self.iter_height = next_height;

        // NB: the next height might not have been committed
        // yet, and we must block
        while next_height > self.last_committed_height {
            let ControlFlow::Continue(block) = retry::every(
                fetch_retry_interval,
                async || {
                    RPC.shell()
                        .last_block(http_client)
                        .await
                        .context(
                            "Failed to query Namada's last committed block",
                        )
                        .and_then(|maybe_block| {
                            maybe_block
                                .context("No block has been committed yet")
                        })
                },
                async |_| {
                    if must_exit() {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
            )
            .await
            else {
                return Ok(None);
            };

            self.last_committed_height = block.height.into();
        }

        Ok(Some(next_height))
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
