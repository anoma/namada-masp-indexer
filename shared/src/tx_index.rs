use namada_sdk::state::TxIndex as NamadaTxIndex;

pub struct TxIndex(pub u32);

impl From<NamadaTxIndex> for TxIndex {
    fn from(value: NamadaTxIndex) -> Self {
        Self(value.0)
    }
}
