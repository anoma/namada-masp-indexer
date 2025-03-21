use std::borrow::Cow;
use std::fmt::Display;

use namada_core::hash::Hash;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_sdk::token::Transfer;
use namada_tx::event::MaspTxRef;
use namada_tx::{Data, Section, Tx as NamadaTx};

use crate::id::Id;
use crate::indexed_tx::MaspIndexedTx;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_indexed_tx: MaspIndexedTx,
    pub masp_tx: NamadaMaspTransaction,
}

impl Transaction {
    pub fn from_namada_tx(
        transaction: &NamadaTx,
        masp_indexed_tx: MaspIndexedTx,
        valid_masp_tx_ref: &MaspTxRef,
    ) -> Result<Self, String> {
        let transaction_id = transaction.header_hash();

        let masp_tx = match &valid_masp_tx_ref {
            MaspTxRef::MaspSection(masp_tx_id) => transaction
                .get_masp_section(masp_tx_id)
                .ok_or_else(|| {
                    "Missing expected masp section with id: {id}".to_string()
                })?
                .to_owned(),
            MaspTxRef::IbcData(sechash) => get_masp_tx_from_ibc_data(
                transaction,
                sechash,
            )
            .ok_or_else(|| {
                "Missing expected data section with hash: {sechash}".to_string()
            })?,
        };

        Ok(Transaction {
            masp_tx,
            hash: Id::from(transaction_id),
            masp_indexed_tx,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Hash: {}, Batch index: {}",
            self.hash, self.masp_indexed_tx.indexed_tx.masp_tx_index
        )
    }
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
