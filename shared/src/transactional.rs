#[derive(Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Transactional<T> {
    committed: T,
    working_copy: Option<T>,
}

impl<T> Transactional<T> {
    pub const fn new(initial_value: T) -> Self {
        Self {
            committed: initial_value,
            working_copy: None,
        }
    }
}

impl<T> AsRef<T> for Transactional<T> {
    fn as_ref(&self) -> &T {
        self.working_copy.as_ref().unwrap_or(&self.committed)
    }
}

impl<T> AsMut<T> for Transactional<T>
where
    T: Clone,
{
    fn as_mut(&mut self) -> &mut T {
        self.working_copy
            .get_or_insert_with(|| self.committed.clone())
    }
}

impl<T> Transactional<T> {
    pub fn commit(&mut self) -> bool {
        let Some(new_data) = self.working_copy.take() else {
            return false;
        };
        self.committed = new_data;
        true
    }
}
