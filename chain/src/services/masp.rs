use namada_core::masp_primitives::ff::PrimeField;
use namada_core::masp_primitives::sapling::Node;
use namada_core::masp_primitives::transaction::Transaction;
use namada_sdk::masp_primitives::merkle_tree::IncrementalWitness;
use shared::indexed_tx::IndexedTx;

use crate::entity::commitment_tree::CommitmentTree;
use crate::entity::tx_notes_index::TxNoteMap;
use crate::entity::witness_map::WitnessMap;

pub fn update_commitment_tree(
    commitment_tree: &CommitmentTree,
    stx_batch: &Transaction,
) -> anyhow::Result<()> {
    for so in stx_batch
        .sapling_bundle()
        .map_or(&vec![], |x| &x.shielded_outputs)
    {
        let node = Node::new(so.cmu.to_repr());
        if !commitment_tree.append(node) {
            anyhow::bail!("Note commitment tree is full");
        }
    }

    Ok(())
}

pub fn update_witness_map_and_note_index(
    note_pos: &mut usize,
    commitment_tree: &CommitmentTree,
    tx_notes_index: &mut TxNoteMap,
    witness_map: &WitnessMap,
    indexed_tx: IndexedTx,
    shielded: &Transaction,
) -> anyhow::Result<()> {
    tx_notes_index.insert(indexed_tx, *note_pos);

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

        // Finally, make it easier to construct merkle paths to this new
        // note
        let witness =
            IncrementalWitness::<Node>::from_tree(&commitment_tree.get_tree());
        witness_map.insert(*note_pos, witness);
        *note_pos += 1;
    }

    Ok(())
}
