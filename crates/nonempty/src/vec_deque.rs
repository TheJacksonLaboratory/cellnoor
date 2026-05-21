use std::{collections::VecDeque, fmt::Debug};

use crate::vec::Error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(try_from = "VecDeque<T>"))]
#[cfg_attr(feature = "schemars", schemars(with = "VecDeque<T>"))]
pub struct NonemptyBoundedVecDeque<T, const N: usize>(VecDeque<T>);

impl<T, const N: usize> From<T> for NonemptyBoundedVecDeque<T, N> {
    fn from(value: T) -> Self {
        Self(VecDeque::from_iter([value]))
    }
}

impl<T, const N: usize> NonemptyBoundedVecDeque<T, N> {
    pub fn new(v: VecDeque<T>) -> Result<Self, Error<VecDeque<T>, N>> {
        if v.is_empty() {
            return Err(Error(v));
        }

        if v.len() > N {
            return Err(Error(v));
        }

        Ok(Self(v))
    }

    pub fn push_front(&mut self, value: T) {
        self.0.push_front(value);
    }
}

impl<T, const N: usize> NonemptyBoundedVecDeque<T, N> {
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.0.iter()
    }
}

impl<T, const N: usize> IntoIterator for NonemptyBoundedVecDeque<T, N> {
    type IntoIter = <VecDeque<T> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a NonemptyBoundedVecDeque<T, N> {
    type IntoIter = std::collections::vec_deque::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<T, const N: usize> From<NonemptyBoundedVecDeque<T, N>> for VecDeque<T> {
    fn from(value: NonemptyBoundedVecDeque<T, N>) -> Self {
        value.0
    }
}

impl<T, const N: usize> TryFrom<VecDeque<T>> for NonemptyBoundedVecDeque<T, N> {
    type Error = Error<VecDeque<T>, N>;

    fn try_from(value: VecDeque<T>) -> Result<Self, Error<VecDeque<T>, N>> {
        Self::new(value)
    }
}

pub type NonemptyVecDeque<T> = NonemptyBoundedVecDeque<T, { usize::MAX }>;
