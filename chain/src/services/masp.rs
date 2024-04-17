use std::collections::{BTreeMap, BTreeSet, HashMap};

use namada_core::masp_primitives::ff::PrimeField;
use namada_core::masp_primitives::merkle_tree::{
    CommitmentTree, IncrementalWitness,
};
use namada_core::masp_primitives::sapling::Node;
use namada_core::token::Transfer as NamadaMaspTransfer;
use shared::block_results::{BlockResult, Event};
use shared::extracted_masp_tx::ExtractedMaspTx;
use shared::indexed_tx::IndexedTx;
use shared::transaction::{MaspTxType, Transaction};
use shared::tx_index::TxIndex;

/// Update the merkle tree of witnesses the first time we
/// scan a new MASP transaction.
pub fn update_witness_map(
    commitment_tree: &mut CommitmentTree<Node>,
    tx_note_map: &mut BTreeMap<IndexedTx, usize>,
    witness_map: &mut HashMap<usize, IncrementalWitness<Node>>,
    indexed_tx: IndexedTx,
    shielded: &namada_core::masp_primitives::transaction::Transaction,
) -> Result<(), String> {
    let mut note_pos = commitment_tree.size();
    tx_note_map.insert(indexed_tx, note_pos);
    for so in shielded
        .sapling_bundle()
        .map_or(&vec![], |x| &x.shielded_outputs)
    {
        // Create merkle tree leaf node from note commitment
        let node = Node::new(so.cmu.to_repr());
        // Update each merkle tree in the witness map with the latest
        // addition
        for (_, witness) in witness_map.iter_mut() {
            witness
                .append(node)
                .map_err(|()| "note commitment tree is full".to_string())?;
        }
        commitment_tree
            .append(node)
            .map_err(|()| "note commitment tree is full".to_string())?;
        // Finally, make it easier to construct merkle paths to this new
        // note
        let witness = IncrementalWitness::<Node>::from_tree(&commitment_tree);
        witness_map.insert(note_pos, witness);
        note_pos += 1;
    }
    Ok(())
}

/// Retrieves all the indexes and tx events at the specified height which refer
/// to a valid masp transaction. If an index is given, it filters only the
/// transactions with an index equal or greater to the provided one.
async fn get_indexed_masp_events_at_height(
    block_results: BlockResult,
    first_idx_to_query: TxIndex,
) -> Vec<(TxIndex, Event)> {
    block_results
        .end_events
        .into_iter()
        .filter_map(|event| {
            let tx_index = event
                .attributes
                .is_valid_masp_tx
                .map(|index| TxIndex(index as u32));
            match tx_index {
                Some(idx) => {
                    if idx.0 >= first_idx_to_query.0 {
                        Some((idx, event))
                    } else {
                        None
                    }
                }
                None => None,
            }
        })
        .collect::<Vec<_>>()
}

/// Extract the relevant shield portions of a [`Tx`], if any.
pub async fn extract_masp_tx(
    tx: &Transaction,
    tx_event: &Event,
) -> Option<(ExtractedMaspTx, String)> {
    // We use the changed keys instead of the Transfer object
    // because those are what the masp validity predicate works on
    let (wrapper_changed_keys, changed_keys) =
        match tx_event.attributes.inner_tx.clone() {
            Some(tx_result) => {
                (tx_result.wrapper_changed_keys, tx_result.changed_keys)
            }
            None => (Default::default(), Default::default()),
        };

    let maybe_fee_unshield =
        if let Some(unshield_fee_tx) = &tx.fee_unshielding_tx {
            Some((wrapper_changed_keys, unshield_fee_tx))
        } else {
            None
        };

    let maybe_masp_tx = match &tx.masp_tx {
        MaspTxType::Normal(tx) => Some((changed_keys, tx.clone())),
        MaspTxType::IBC(tx_data) => extract_payload_from_shielded_action(
            &tx_data, tx_event,
        )
        .and_then(|(s, t)| {
            if let Some(hash) = t.shielded {
                let masp_tx = tx
                    .get_section(&hash)
                    .ok_or_else(|| {
                        "Missing masp section in transaction".to_string()
                    })?
                    .masp_tx()
                    .ok_or_else(|| "Missing masp transaction".to_string())?;
                Ok::<_, String>((changed_keys, masp_tx))
            } else {
                Ok(None)
            }
        }),
    };

    Ok(ExtractedMaspTx {
        fee_unshielding: maybe_fee_unshield,
        inner_tx: maybe_masp_tx,
    })
}

// Extract the changed keys and Transaction hash from a masp over ibc message
fn extract_payload_from_shielded_action(
    tx_data: &[u8],
    tx_event: &Event,
) -> Option<(BTreeSet<String>, NamadaMaspTransfer)> {
    use namada_core::ibc::IbcMessage;
    let message =
        namada_ibc::decode_message(tx_data).map_err(|e| e.to_string())?;

    let result = match message {
        IbcMessage::Transfer(msg) => {
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer.ok_or_else(|| {
                "Missing masp tx in the ibc message".to_string()
            })?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::NftTransfer(msg) => {
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer.ok_or_else(|| {
                "Missing masp tx in the ibc message".to_string()
            })?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::RecvPacket(msg) => {
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer.ok_or_else(|| {
                "Missing masp tx in the ibc message".to_string()
            })?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::AckPacket(msg) => {
            // Refund tokens by the ack message
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer.ok_or_else(|| {
                "Missing masp tx in the ibc message".to_string()
            })?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::Timeout(msg) => {
            // Refund tokens by the timeout message
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer.ok_or_else(|| {
                "Missing masp tx in the ibc message".to_string()
            })?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::Envelope(_) => {
            return None;
        }
    };

    Some(result)
}
