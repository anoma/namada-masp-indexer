use serde::{Deserialize, Serialize};
use xorf::BinaryFuse16;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct LatestHeightResponse {
    pub block_height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockIndexResponse {
    pub block_height: u64,
    pub index: BinaryFuse16,
}
