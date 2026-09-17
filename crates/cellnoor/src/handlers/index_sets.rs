use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cellnoor_types::Relation;
pub use dual::create::create_dual_index_sets;
#[cfg(test)]
pub use dual::create::tests::DUAL_INDEX_SET_NAME;
#[cfg(test)]
pub use dual::create::tests::insert_test_dual_index_set;
pub use single::create::create_single_index_sets;
#[cfg(test)]
pub use single::create::tests::insert_test_single_index_set;

use crate::{
    db::{self, DbError, FieldValues, Insert},
    error::error_response,
    handlers::index_sets::index_set_name::IndexKitName,
};

mod dual;
mod single;

#[derive(
    Debug,
    Clone,
    thiserror::Error,
    serde::Serialize,
    schemars::JsonSchema,
    PartialEq,
    Eq,
)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IndexSetError {
    #[error("malformed index set name '{name}' (must match {must_match})")]
    MalformedName {
        name: String,
        must_match: &'static str,
    },
    #[error("malformed DNA sequence '{sequence}' (must match {must_match})")]
    MalformedSequence {
        sequence: String,
        must_match: &'static str,
    },
    #[error("all index sets must share the same kit name")]
    MixedKitNames,
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl IndexSetError {
    fn status(&self) -> StatusCode {
        match self {
            Self::MalformedName { .. } | Self::MalformedSequence { .. } | Self::MixedKitNames => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::Db(e) => e.status(),
        }
    }
}

impl IntoResponse for IndexSetError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for IndexSetError {
    type Inner = Self;

    fn operation_response(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Option<OpenApiResponse> {
        Json::<Self>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        let Some(response) = Self::operation_response(ctx, operation) else {
            return Vec::new();
        };

        vec![(None, response)]
    }
}

mod index_set_name {
    use std::sync::LazyLock;

    use postgres_types::ToSql;
    use regex::Regex;

    use crate::handlers::index_sets::IndexSetError;

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
        pub fn new(index_set_name: &'a str) -> Result<Self, IndexSetError> {
            if !INDEX_SET_NAME_REGEX.is_match(index_set_name) {
                return Err(IndexSetError::MalformedName {
                    name: index_set_name.to_owned(),
                    must_match: INDEX_SET_NAME_REGEX.as_str(),
                });
            }

            Ok(Self(index_set_name))
        }

        pub fn kit_name(&self) -> IndexKitName<'a> {
            IndexKitName(&self.0[3..5])
        }

        pub fn well_name(&self) -> IndexSetWellName<'a> {
            IndexSetWellName(&self.0[6..])
        }
    }
}

mod sequence {
    use std::sync::LazyLock;

    use postgres_types::ToSql;
    use regex::Regex;

    use crate::handlers::index_sets::IndexSetError;

    static DNA_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([ACGT]{8}|[ACGT]{10})$").unwrap());

    #[derive(Clone, Copy, Debug, ToSql)]
    #[postgres(transparent)]
    pub struct DnaSequence<'a>(&'a str);

    impl<'a> DnaSequence<'a> {
        pub fn new(sequence: &'a str) -> Result<Self, IndexSetError> {
            if !DNA_REGEX.is_match(sequence) {
                return Err(IndexSetError::MalformedSequence {
                    sequence: sequence.to_owned(),
                    must_match: DNA_REGEX.as_str(),
                });
            }

            Ok(Self(sequence))
        }
    }
}

struct NewIndexKit<'a> {
    name: IndexKitName<'a>,
}

impl Relation for NewIndexKit<'_> {
    const NAME: &'static str = "index_kit";
}

impl Insert for NewIndexKit<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        vec![("name", &self.name)]
    }
}

async fn insert_index_kit(
    tx: &db::Transaction<'_>,
    index_kit: &NewIndexKit<'_>,
) -> Result<(), DbError> {
    tx.insert(index_kit).await?;

    Ok(())
}
