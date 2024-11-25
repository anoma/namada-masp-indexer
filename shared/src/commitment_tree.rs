use std::sync::LazyLock;

use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::masp_primitives::merkle_tree::CommitmentTree;
use namada_sdk::masp_primitives::sapling::Node;

/// Return an empty serialized [`CommitmentTree`].
#[inline]
pub fn empty() -> Vec<u8> {
    static EMPTY_TREE: LazyLock<Vec<u8>> =
        LazyLock::new(|| CommitmentTree::<Node>::empty().serialize_to_vec());
    EMPTY_TREE.clone()
}
