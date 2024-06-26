use shared::height::BlockHeight;

use crate::appstate::AppState;
use crate::repository::witness_map::{
    WitnessMapRepository, WitnessMapRepositoryTrait,
};

#[derive(Clone)]
pub struct WitnessMapService {
    witness_map_repo: WitnessMapRepository,
}

impl WitnessMapService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            witness_map_repo: WitnessMapRepository::new(app_state),
        }
    }

    pub async fn get_witnesses(
        &self,
        block_height: BlockHeight,
    ) -> Option<Vec<(Vec<u8>, u64)>> {
        self.witness_map_repo
            .get_witnesses(block_height.0 as i32)
            .await
            .ok()
            .map(|witnesses| {
                witnesses
                    .into_iter()
                    .map(|witness| {
                        (witness.witness_bytes, witness.witness_idx as u64)
                    })
                    .collect()
            })
            .and_then(|witnesses: Vec<(Vec<u8>, u64)>| {
                let non_empty_witnesses = !witnesses.is_empty();
                non_empty_witnesses.then_some(witnesses)
            })
    }
}
