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
    Ok(true)
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
    Ok(serde_json::from_reader(std::fs::File::open("result-block.json")?)?)
}

async fn query_raw_block_results_at_height(
    client: &HttpClient,
    height: BlockHeight,
) -> anyhow::Result<block_results::Response> {
    Ok(serde_json::from_reader(std::fs::File::open("result-block_results.json")?)?)
}
