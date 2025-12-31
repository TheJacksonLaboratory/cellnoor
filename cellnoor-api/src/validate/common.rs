use jiff::Timestamp;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "TimestampError"))]
#[error("t1 must be before t2")]
pub struct TimestampError {
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    t1: Timestamp,
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    t2: Timestamp,
    field: &'static str,
}

pub(super) fn validate_timestamps(
    first: Timestamp,
    second: Timestamp,
    field: &'static str,
) -> Result<(), TimestampError> {
    if first > second {
        return Err(TimestampError {
            t1: first,
            t2: second,
            field,
        });
    }

    Ok(())
}
