// This is necessary to prevent stupid warnings on the test binary
#![cfg_attr(test, allow(dead_code_pub_in_binary))]
mod string;
mod vec;

pub use string::NonemptyString;
pub use vec::{NonemptyBoundedVec, NonemptyVec};

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("empty collection not allowed")]
pub struct Error<T>(pub T);
