use std::fmt::Display;

use namada_core::borsh::BorshDeserialize;
use namada_core::masp_primitives::transaction::Transaction as NamadaMaspTransaction;
use namada_core::token::Transfer as NamadaMaspTransfer;
use namada_tx::data::TxType;
use namada_tx::Tx as NamadaTx;

use crate::block_results::{TxAttributes, TxEventStatusCode};
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

impl TransactionExitStatus {
    pub fn from(tx_attributes: &TxAttributes) -> Self {
        match tx_attributes.code {
            TxEventStatusCode::Ok => TransactionExitStatus::Applied,
            TxEventStatusCode::Fail => TransactionExitStatus::Rejected,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MaspTxType {
    Normal(NamadaMaspTransaction),
    IBC(NamadaTx),
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub masp_tx: MaspTxType,
    pub fee_unshielding_tx: Option<NamadaMaspTransaction>,
    pub memo: Option<Vec<u8>>,
}

impl TryFrom<&[u8]> for Transaction {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let tx = NamadaTx::try_from(value).map_err(|e| e.to_string())?;

        let fee_unshielding_tx =
            if let TxType::Wrapper(wrapper_tx) = tx.header().tx_type {
                wrapper_tx.unshield_section_hash.and_then(|hash| {
                    tx.get_section(&hash).and_then(|section| section.masp_tx())
                })
            } else {
                None
            };

        let tx_data = tx
            .data()
            .ok_or_else(|| "MASP tx requires data.".to_string())?;

        let masp_tx = match NamadaMaspTransfer::try_from_slice(&tx_data) {
            Ok(transfer) => transfer
                .shielded
                .and_then(|hash| {
                    tx.get_section(&hash).and_then(|s| s.masp_tx())
                })
                .map(MaspTxType::Normal)
                .ok_or_else(|| "Not a MASP tx".to_string())?,
            Err(_) => MaspTxType::IBC(tx.clone()),
        };

        Ok(Self {
            hash: Id::from(tx.header_hash()),
            memo: tx.memo(),
            fee_unshielding_tx,
            masp_tx,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash)
    }
}
