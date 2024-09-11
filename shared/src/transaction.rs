use std::borrow::Cow;
use std::fmt::Display;

use namada_core::borsh::BorshDeserialize;
use namada_core::collections::HashMap;
use namada_core::hash::Hash;
use namada_core::masp::TxId;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_sdk::token::Transfer;
use namada_tx::{Data, Section, Tx as NamadaTx, TxCommitments};

use crate::block_results::IndexedMaspTxRef;
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
        valid_masp_tx_refs: &[IndexedMaspTxRef],
    ) -> Option<Self> {
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

        let masp_txs = valid_masp_tx_refs
            .iter()
            .filter_map(|masp_tx_ref| {
                let masp_tx = match masp_tx_ref {
                    IndexedMaspTxRef::TxId(masp_tx_id) => {
                        let Some(masp_tx) = transaction.get_masp_section(masp_tx_id)
                        else {
                            tracing::warn!(
                                %transaction_id,
                                ?masp_tx_id,
                                "Shielded tx not found in Namada transaction"
                            );
                            return None;
                        };
                        Cow::Borrowed(masp_tx)
                    }
                    IndexedMaspTxRef::IbcEnvelopeDataSecHash(sechash) => {
                        let Some(masp_tx) = get_masp_tx_from_ibc_data(&transaction, sechash) else {
                            tracing::warn!(
                                %transaction_id,
                                ibc_data_sechash = ?sechash,
                                "IBC shielding tx not found in Namada transaction"
                            );
                            return None;
                        };
                        Cow::Owned(masp_tx)
                    }
                };

                let masp_tx_index =
                    all_masp_txs.get(&TxId::from(masp_tx.txid())).cloned().or_else(|| {
                        tracing::warn!(
                            %transaction_id,
                            ?masp_tx,
                            "Shielded tx not found in Namada transaction"
                        );
                        None
                    })?;

                Some((masp_tx_index, masp_tx.into_owned()))
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
    let tx_data = get_namada_tx_data(transaction, &cmt.data_hash)?;

    Transfer::try_from_slice(tx_data)
        .ok()
        .and_then(|tx| tx.shielded_section_hash)
        .or_else(|| {
            get_masp_tx_from_ibc_data(transaction, &cmt.data_hash)
                .map(|tx| TxId::from(tx.txid()))
        })
}

fn get_masp_tx_from_ibc_data(
    transaction: &NamadaTx,
    data_sechash: &Hash,
) -> Option<NamadaMaspTransaction> {
    let tx_data = get_namada_tx_data(transaction, data_sechash)?;

    let ibc_msg = namada_sdk::ibc::decode_message::<Transfer>(tx_data).ok()?;
    let namada_sdk::ibc::IbcMessage::Envelope(envelope) = ibc_msg else {
        return None;
    };

    namada_sdk::ibc::extract_masp_tx_from_envelope(&envelope)
}

fn get_namada_tx_data<'tx>(
    transaction: &'tx NamadaTx,
    data_sechash: &'tx Hash,
) -> Option<&'tx [u8]> {
    if let Some(Cow::Borrowed(Section::Data(Data { data: tx_data, .. }))) =
        transaction.get_section(data_sechash)
    {
        Some(tx_data.as_slice())
    } else {
        None
    }
}
