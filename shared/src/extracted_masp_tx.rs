use std::collections::BTreeSet;

use namada_core::masp_primitives::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct ExtractedMaspTx {
    pub inner_tx: Option<(BTreeSet<String>, Transaction)>,
}
