use crate::positive::{Positive, PositiveBounded};

mod positive;

pub type PositiveF32 = Positive<f32>;

pub type PositiveI32 = Positive<i32>;

pub type PositiveBoundedF32<const N: u32> = PositiveBounded<f32, N>;

pub type PositiveBoundedI32<const N: u32> = PositiveBounded<i32, N>;
