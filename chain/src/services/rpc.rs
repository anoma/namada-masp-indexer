use anyhow::{anyhow, Context};
use namada_sdk::{queries::RPC, rpc};
use shared::{epoch::Epoch, height::BlockHeight};
use tendermint_rpc::HttpClient;

pub async fn is_block_committed(
    client: &HttpClient,
    block_height: &BlockHeight,
) -> anyhow::Result<bool> {
    let last_block = RPC
        .shell()
        .last_block(client)
        .await
        .context("Failed to query Namada's last committed block")?;

    Ok(last_block
        .map(|b| block_height.0 <= b.height.0)
        .unwrap_or(false))
}

pub async fn get_epoch_at_block_height(
    client: &HttpClient,
    block_height: BlockHeight,
) -> anyhow::Result<Epoch> {
    let epoch = rpc::query_epoch_at_height(client, block_height.into())
        .await
        .with_context(|| {
            format!("Failed to query Namada's epoch at height {block_height}")
        })?
        .ok_or_else(|| {
            anyhow!("No Namada epoch found for height {block_height}")
        })?;
    Ok(Epoch::from(epoch.0))
}
