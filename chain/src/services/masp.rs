use std::collections::BTreeSet;

use namada_core::masp_primitives::ff::PrimeField;
use namada_core::masp_primitives::sapling::Node;
use namada_core::token::Transfer as NamadaMaspTransfer;
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use shared::block_results::Event;
use shared::extracted_masp_tx::ExtractedMaspTx;
use shared::indexed_tx::IndexedTx;
use shared::transaction::{MaspTxType, Transaction};

use crate::entity::commitment_tree::CommitmentTree;
use crate::entity::tx_note_map::TxNoteMap;
use crate::entity::witness_map::WitnessMap;

/// Update the merkle tree of witnesses the first time we
/// scan a new MASP transaction.
pub fn update_witness_map(
    commitment_tree: CommitmentTree,
    tx_note_map: TxNoteMap,
    witness_map: WitnessMap,
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
        witness_map
            .update(node)
            .map_err(|()| "note commitment tree is full".to_string())?;

        commitment_tree
            .append(node)
            .map_err(|()| "note commitment tree is full".to_string())?;

        // Finally, make it easier to construct merkle paths to this new
        // note
        let witness =
            IncrementalWitness::<Node>::from_tree(&commitment_tree.get_tree());
        witness_map.insert(note_pos, witness);
        note_pos += 1;
    }

    Ok(())
}

/// Extract the relevant shield portions of a [`Tx`], if any.
pub fn extract_masp_tx(
    tx: &Transaction,
    tx_event: &Event,
) -> Result<ExtractedMaspTx, String> {
    // We use the changed keys instead of the Transfer object
    // because those are what the masp validity predicate works on
    let (wrapper_changed_keys, changed_keys) =
        match tx_event.attributes.inner_tx.clone() {
            Some(tx_result) => {
                (tx_result.wrapper_changed_keys, tx_result.changed_keys)
            }
            None => (Default::default(), Default::default()),
        };

    let maybe_fee_unshield = tx
        .fee_unshielding_tx
        .as_ref()
        .map(|unshield_fee_tx| (wrapper_changed_keys, unshield_fee_tx.clone()));

    let maybe_masp_tx = match &tx.masp_tx {
        MaspTxType::Normal(tx) => Some((changed_keys, tx.clone())),
        MaspTxType::IBC(tx) => extract_payload_from_shielded_action(
            tx.data().unwrap().as_ref(),
            tx_event,
        )
        .and_then(|(_s, t)| {
            if let Some(hash) = t.shielded {
                let masp_tx = tx.get_section(&hash)?.masp_tx()?;
                Some((changed_keys, masp_tx))
            } else {
                None
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
    let message = namada_ibc::decode_message(tx_data).ok()?;

    let result = match message {
        IbcMessage::Transfer(msg) => {
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::NftTransfer(msg) => {
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::RecvPacket(msg) => {
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::AckPacket(msg) => {
            // Refund tokens by the ack message
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::Timeout(msg) => {
            // Refund tokens by the timeout message
            let tx_result = tx_event.get_tx_result()?;

            let transfer = msg.transfer?;

            (tx_result.changed_keys.clone(), transfer)
        }
        IbcMessage::Envelope(_) => {
            return None;
        }
    };

    Some(result)
}
