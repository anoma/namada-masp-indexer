use crate::appstate::AppState;
use crate::service::notes_map::NotesMapService;
use crate::service::tree::TreeService;
use crate::service::tx::TxService;
use crate::service::witness_map::WitnessMapService;

#[derive(Clone)]
pub struct CommonState {
    pub tree_service: TreeService,
    pub witness_map_service: WitnessMapService,
    pub notes_map_service: NotesMapService,
    pub tx_service: TxService,
}

impl CommonState {
    pub fn new(data: AppState) -> Self {
        Self {
            tree_service: TreeService::new(data.clone()),
            witness_map_service: WitnessMapService::new(data.clone()),
            notes_map_service: NotesMapService::new(data.clone()),
            tx_service: TxService::new(data),
        }
    }
}
