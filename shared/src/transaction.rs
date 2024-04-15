use std::fmt::Display;

use crate::{
    block_results::{TxAttributes, TxEventStatusCode},
    id::Id,
};
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_tx::{data::TxType, Section, Tx as NamadaTx};

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

impl TransactionExitStatus {
    pub fn from(tx_attributes: &TxAttributes) -> Self {
        match tx_attributes.code {
            TxEventStatusCode::Ok => TransactionExitStatus::Applied,
            TxEventStatusCode::Fail => TransactionExitStatus::Rejected,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_tx: NamadaMaspTransaction,
    pub fee_unshielding_tx: Option<NamadaMaspTransaction>,
    pub memo: Option<Vec<u8>>,
}

impl TryFrom<&[u8]> for Transaction {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let transaction =
            NamadaTx::try_from(value).map_err(|e| e.to_string())?;

        let fee_unshielding_tx =
            if let TxType::Wrapper(wrapper_tx) = transaction.header().tx_type {
                wrapper_tx.unshield_section_hash.and_then(|hash| {
                    transaction
                        .get_section(&hash)
                        .and_then(|section| section.masp_tx())
                })
            } else {
                None
            };

        Ok(Self {
            hash: Id::from(transaction.header_hash()),
            memo: transaction.memo(),
            fee_unshielding_tx: fee_unshielding_tx,
            masp_tx: transaction
                .sections
                .into_iter()
                .find_map(|section| match section {
                    Section::MaspTx(masp) => Some(masp),
                    _ => None,
                })
                .ok_or_else(|| "Not a masp transaction")?,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash)
    }
}
