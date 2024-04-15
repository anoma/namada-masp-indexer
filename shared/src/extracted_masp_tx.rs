use std::collections::BTreeSet;

use namada_core::masp_primitives::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct ExtractedMaspTx {
    pub fee_unshielding: Option<(BTreeSet<String>, Transaction)>,
    pub inner_tx: Option<(BTreeSet<String>, Transaction)>,
}
