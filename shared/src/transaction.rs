use std::fmt::Display;

use namada_core::hash::Hash;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_tx::Tx as NamadaTx;

use crate::id::Id;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_txs: Vec<NamadaMaspTransaction>,
    pub fee_unshielding_tx: Option<NamadaMaspTransaction>, // TODO
}

impl Transaction {
    pub fn from_namada_tx(
        nam_tx_bytes: &[u8],
        masp_tx_sechashes: &[Hash],
    ) -> Option<Self> {
        let transaction = NamadaTx::try_from(nam_tx_bytes)
            .map_err(|e| e.to_string())
            .ok()?;
        let transaction_id = transaction.header_hash();

        // TODO: handle IBC masp?
        let masp_txs = masp_tx_sechashes
            .iter()
            .filter_map(|section_hash| {
                let masp_tx = transaction
                    .get_section(section_hash)
                    .and_then(|section| section.masp_tx())
                    .or_else(|| {
                        tracing::warn!(%transaction_id, "Failed to deserialize masp tx");
                        None
                    })?;
                Some(masp_tx)
            })
            .collect();

        Some(Transaction {
            masp_txs,
            hash: Id::from(transaction_id),
            fee_unshielding_tx: None,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash)
    }
}
