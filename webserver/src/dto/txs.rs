use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct TxQueryParams {
    #[validate(range(min = 1))]
    pub height: u64,
    #[validate(range(min = 0, max = 30))]
    pub height_offset: u64,
}
