use anyhow::Context;
use namada_sdk::queries::RPC;
use shared::height::BlockHeight;
use tendermint_rpc::HttpClient;

pub async fn is_block_committed(
    client: &HttpClient,
    block_height: &BlockHeight,
) -> anyhow::Result<bool> {
    Ok(true)
}
