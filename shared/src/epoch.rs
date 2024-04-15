#[derive(Debug, Clone, Default)]
pub struct Epoch(u64);

impl From<u64> for Epoch {
    fn from(value: u64) -> Self {
        Self(value)
    }
}