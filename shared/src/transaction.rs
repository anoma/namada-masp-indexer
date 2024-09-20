use std::borrow::Cow;
use std::fmt::Display;

use namada_core::hash::Hash;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_sdk::events::extend::MaspTxRef;
use namada_sdk::token::Transfer;
use namada_tx::{Data, Section, Tx as NamadaTx};

use crate::id::Id;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_txs: Vec<NamadaMaspTransaction>,
}

impl Transaction {
    pub fn from_namada_tx(
        nam_tx_bytes: &[u8],
        valid_masp_tx_refs: &[MaspTxRef],
    ) -> Result<Self, String> {
        let transaction =
            NamadaTx::try_from(nam_tx_bytes).map_err(|e| e.to_string())?;
        let transaction_id = transaction.header_hash();

        let masp_txs = valid_masp_tx_refs.iter().try_fold(
            vec![],
            |mut acc, masp_tx_ref| {
                let masp_tx = match &masp_tx_ref {
                    MaspTxRef::MaspSection(masp_tx_id) => {
                        let masp_tx = transaction
                            .get_masp_section(masp_tx_id)
                            .ok_or_else(|| {
                                "Missing expected masp section with id: {id}"
                                    .to_string()
                            })?;
                        Cow::Borrowed(masp_tx)
                    }
                    MaspTxRef::IbcData(sechash) => {
                        let masp_tx =
                            get_masp_tx_from_ibc_data(&transaction, sechash)
                                .ok_or_else(|| {
                                    "Missing expected data section with hash: \
                                     {sechash}"
                                        .to_string()
                                })?;
                        Cow::Owned(masp_tx)
                    }
                };

                acc.push(masp_tx.into_owned());
                Result::<_, String>::Ok(acc)
            },
        )?;

        Ok(Transaction {
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
