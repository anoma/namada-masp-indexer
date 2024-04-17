use anyhow::Context;
use shared::block::Block;
use shared::block_results::BlockResult;
use shared::height::BlockHeight;
use tendermint_rpc::{Client, HttpClient};

pub async fn query_raw_block_at_height(
    client: &HttpClient,
    height: BlockHeight,
) -> anyhow::Result<Block> {
    client
        .block(height)
        .await
        .context("Failed to query CometBFT's last committed height")
        .map(Block::from)
}

pub async fn query_raw_block_results_at_height(
    client: &HttpClient,
    height: BlockHeight,
) -> anyhow::Result<BlockResult> {
    client
        .block_results(height)
        .await
        .context("Failed to query CometBFT's block results")
        .map(BlockResult::from)
}
