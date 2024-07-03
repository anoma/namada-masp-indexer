use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct NotesMapQueryParams {
    #[validate(range(min = 1))]
    pub height: u64,
}
