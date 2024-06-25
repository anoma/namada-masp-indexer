use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct LatestHeightResponse {
    pub block_height: u64,
}
