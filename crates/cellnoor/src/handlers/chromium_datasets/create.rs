use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cellnoor_types::{
    Relation,
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetField, NewChromiumDataset, NewChromiumDatasetRecord,
    },
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert, Sql},
    error::error_response,
    handlers::chromium_datasets::show::select_chromium_dataset_by_id,
    state::AppState,
};

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
pub enum CreateChromiumDatasetError {
    #[error("all libraries in a Chromium dataset must come from the same GEM well")]
    LibrariesFromDifferentGemWells,
    #[error("cannot have multiple instances of the same library type in one Chromium dataset")]
    DuplicateLibraryType,
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl CreateChromiumDatasetError {
    fn status(&self) -> StatusCode {
        match self {
            Self::LibrariesFromDifferentGemWells | Self::DuplicateLibraryType => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::Db(e) => e.status(),
        }
    }
}

impl IntoResponse for CreateChromiumDatasetError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for CreateChromiumDatasetError {
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

pub async fn create_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewChromiumDataset>,
) -> Result<Json<ChromiumDatasetDetailed>, CreateChromiumDatasetError> {
    state
        .in_transaction(user, async |tx| {
            insert_chromium_dataset(tx, &state.public_files_url, record).await
        })
        .await
}

async fn insert_chromium_dataset(
    tx: &db::Transaction<'_>,
    raw_files_url: &str,
    NewChromiumDataset {
        record,
        library_ids,
    }: NewChromiumDataset,
) -> Result<ChromiumDatasetDetailed, CreateChromiumDatasetError> {
    validate_libraries_have_same_gem_well(tx, library_ids.as_ref()).await?;

    let id = tx.insert_returning_id(&record).await?;

    insert_chromium_dataset_libraries(tx, id, library_ids.as_ref()).await?;

    Ok(select_chromium_dataset_by_id(tx, raw_files_url, id).await?)
}

pub async fn validate_libraries_have_same_gem_well(
    tx: &db::Transaction<'_>,
    library_ids: &[Uuid],
) -> Result<(), CreateChromiumDatasetError> {
    static SELECT_N_GEM_WELLS_AND_LIBRARY_TYPES: &str =
        include_str!("create/select_n_gem_wells_and_lib_types.sql");

    let sql = Sql::new(SELECT_N_GEM_WELLS_AND_LIBRARY_TYPES, vec![&library_ids]);

    let (n_gem_wells, n_library_types): (i64, i64) = tx
        .query_one(&sql)
        .await
        .map(|row| (row.get("n_gem_wells"), row.get("n_library_types")))?;

    if n_gem_wells != 1 {
        return Err(CreateChromiumDatasetError::LibrariesFromDifferentGemWells);
    }

    if library_ids.len() as i64 != n_library_types {
        return Err(CreateChromiumDatasetError::DuplicateLibraryType);
    }

    Ok(())
}

async fn insert_chromium_dataset_libraries(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    library_ids: &[Uuid],
) -> Result<(), DbError> {
    let rows: Vec<_> = library_ids
        .iter()
        .map(|&library_id| NewChromiumDatasetLibrary {
            dataset_id,
            library_id,
        })
        .collect();

    tx.insert_many(&rows).await
}

struct NewChromiumDatasetLibrary {
    dataset_id: Uuid,
    library_id: Uuid,
}

impl Relation for NewChromiumDatasetLibrary {
    const NAME: &'static str = "chromium_dataset_library";
}

impl Insert for NewChromiumDatasetLibrary {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            dataset_id,
            library_id,
        } = self;

        vec![("dataset_id", dataset_id), ("library_id", library_id)]
    }
}

impl Insert for NewChromiumDatasetRecord {
    type Field = ChromiumDatasetField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use ChromiumDatasetField::*;

        let Self {
            id: _,
            name,
            delivered_at,
        } = self;

        vec![(Name, name), (DeliveredAt, delivered_at)]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        chromium_dataset::{ChromiumDatasetDetailed, NewChromiumDataset, NewChromiumDatasetRecord},
        id::NoId,
        nonempty::NonemptyVec,
    };
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        handlers::{
            chromium_datasets::create::{CreateChromiumDatasetError, insert_chromium_dataset},
            libraries::create::test::insert_test_library,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_chromium_dataset<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewChromiumDataset, ChromiumDatasetDetailed), CreateChromiumDatasetError>
    where
        F: FnMut(&mut NewChromiumDataset),
    {
        let (_, library) = insert_test_library(tx, |_| ()).await?;

        let mut new = NewChromiumDataset {
            record: NewChromiumDatasetRecord {
                id: NoId,
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                delivered_at: Timestamp::now(),
            },
            library_ids: NonemptyVec::new(vec![*library.record.id]).unwrap(),
        };

        modify(&mut new);

        let inserted = insert_chromium_dataset(tx, "files", new.clone()).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_chromium_dataset(&tx, |_| ()).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn libraries_from_different_gem_wells_fail() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, library1) = insert_test_library(&tx, |_| ()).await.unwrap();
        let (_, library2) = insert_test_library(&tx, |_| ()).await.unwrap();

        let new = NewChromiumDataset {
            record: NewChromiumDatasetRecord {
                id: NoId,
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                delivered_at: Timestamp::now(),
            },
            library_ids: NonemptyVec::new(vec![*library1.record.id, *library2.record.id]).unwrap(),
        };

        let err = insert_chromium_dataset(&tx, "files", new)
            .await
            .unwrap_err();

        assert_eq!(
            err,
            CreateChromiumDatasetError::LibrariesFromDifferentGemWells
        );
    }
}
