use serde::{Deserialize, Serialize};
use shared::height::BlockHeight;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct WitnessMapResponse {
    pub witnesses: Vec<Witness>,
    pub block_height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Witness {
    pub bytes: Vec<u8>,
    pub index: u64,
}

impl WitnessMapResponse {
    pub fn new(
        block_height: BlockHeight,
        witnesses: Vec<(Vec<u8>, u64)>,
    ) -> Self {
        Self {
            witnesses: witnesses
                .into_iter()
                .map(|(bytes, index)| Witness { bytes, index })
                .collect(),
            block_height: block_height.0,
        }
    }
}
