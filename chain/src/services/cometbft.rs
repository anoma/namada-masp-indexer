use anyhow::Context;
use shared::block::Block;
use shared::height::BlockHeight;
use tendermint_rpc::{Client, HttpClient};

pub async fn query_block(
    client: &HttpClient,
    height: BlockHeight,
) -> anyhow::Result<Block> {
    client
        .block(height)
        .await
        .context("Failed to query CometBFT's last committed height")
        .map(Block::from)
}
