use namada_sdk::state::TxIndex as NamadaTxIndex;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxIndex(pub u32);

impl From<NamadaTxIndex> for TxIndex {
    fn from(value: NamadaTxIndex) -> Self {
        Self(value.0)
    }
}
