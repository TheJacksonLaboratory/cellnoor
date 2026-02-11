use jiff::Timestamp;

pub use super::routes::people::validate_email;
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

pub trait AllSame {
    fn all_same(&mut self) -> bool;
}

impl<I, T> AllSame for I
where
    I: Iterator<Item = T>,
    T: PartialEq,
{
    fn all_same(&mut self) -> bool {
        let Some(first) = self.next() else {
            return true;
        };

        self.all(|it| first == it)
    }
}
