use crate::appstate::AppState;
use crate::service::tree::TreeService;

#[derive(Clone)]
pub struct CommonState {
    pub tree_service: TreeService,
}

impl CommonState {
    pub fn new(data: AppState) -> Self {
        Self {
            tree_service: TreeService::new(data),
        }
    }
}
