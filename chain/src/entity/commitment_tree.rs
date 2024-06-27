use std::sync::{Arc, Mutex};

use namada_sdk::borsh::{BorshDeserialize, BorshSerializeExt};
use namada_sdk::masp_primitives::merkle_tree::CommitmentTree as MaspCommitmentTree;
use namada_sdk::masp_primitives::sapling::Node;
use orm::tree::{TreeDb, TreeInsertDb};
use shared::height::BlockHeight;
use shared::transactional::Transactional;

#[derive(Debug)]
struct InnerCommitmentTree {
    transactional: Transactional<MaspCommitmentTree<Node>>,
}

impl Default for InnerCommitmentTree {
    fn default() -> Self {
        Self {
            transactional: Transactional::new(MaspCommitmentTree::empty()),
        }
    }
}

impl InnerCommitmentTree {
    const fn new(tree: MaspCommitmentTree<Node>) -> Self {
        Self {
            transactional: Transactional::new(tree),
        }
    }

    fn append(&mut self, node: Node) -> bool {
        self.transactional.as_mut().append(node).is_ok()
    }

    fn size(&self) -> usize {
        self.transactional.as_ref().size()
    }

    fn get_tree(&self) -> MaspCommitmentTree<Node> {
        self.transactional.as_ref().clone()
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_db(&mut self, block_height: BlockHeight) -> Option<TreeInsertDb> {
        if !self.transactional.commit() {
            return None;
        }
        Some(TreeInsertDb {
            tree: self.transactional.as_ref().serialize_to_vec(),
            block_height: block_height.0 as i32,
        })
    }
}

#[derive(Default, Clone, Debug)]
pub struct CommitmentTree(Arc<Mutex<InnerCommitmentTree>>);

impl CommitmentTree {
    pub fn new(tree: MaspCommitmentTree<Node>) -> Self {
        Self(Arc::new(Mutex::new(InnerCommitmentTree::new(tree))))
    }

    pub fn size(&self) -> usize {
        self.0.lock().unwrap().size()
    }

    pub fn append(&self, node: Node) -> bool {
        self.0.lock().unwrap().append(node)
    }

    pub fn get_tree(&self) -> MaspCommitmentTree<Node> {
        self.0.lock().unwrap().get_tree()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_db(&self, block_height: BlockHeight) -> Option<TreeInsertDb> {
        self.0.lock().unwrap().into_db(block_height)
    }
}

impl TryFrom<TreeDb> for CommitmentTree {
    type Error = String;

    fn try_from(value: TreeDb) -> Result<Self, Self::Error> {
        let commitment_tree =
            MaspCommitmentTree::<Node>::try_from_slice(&value.tree)
                .map_err(|e| e.to_string())?;
        Ok(Self::new(commitment_tree))
    }
}
