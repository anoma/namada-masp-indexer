use std::collections::BTreeMap;

use namada_core::hash::Hash;
use namada_core::masp::MaspTxId;
use namada_sdk::events::extend::{
    MaspTxBatchRefs, MaspTxBlockIndex, MaspTxRef, ReadFromEventAttributes,
};
use tendermint_rpc::endpoint::block_results;

// FIXME: need this? Maybe I can directly use the MaspTxRef type of namada if it
// is public. It is, use that one, but in a second commit
pub enum IndexedMaspTxRef {
    /// The masp tx is located in section with
    /// the given masp [`MaspTxId`].
    TxId(MaspTxId),
    /// The masp tx pertains to an ibc shielding,
    /// and is located in an ibc envelope message
    /// inside the data section with the given [`Hash`].
    IbcEnvelopeDataSecHash(Hash),
}

pub struct IndexedMaspTxs {
    /// Mapping of block indexes to valid masp tx ids.
    pub locations: BTreeMap<usize, Vec<IndexedMaspTxRef>>,
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
                .unwrap_or_default()
                .0
                .into_iter()
                .map(|masp_ref| match masp_ref {
                    MaspTxRef::MaspSection(tx_id) => {
                        IndexedMaspTxRef::TxId(tx_id)
                    }
                    MaspTxRef::IbcData(data_hash) => {
                        IndexedMaspTxRef::IbcEnvelopeDataSecHash(data_hash)
                    }
                })
                .collect();

                Some((index.0 as usize, refs))
            })
            .collect(),
    }
}
