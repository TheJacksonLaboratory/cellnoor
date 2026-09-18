use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetPredicateInner, ChromiumDatasetQuery,
    },
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{IdParam, chromium_datasets::index_detailed::select_chromium_datasets_detailed},
    state::AppState,
};

pub async fn show_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<ChromiumDatasetDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| {
            select_chromium_dataset_by_id(tx, &state.public_files_url, id).await
        })
        .await
}

pub(super) async fn select_chromium_dataset_by_id(
    tx: &db::Transaction<'_>,
    raw_files_url: &str,
    id: Uuid,
) -> Result<ChromiumDatasetDetailed, DbError> {
    let query = ChromiumDatasetQuery::from_filter(
        ChromiumDatasetPredicateInner::Id(UuidOperator::Eq(id)).into(),
    );

    let mut results = select_chromium_datasets_detailed(tx, raw_files_url, &query).await?;

    if results.len() != 1 {
        return Err(DbError::ResourceNotFound);
    }

    Ok(results.swap_remove(0))
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db::DbError, handlers::chromium_datasets::show::select_chromium_dataset_by_id,
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn missing_dataset_not_found() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = select_chromium_dataset_by_id(&tx, "", Uuid::new_v4())
            .await
            .unwrap_err();

        assert_eq!(error, DbError::ResourceNotFound);
    }
}
