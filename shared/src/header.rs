use tendermint::block::Header;

use crate::height::BlockHeight;

use super::id::Id;

#[derive(Debug, Clone, Default)]
pub struct BlockHeader {
    pub height: BlockHeight,
    pub proposer_address: Id,
    pub timestamp: String,
    pub app_hash: Id,
}

impl From<Header> for BlockHeader {
    fn from(value: Header) -> Self {
        Self {
            height: BlockHeight::from(value.height),
            proposer_address: Id::Account(value.proposer_address.to_string().to_lowercase()),
            timestamp: value.time.to_rfc3339(),
            app_hash: Id::from(value.app_hash),
        }
    }
}