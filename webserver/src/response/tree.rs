use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TreeResponse {
    pub commitment_tree: Vec<u8>,
    pub block_height: u64,
}
