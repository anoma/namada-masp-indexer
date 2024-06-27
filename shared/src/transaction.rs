use std::collections::HashMap;
use std::fmt::Display;

use namada_core::hash::Hash;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_tx::{Section, Tx as NamadaTx};

use crate::id::Id;
use crate::tx_index::MaspTxIndex;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_txs: Vec<(MaspTxIndex, NamadaMaspTransaction)>,
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

        let masp_tx_sections: HashMap<_, _> = transaction
            .sections
            .into_iter()
            .filter_map(|section| {
                if !matches!(&section, Section::MaspTx(_)) {
                    return None;
                };
                let sechash = section.get_hash();
                let Section::MaspTx(masp_tx) = section else {
                    unreachable!()
                };
                Some((sechash, masp_tx))
            })
            .collect();

        // TODO: handle IBC masp?
        let masp_txs = masp_tx_sechashes
            .iter()
            .enumerate()
            .filter_map(|(masp_tx_index, section_hash)| {
                masp_tx_sections
                    .get(section_hash)
                    .cloned()
                    .map(|masp_tx| (MaspTxIndex(masp_tx_index), masp_tx))
                    .or_else(|| {
                        tracing::warn!(
                            %transaction_id,
                            %section_hash,
                            %masp_tx_index,
                            "Shielded tx not found in Namada transaction"
                        );
                        None
                    })
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
