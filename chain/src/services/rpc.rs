use anyhow::Context;
use namada_sdk::queries::RPC;
use shared::height::BlockHeight;
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
