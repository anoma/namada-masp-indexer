use orm::chain_state::ChainStateteInsertDb;
use shared::height::BlockHeight;

#[derive(Clone, Copy, Debug)]
pub struct ChainState {
    pub block_height: BlockHeight,
}

impl ChainState {
    pub fn new(block_height: BlockHeight) -> Self {
        Self { block_height }
    }

    pub fn into_db(&self) -> ChainStateteInsertDb {
        ChainStateteInsertDb {
            id: 0, // NB: overwrite old row
            block_height: self.block_height.0 as i32,
        }
    }
}
