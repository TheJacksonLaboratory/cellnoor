use std::sync::LazyLock;

use regex::Regex;

use crate::{
    db::{self, AsFieldValuePairs, insert_into, insert_into_no_returning},
    error::ErrorInner,
};

static INDEX_SET_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^SI-([NA]{2}|[TN]{2}|[GA]{2}|[TS]{2}|[TT]{2})-[A-H]\d{1,2}$").unwrap()
});

static DNA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ACGT]{8}|[ACGT]{10}$").unwrap());

const INDEX_SET_NAME_ERROR_MESSAGE: &str = "malformed index set name";

fn extract_kit_name(s: &str) -> Result<&str, ErrorInner> {
    s.get(3..5).ok_or(ErrorInner::DataConstraint {
        resource: None,
        message: INDEX_SET_NAME_ERROR_MESSAGE.to_owned(),
        field: None,
        detail: Some(format!("must match {}", INDEX_SET_NAME_REGEX.to_string())),
    })
}

fn extract_well_name(s: &str) -> Result<&str, ErrorInner> {
    s.get(6..8).ok_or(ErrorInner::DataConstraint {
        resource: None,
        message: INDEX_SET_NAME_ERROR_MESSAGE.to_owned(),
        field: None,
        detail: Some(format!("must match {}", INDEX_SET_NAME_REGEX.to_string())),
    })
}

async fn insert_index_kit(tx: &db::Transaction<'_>, name: &str) -> anyhow::Result<()> {
    struct NewIndexKit<'a> {
        name: &'a str,
    }

    impl AsFieldValuePairs<&'static str, 1> for NewIndexKit<'_> {
        fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, &'static str, 1> {
            [("name", &self.name)]
        }
    }

    insert_into_no_returning(tx, "index_kit", &NewIndexKit { name }).await?;

    Ok(())
}
