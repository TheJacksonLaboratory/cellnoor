//! Collections that cannot be empty.

mod string;
mod vec;

pub use string::NonemptyString;
pub use vec::{NonemptyBoundedVec, NonemptyVec};

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("empty collection not allowed")]
pub struct Error;
