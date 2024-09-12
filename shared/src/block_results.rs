use std::collections::BTreeMap;

use namada_core::hash::Hash;
use namada_core::masp::TxId;
use namada_sdk::events::extend::{
    IbcMaspTxBatchRefs, MaspTxBatchRefs, MaspTxBlockIndex,
    ReadFromEventAttributes,
};
use tendermint_rpc::endpoint::block_results;

pub enum IndexedMaspTxRef {
    /// The masp tx is located in section with
    /// the given masp [`TxId`].
    TxId(TxId),
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
                let mut refs = vec![];
                if let Ok(tx_ids) = MaspTxBatchRefs::read_from_event_attributes(
                    &event.attributes,
                ) {
                    refs.extend(
                        tx_ids.0.into_iter().map(IndexedMaspTxRef::TxId),
                    );
                }
                if let Ok(sechashes) =
                    IbcMaspTxBatchRefs::read_from_event_attributes(
                        &event.attributes,
                    )
                {
                    refs.extend(
                        sechashes
                            .0
                            .into_iter()
                            .map(IndexedMaspTxRef::IbcEnvelopeDataSecHash),
                    );
                }
                Some((index.0 as usize, refs))
            })
            .collect(),
    }
}
