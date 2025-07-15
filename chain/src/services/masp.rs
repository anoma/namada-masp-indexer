use namada_core::masp_primitives::ff::PrimeField;
use namada_core::masp_primitives::sapling::Node;
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use rayon::prelude::*;
use shared::indexed_tx::MaspIndexedTx;

use crate::entity::commitment_tree::CommitmentTree;
use crate::entity::tx_notes_index::TxNoteMap;
use crate::entity::witness_map::WitnessMap;

pub fn update_witness_map(
    commitment_tree: &mut CommitmentTree,
    tx_notes_index: &mut TxNoteMap,
    witness_map: &mut WitnessMap,
    indexed_tx: MaspIndexedTx,
    shielded: &namada_core::masp_primitives::transaction::Transaction,
) -> anyhow::Result<()> {
    tracing::info!(?indexed_tx, "Updating witness map");

    let mut note_pos = commitment_tree.size();
    tx_notes_index.insert(indexed_tx, note_pos);

    for so in shielded
        .sapling_bundle()
        .map_or(&vec![], |x| &x.shielded_outputs)
    {
        // Create merkle tree leaf node from note commitment
        let node = Node::new(so.cmu.to_repr());

        // Update each merkle tree in the witness map with the latest
        // addition
        witness_map.update(node).map_err(|note_pos| {
            anyhow::anyhow!("Witness map is full at note position {note_pos}")
        })?;

        if !commitment_tree.append(node) {
            anyhow::bail!("Note commitment tree is full");
        }

        // Finally, make it easier to construct merkle paths to this new
        // note
        let witness =
            IncrementalWitness::<Node>::from_tree(&commitment_tree.get_tree());
        witness_map.insert(note_pos, witness);
        note_pos += 1;
    }

    Ok(())
}

pub fn query_witness_map_anchor_existence(
    witness_map: &WitnessMap,
    cmt_tree_root: Node,
    roots_to_check: usize,
) -> anyhow::Result<()> {
    let witness_map_roots = witness_map.roots(roots_to_check);

    if !witness_map_roots.into_par_iter().all(
        |(note_index, witness_map_root)| {
            if witness_map_root == cmt_tree_root {
                true
            } else {
                tracing::error!(
                    ?cmt_tree_root,
                    ?witness_map_root,
                    %note_index,
                    "Anchor mismatch"
                );
                false
            }
        },
    ) {
        anyhow::bail!("There is an invalid anchor in the witness map");
    }

    Ok(())
}
