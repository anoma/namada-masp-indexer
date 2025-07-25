use anyhow::Context;
use namada_sdk::borsh::{BorshDeserialize, BorshSerializeExt};
use namada_sdk::masp_primitives::merkle_tree::CommitmentTree as MaspCommitmentTree;
use namada_sdk::masp_primitives::sapling::Node;
use orm::tree::{TreeDb, TreeInsertDb};
use shared::height::BlockHeight;
use shared::transactional::Transactional;

#[derive(Debug)]
pub struct CommitmentTree {
    transactional: Transactional<MaspCommitmentTree<Node>>,
}

impl Default for CommitmentTree {
    fn default() -> Self {
        Self {
            transactional: Transactional::new(MaspCommitmentTree::empty()),
        }
    }
}

impl CommitmentTree {
    pub const fn new(tree: MaspCommitmentTree<Node>) -> Self {
        Self {
            transactional: Transactional::new(tree),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.transactional.is_dirty()
    }

    pub fn rollback(&mut self) {
        self.transactional.rollback();
    }

    pub fn append(&mut self, node: Node) -> bool {
        self.transactional.as_mut().append(node).is_ok()
    }

    pub fn size(&self) -> usize {
        self.transactional.as_ref().size()
    }

    pub fn root(&self) -> Node {
        self.transactional.as_ref().root()
    }

    pub fn get_tree(&self) -> MaspCommitmentTree<Node> {
        self.transactional.as_ref().clone()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_db(
        &mut self,
        block_height: BlockHeight,
    ) -> Option<TreeInsertDb> {
        if !self.transactional.commit() {
            return None;
        }
        Some(TreeInsertDb {
            tree: self.transactional.as_ref().serialize_to_vec(),
            block_height: block_height.0 as i32,
        })
    }
}

impl TryFrom<TreeDb> for CommitmentTree {
    type Error = anyhow::Error;

    fn try_from(value: TreeDb) -> Result<Self, Self::Error> {
        let commitment_tree =
            MaspCommitmentTree::<Node>::try_from_slice(&value.tree).context(
                "Failed to deserialize commitment tree from db borsh encoded \
                 bytes",
            )?;
        Ok(Self::new(commitment_tree))
    }
}
