use std::collections::BTreeMap;

use namada_core::hash::Hash;
use namada_sdk::events::extend::{
    MaspTxBatchRefs, MaspTxBlockIndex, ReadFromEventAttributes,
};
use tendermint_rpc::endpoint::block_results;

pub struct IndexedMaspTxs {
    /// Mapping of block indexes to valid masp tx section hashes.
    pub locations: BTreeMap<usize, Vec<Hash>>,
}

pub fn locate_masp_txs(
    raw_block_results: &block_results::Response,
) -> IndexedMaspTxs {
    IndexedMaspTxs {
        locations: raw_block_results
            .end_block_events
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|event| {
                let index = MaspTxBlockIndex::read_from_event_attributes(
                    &event.attributes,
                )
                .ok()?;
                let masp_tx_sechashes =
                    MaspTxBatchRefs::read_from_event_attributes(
                        &event.attributes,
                    )
                    .ok()?;
                Some((index.0 as usize, masp_tx_sechashes.0))
            })
            .collect(),
    }
}
