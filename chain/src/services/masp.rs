use namada_core::masp_primitives::ff::PrimeField;
use namada_core::masp_primitives::sapling::Node;
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use shared::indexed_tx::IndexedTx;

use crate::entity::commitment_tree::CommitmentTree;
use crate::entity::tx_notes_index::TxNoteMap;
use crate::entity::witness_map::WitnessMap;

/// Update the merkle tree of witnesses the first time we
/// scan a new MASP transaction.
pub fn update_witness_map(
    commitment_tree: &CommitmentTree,
    tx_notes_index: &mut TxNoteMap,
    witness_map: &WitnessMap,
    indexed_tx: IndexedTx,
    shielded: &namada_core::masp_primitives::transaction::Transaction,
) -> anyhow::Result<()> {
    let mut note_pos = commitment_tree.size();
    tx_notes_index
        .insert(indexed_tx, false /* is_fee_unshielding */, note_pos);

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
