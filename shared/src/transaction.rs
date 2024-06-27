use std::fmt::Display;

use namada_core::borsh::BorshDeserialize;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_sdk::masp::ShieldedTransfer;
use namada_sdk::token::{ShieldingTransfer, UnshieldingTransfer};
use namada_tx::Tx as NamadaTx;

use crate::block_results::TxEventStatusCode;
use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionExitStatus {
    Applied,
    Rejected,
}

impl Display for TransactionExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Applied => write!(f, "Applied"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

impl From<TxEventStatusCode> for TransactionExitStatus {
    fn from(value: TxEventStatusCode) -> Self {
        match value {
            TxEventStatusCode::Ok => Self::Applied,
            TxEventStatusCode::Fail => Self::Rejected,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MaspTxKind {
    ShieldedTransfer,
    ShieldingTransfer,
    UnshieldingTransfer,
}

#[derive(Debug, Clone)]
pub struct MaspTx {
    pub masp_tx: NamadaMaspTransaction,
    pub tx_memo: Option<Vec<u8>>,
    pub kind: MaspTxKind,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_txs: Vec<MaspTx>,
    pub fee_unshielding_tx: Option<MaspTx>, // TODO
}

impl TryFrom<&[u8]> for Transaction {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let transaction = NamadaTx::try_from(value).map_err(|e| e.to_string());

        let transaction = if let Ok(transaction) = transaction {
            transaction
        } else {
            return Err("Invalid tx".to_string());
        };

        let transaction_id = transaction.header_hash();

        // TODO: handle IBC masp
        let masp_txs = transaction
            .header()
            .batch
            .into_iter()
            .filter_map(|tx_commitment| {
                let tx_data = transaction.data(&tx_commitment)?;

                let shielded_transfer = || {
                    let data =
                        ShieldedTransfer::try_from_slice(&tx_data).ok()?;
                    let tx_memo = transaction.memo(&tx_commitment);
                    Some(MaspTx {
                        tx_memo,
                        masp_tx: data.masp_tx,
                        kind: MaspTxKind::ShieldedTransfer,
                    })
                };
                let unshielding_transfer = || {
                    let data =
                        UnshieldingTransfer::try_from_slice(&tx_data).ok()?;
                    let masp_tx = transaction
                        .get_section(&data.shielded_section_hash)
                        .and_then(|s| s.masp_tx())?;
                    let tx_memo = transaction.memo(&tx_commitment);
                    Some(MaspTx {
                        masp_tx,
                        tx_memo,
                        kind: MaspTxKind::UnshieldingTransfer,
                    })
                };
                let shielding_transfer = || {
                    let data =
                        ShieldingTransfer::try_from_slice(&tx_data).ok()?;
                    let masp_tx = transaction
                        .get_section(&data.shielded_section_hash)
                        .and_then(|s| s.masp_tx())?;
                    let tx_memo = transaction.memo(&tx_commitment);
                    Some(MaspTx {
                        masp_tx,
                        tx_memo,
                        kind: MaspTxKind::ShieldingTransfer,
                    })
                };

                shielded_transfer()
                    .or_else(unshielding_transfer)
                    .or_else(shielding_transfer)
                //.or_else(|| {
                //    tracing::warn!(?tx_data, "Failed to deserialize masp tx
                // data");    None
                //})
            })
            .collect();

        Ok(Transaction {
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
