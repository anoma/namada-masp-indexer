use std::collections::BTreeMap;

use namada_sdk::events::extend::{
    MaspTxBatchRefs, MaspTxBlockIndex, MaspTxRefs, ReadFromEventAttributes,
};
use tendermint_rpc::endpoint::block_results;

pub struct IndexedMaspTxs {
    /// Mapping of block indexes to valid masp tx ids.
    pub locations: BTreeMap<usize, MaspTxRefs>,
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

                // Extract the references to the correct masp sections
                let refs = MaspTxBatchRefs::read_from_event_attributes(
                    &event.attributes,
                )
                .unwrap_or_default();

                Some((index.0 as usize, refs))
            })
            .collect(),
    }
}
