use std::collections::BTreeMap;

use namada_sdk::events::extend::{
    IndexedMaspData, MaspDataRefs, MaspTxRefs, ReadFromEventAttributes,
};
use tendermint_rpc::endpoint::block_results;

// FIXME: maybe can avoid this too and use the type from namada
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
                MaspDataRefs::read_from_event_attributes(&event.attributes)
                    .ok()
                    .map(
                        |IndexedMaspData {
                             tx_index,
                             masp_refs,
                         }| {
                            (tx_index.0 as usize, masp_refs)
                        },
                    )
            })
            .collect(),
    }
}
