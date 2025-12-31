#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct DefaultVec<T>(Vec<T>);

impl<T> DefaultVec<T> {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }
}

impl<T> Default for DefaultVec<T>
where
    T: Default,
{
    fn default() -> Self {
        Self(vec![T::default()])
    }
}

impl<T> AsMut<[T]> for DefaultVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T> AsRef<[T]> for DefaultVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> From<T> for DefaultVec<T> {
    fn from(value: T) -> Self {
        Self(vec![value])
    }
}

impl<T, const N: usize> From<[T; N]> for DefaultVec<T> {
    fn from(value: [T; N]) -> Self {
        Self(value.into())
    }
}
