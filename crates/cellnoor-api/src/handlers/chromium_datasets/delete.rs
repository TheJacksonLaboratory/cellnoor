use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use camino::Utf8Path;
use cellnoor_types::chromium_dataset::SavedChromiumDatasetRecord;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    error::error_response,
    handlers::{
        IdParam, chromium_datasets::dataset_directory::FinalDirectory,
        specific_error_inferred_early_responses,
    },
    state::AppState,
};

pub async fn delete_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, DeleteChromiumDatasetError> {
    state
        .in_transaction(user, async |tx| {
            delete_dataset_from_db_and_disk(tx, &state.static_files_dir, id).await
        })
        .await
}

pub async fn delete_files(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, DeleteChromiumDatasetError> {
    state
        .in_transaction(user, async |tx| {
            delete_files_from_db_and_disk(tx, &state.static_files_dir, id).await
        })
        .await
}

async fn delete_dataset_from_db_and_disk(
    tx: &db::Transaction<'_>,
    static_files_dir: &Utf8Path,
    dataset_id: Uuid,
) -> Result<(), DeleteChromiumDatasetError> {
    tx.delete::<SavedChromiumDatasetRecord>(dataset_id).await?;

    remove_dataset_directory(static_files_dir, dataset_id)
}

async fn delete_files_from_db_and_disk(
    tx: &db::Transaction<'_>,
    static_files_dir: &Utf8Path,
    dataset_id: Uuid,
) -> Result<(), DeleteChromiumDatasetError> {
    delete_file_records(tx, dataset_id).await?;

    remove_dataset_directory(static_files_dir, dataset_id)
}

// Row-level security hides the records from a user who isn't allowed to delete
// them, rather than raising an error. Returning an error when nothing was
// deleted is what stops that user from removing the dataset directory
async fn delete_file_records(tx: &db::Transaction<'_>, dataset_id: Uuid) -> Result<(), DbError> {
    static DELETE_RAW_FILES: &str = "delete from chromium_dataset_raw_file where dataset_id = $1";

    let n_deleted = tx
        .execute(&Sql::new(DELETE_RAW_FILES, vec![&dataset_id]))
        .await?;

    if n_deleted == 0 {
        return Err(DbError::ResourceNotFound);
    }

    Ok(())
}

fn remove_dataset_directory(
    static_files_dir: &Utf8Path,
    dataset_id: Uuid,
) -> Result<(), DeleteChromiumDatasetError> {
    FinalDirectory::new(static_files_dir, dataset_id)
        .remove()
        .map_err(|e| DeleteChromiumDatasetError::RemoveFiles {
            message: e.to_string(),
        })
}

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeleteChromiumDatasetError {
    #[error("failed to remove files from disk: {message}")]
    RemoveFiles { message: String },
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl DeleteChromiumDatasetError {
    fn status(&self) -> StatusCode {
        match self {
            Self::RemoveFiles { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Db(e) => e.status(),
        }
    }
}

impl IntoResponse for DeleteChromiumDatasetError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for DeleteChromiumDatasetError {
    type Inner = Self;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        specific_error_inferred_early_responses::<Self>(ctx, operation)
    }
}

#[cfg(test)]
mod tests {
    use camino::Utf8Path;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::{delete_dataset_from_db_and_disk, delete_files_from_db_and_disk};
    use crate::{
        db::{DbError, Sql},
        handlers::{
            chromium_datasets::{
                create::insert_test_chromium_dataset, upload_files::write_test_dataset_files,
            },
            chromium_runs::create::TestChromiumRunKind,
        },
        state::dev_util::{db_client_as_admin, test_db_pool},
    };

    async fn dataset_with_files(static_files_dir: &Utf8Path) -> Uuid {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, ds) = insert_test_chromium_dataset(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let id = *ds.record.id;
        write_test_dataset_files(&test_db_pool(), id, static_files_dir).await;

        id
    }

    fn assert_is_empty(dir: &Utf8Path) {
        assert!(
            dir.read_dir().unwrap().next().is_none(),
            "{dir} is not empty"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_files() {
        let static_files_dir = tempdir().unwrap();
        let static_files_dir = Utf8Path::from_path(static_files_dir.path()).unwrap();

        let id = dataset_with_files(static_files_dir).await;

        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        delete_files_from_db_and_disk(&tx, static_files_dir, id)
            .await
            .unwrap();

        let n_files: i64 = tx
            .query_one_into(&Sql::new(
                "select count(*) from chromium_dataset_raw_file where dataset_id = $1",
                vec![&id],
            ))
            .await
            .unwrap();
        assert_eq!(n_files, 0);

        assert_is_empty(&static_files_dir.join("chromium-datasets"));

        assert_eq!(
            delete_files_from_db_and_disk(&tx, static_files_dir, id).await,
            Err(DbError::ResourceNotFound.into())
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_dataset() {
        let static_files_dir = tempdir().unwrap();
        let static_files_dir = Utf8Path::from_path(static_files_dir.path()).unwrap();

        let id = dataset_with_files(static_files_dir).await;

        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        delete_dataset_from_db_and_disk(&tx, static_files_dir, id)
            .await
            .unwrap();

        assert_is_empty(&static_files_dir.join("chromium-datasets"));

        assert_eq!(
            delete_dataset_from_db_and_disk(&tx, static_files_dir, id).await,
            Err(DbError::ResourceNotFound.into())
        );
    }
}
