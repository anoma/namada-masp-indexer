use namada_sdk::events::extend::{
    IndexedMaspData, MaspDataRefs, ReadFromEventAttributes,
};
use tendermint_rpc::endpoint::block_results;

pub fn locate_masp_txs(
    raw_block_results: &block_results::Response,
) -> Vec<IndexedMaspData> {
    raw_block_results
        .end_block_events
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|event| {
            MaspDataRefs::read_from_event_attributes(&event.attributes).ok()
        })
        .collect()
}
