use anyhow::{Context, anyhow};
use namada_core::masp_primitives::sapling::Node;
use shared::block::Block;
use shared::height::BlockHeight;
use tendermint_rpc::endpoint::{block, block_results};
use tendermint_rpc::{Client, HttpClient};

pub async fn query_commitment_tree_anchor_existence(
    client: &HttpClient,
    commitment_tree_root: Node,
) -> anyhow::Result<bool> {
    let anchor_key = namada_sdk::token::storage_key::masp_commitment_anchor_key(
        commitment_tree_root,
    );

    namada_sdk::rpc::query_has_storage_key(client, &anchor_key)
        .await
        .context("Failed to check if commitment tree root is in storage")
}

pub async fn query_masp_txs_in_block(
    client: &HttpClient,
    height: BlockHeight,
) -> anyhow::Result<Block> {
    let (raw_block, raw_block_results) = futures::try_join!(
        query_raw_block(client, height),
        query_raw_block_results_at_height(client, height),
    )?;

    Block::new(raw_block, raw_block_results).map_err(|err| anyhow!(err))
}

async fn query_raw_block(
    client: &HttpClient,
    height: BlockHeight,
) -> anyhow::Result<block::Response> {
    client
        .block(height)
        .await
        .context("Failed to query CometBFT's last committed height")
}

async fn query_raw_block_results_at_height(
    client: &HttpClient,
    height: BlockHeight,
) -> anyhow::Result<block_results::Response> {
    client
        .block_results(height)
        .await
        .context("Failed to query CometBFT's block results")
}
