#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[error("value {found} must be  between {minimum} and {maximum}")]
pub(super) struct InvalidMeasurement<const MIN: i32, const MAX: i32> {
    minimum: i32,
    maximum: i32,
    found: f32,
}

impl<const MIN: i32, const MAX: i32> InvalidMeasurement<MIN, MAX> {
    pub fn new(found: f32) -> Self {
        Self {
            minimum: MIN,
            maximum: MAX,
            found,
        }
    }
}
