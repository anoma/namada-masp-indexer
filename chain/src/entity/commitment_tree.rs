use std::sync::{Arc, Mutex};

use namada_sdk::masp_primitives::merkle_tree::CommitmentTree as MaspCommitmentTree;
use namada_sdk::masp_primitives::sapling::Node;

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
}

impl Default for CommitmentTree {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(MaspCommitmentTree::empty())))
    }
}
