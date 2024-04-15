use std::{collections::BTreeSet, str::FromStr};

use namada_tx::data::TxResult as NamadaTxResult;

#[derive(Debug, Clone)]
pub struct TxResult {
    pub wrapper_changed_keys: BTreeSet<String>,
    pub changed_keys: BTreeSet<String>,
}

impl FromStr for TxResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tx_result = NamadaTxResult::from_str(s).map_err(|e| e.to_string())?;
        Ok(Self {
            wrapper_changed_keys: tx_result.wrapper_changed_keys.iter().map(|key| key.to_string()).collect(),
            changed_keys: tx_result.changed_keys.iter().map(|key| key.to_string()).collect(),
        })
    }
}