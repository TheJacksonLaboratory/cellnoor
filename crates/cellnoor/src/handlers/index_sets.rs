pub use dual::create::create_dual_index_sets;
#[cfg(test)]
pub use dual::create::tests::DUAL_INDEX_SET_NAME;
#[cfg(test)]
pub use dual::create::tests::insert_test_dual_index_set;
pub use single::create::create_single_index_sets;
#[cfg(test)]
pub use single::create::tests::insert_test_single_index_set;

use crate::{
    db::{self, AsFieldValuePairs, insert_into_no_returning},
    error::ErrorInner,
    handlers::index_sets::index_set_name::IndexKitName,
};

mod dual;
mod single;

mod index_set_name {
    use std::sync::LazyLock;

    use postgres_types::ToSql;
    use regex::Regex;

    use crate::error::ErrorInner;

    static INDEX_SET_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^SI-([NA]{2}|[TN]{2}|[GA]{2}|[TS]{2}|[TT]{2})-[A-H]\d{1,2}$").unwrap()
    });

    #[derive(Clone, Copy, Debug, ToSql)]
    #[postgres(transparent)]
    pub struct IndexSetName<'a>(&'a str);

    #[derive(Clone, Copy, Debug, PartialEq, Eq, ToSql)]
    #[postgres(transparent)]
    pub struct IndexKitName<'a>(&'a str);

    #[derive(Clone, Copy, Debug, ToSql)]
    #[postgres(transparent)]
    pub struct IndexSetWellName<'a>(&'a str);

    impl<'a> IndexSetName<'a> {
        pub fn new(index_set_name: &'a str) -> Result<Self, ErrorInner> {
            if !INDEX_SET_NAME_REGEX.is_match(index_set_name) {
                return Err(ErrorInner::DataConstraint {
                    resource: None,
                    message: "malformed index set name".to_owned(),
                    field: None,
                    detail: Some(format!("must match {}", INDEX_SET_NAME_REGEX.to_string())),
                });
            }

            return Ok(Self(index_set_name));
        }

        pub fn kit_name(&self) -> IndexKitName<'a> {
            IndexKitName(&self.0[3..5])
        }

        pub fn well_name(&self) -> IndexSetWellName<'a> {
            IndexSetWellName(&self.0[6..8])
        }
    }
}

mod sequence {
    use std::sync::LazyLock;

    use postgres_types::ToSql;
    use regex::Regex;

    use crate::error::ErrorInner;

    static DNA_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[ACGT]{8}|[ACGT]{10}$").unwrap());

    #[derive(Clone, Copy, Debug, ToSql)]
    #[postgres(transparent)]
    pub struct DnaSequence<'a>(&'a str);

    impl<'a> DnaSequence<'a> {
        pub fn new(sequence: &'a str) -> Result<Self, ErrorInner> {
            if !DNA_REGEX.is_match(sequence) {
                return Err(ErrorInner::DataConstraint {
                    resource: None,
                    message: "malformed DNA sequence".to_owned(),
                    field: None,
                    detail: Some(format!("must match {}", DNA_REGEX.to_string())),
                });
            }

            Ok(Self(sequence))
        }
    }
}

struct NewIndexKit<'a> {
    name: IndexKitName<'a>,
}

impl AsFieldValuePairs<&'static str, 1> for NewIndexKit<'_> {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, &'static str, 1> {
        [("name", &self.name)]
    }
}

async fn insert_index_kit(
    tx: &db::Transaction<'_>,
    index_kit: &NewIndexKit<'_>,
) -> Result<(), ErrorInner> {
    insert_into_no_returning(tx, "index_kit", index_kit).await?;

    Ok(())
}
