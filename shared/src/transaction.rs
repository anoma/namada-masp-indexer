use std::borrow::Cow;
use std::fmt::Display;

use namada_core::borsh::BorshDeserialize;
use namada_core::collections::HashMap;
use namada_core::masp::TxId;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_sdk::token::Transfer;
use namada_tx::{Data, Section, Tx as NamadaTx, TxCommitments};

use crate::id::Id;
use crate::tx_index::MaspTxIndex;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_txs: Vec<(MaspTxIndex, NamadaMaspTransaction)>,
}

impl Transaction {
    pub fn from_namada_tx(
        nam_tx_bytes: &[u8],
        valid_masp_tx_ids: &[TxId],
    ) -> Option<Self> {
        // TODO: handle IBC masp?

        let transaction = NamadaTx::try_from(nam_tx_bytes)
            .map_err(|e| e.to_string())
            .ok()?;
        let transaction_id = transaction.header_hash();

        let all_masp_txs: HashMap<_, _> = transaction
            .header
            .batch
            .iter()
            .enumerate()
            .filter_map(|(masp_tx_index, cmt)| {
                let masp_tx_id = get_shielded_tx_id(&transaction, cmt)?;
                Some((masp_tx_id, MaspTxIndex(masp_tx_index)))
            })
            .collect();

        let masp_txs = valid_masp_tx_ids
            .iter()
            .filter_map(|masp_tx_id| {
                let Some(masp_tx) = transaction.get_masp_section(masp_tx_id)
                else {
                    tracing::warn!(
                        %transaction_id,
                        ?masp_tx_id,
                        "Shielded tx not found in Namada transaction"
                    );
                    return None;
                };

                let masp_tx_index =
                    all_masp_txs.get(masp_tx_id).cloned().or_else(|| {
                        tracing::warn!(
                            %transaction_id,
                            ?masp_tx_id,
                            ?masp_tx,
                            "Shielded tx not found in Namada transaction"
                        );
                        None
                    })?;

                Some((masp_tx_index, masp_tx.clone()))
            })
            .collect();

        Some(Transaction {
            masp_txs,
            hash: Id::from(transaction_id),
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash)
    }
}

fn get_shielded_tx_id(
    transaction: &NamadaTx,
    cmt: &TxCommitments,
) -> Option<TxId> {
    let Some(Cow::Borrowed(Section::Data(Data { data: tx_data, .. }))) =
        transaction.get_section(&cmt.data_hash)
    else {
        return None;
    };

    Transfer::try_from_slice(tx_data)
        .ok()
        .and_then(|tx| tx.shielded_section_hash)
}
