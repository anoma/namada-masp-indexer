use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct IndexQueryParams {
    #[validate(length(min = 1, max = 30))]
    pub indices: Vec<Index>,
}

#[derive(Copy, Clone, Serialize, Deserialize, Validate)]
pub struct Index {
    #[validate(range(min = 1))]
    pub height: u64,
    #[validate(range(min = 0))]
    pub block_index: u32,
}
