use orm::chain_state::ChainStateteInsertDb;
use shared::height::BlockHeight;

pub struct ChainState {
    pub block_height: BlockHeight,
}

impl ChainState {
    pub fn new(block_height: BlockHeight) -> Self {
        Self { block_height }
    }

    pub fn into_db(&self) -> ChainStateteInsertDb {
        ChainStateteInsertDb {
            block_height: self.block_height.0 as i32,
        }
    }
}
