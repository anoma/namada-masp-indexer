use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct WitnessMapQueryParams {
    #[validate(range(min = 1))]
    pub height: u64,
    #[validate(range(min = 1, max = 30))]
    pub size: u64,
}
