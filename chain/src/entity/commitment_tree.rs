use std::sync::{Arc, Mutex};

use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::masp_primitives::merkle_tree::CommitmentTree as MaspCommitmentTree;
use namada_sdk::masp_primitives::sapling::Node;
use orm::tree::TreeInsertDb;
use shared::height::BlockHeight;

#[derive(Clone, Debug)]
pub struct CommitmentTree(Arc<Mutex<MaspCommitmentTree<Node>>>);

impl CommitmentTree {
    pub fn new(tree: MaspCommitmentTree<Node>) -> Self {
        Self(Arc::new(Mutex::new(tree)))
    }

    pub fn size(&self) -> usize {
        self.0.lock().unwrap().size()
    }

    pub fn append(&self, node: Node) -> Result<(), ()> {
        self.0.lock().unwrap().append(node)
    }

    pub fn get_tree(&self) -> MaspCommitmentTree<Node> {
        self.0.lock().unwrap().clone()
    }

    pub fn into_db(&self, block_height: BlockHeight) -> TreeInsertDb {
        TreeInsertDb {
            tree: self.0.lock().unwrap().serialize_to_vec(),
            block_height: block_height.0 as i32,
        }
    }
}

impl Default for CommitmentTree {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(MaspCommitmentTree::empty())))
    }
}
