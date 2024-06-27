use std::borrow::Cow;
use std::fmt::Display;

use namada_core::borsh::BorshDeserialize;
use namada_core::collections::HashMap;
use namada_core::hash::Hash;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_sdk::token::{
    ShieldedTransfer, ShieldingTransfer, UnshieldingTransfer,
};
use namada_tx::{Data, Section, Tx as NamadaTx, TxCommitments};

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
        valid_masp_tx_sechashes: &[Hash],
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
                let masp_tx_section_hash =
                    get_shielded_tx_sechash(&transaction, cmt)?;
                Some((masp_tx_section_hash, MaspTxIndex(masp_tx_index)))
            })
            .collect();

        let masp_txs = valid_masp_tx_sechashes
            .iter()
            .filter_map(|section_hash| {
                let Some(Cow::Borrowed(Section::MaspTx(masp_tx))) =
                    transaction.get_section(section_hash)
                else {
                    tracing::warn!(
                        %transaction_id,
                        %section_hash,
                        "Shielded tx not found in Namada transaction"
                    );
                    return None;
                };

                let masp_tx_index =
                    all_masp_txs.get(section_hash).cloned().or_else(|| {
                        tracing::warn!(
                            %transaction_id,
                            %section_hash,
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
            fee_unshielding_tx: None,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash)
    }
}

fn get_shielded_tx_sechash(
    transaction: &NamadaTx,
    cmt: &TxCommitments,
) -> Option<Hash> {
    let Some(Cow::Borrowed(Section::Data(Data { data: tx_data, .. }))) =
        transaction.get_section(&cmt.data_hash)
    else {
        return None;
    };

    let shielded_transfer = || {
        let data = ShieldedTransfer::try_from_slice(tx_data).ok()?;
        transaction.get_section(&data.section_hash)?;
        Some(data.section_hash)
    };
    let unshielding_transfer = || {
        let data = UnshieldingTransfer::try_from_slice(tx_data).ok()?;
        transaction.get_section(&data.shielded_section_hash)?;
        Some(data.shielded_section_hash)
    };
    let shielding_transfer = || {
        let data = ShieldingTransfer::try_from_slice(tx_data).ok()?;
        transaction.get_section(&data.shielded_section_hash)?;
        Some(data.shielded_section_hash)
    };

    shielded_transfer()
        .or_else(unshielding_transfer)
        .or_else(shielding_transfer)
}
