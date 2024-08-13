use crate::appstate::AppState;
use crate::service::namada_state::NamadaStateService;
use crate::service::notes_index::NotesIndexService;
use crate::service::tree::TreeService;
use crate::service::tx::TxService;
use crate::service::witness_map::WitnessMapService;

#[derive(Clone)]
pub struct CommonState {
    pub tree_service: TreeService,
    pub witness_map_service: WitnessMapService,
    pub notes_index_service: NotesIndexService,
    pub tx_service: TxService,
    pub namada_state_service: NamadaStateService,
}

impl CommonState {
    pub fn new(data: AppState) -> Self {
        Self {
            tree_service: TreeService::new(data.clone()),
            witness_map_service: WitnessMapService::new(data.clone()),
            notes_index_service: NotesIndexService::new(data.clone()),
            tx_service: TxService::new(data.clone()),
            namada_state_service: NamadaStateService::new(data),
        }
    }
}
