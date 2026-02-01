use jiff::Timestamp;

use crate::db::DataError;

pub fn validate_timestamps(
    (t1, field_name1): (Timestamp, &'static str),
    (t2, field_name2): (Timestamp, &'static str),
) -> Result<(), DataError> {
    if t1 > t2 {
        return Err(DataError::new_timestamp_error(
            (t1, field_name1),
            (t2, field_name2),
        ));
    }

    Ok(())
}
