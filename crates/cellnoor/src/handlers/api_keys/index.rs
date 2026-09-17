use axum::{Json, extract::State};
use cellnoor_types::{
    api_key::{ApiKeyPredicate, ApiKeyQuery, SavedApiKeyRecord},
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_api_keys(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<ApiKeyQuery>,
) -> Result<Json<Vec<SavedApiKeyRecord>>, Error> {
    state
        .in_transaction(user, async |tx| select_api_keys(tx, &query).await)
        .await
}

pub(in super::super) async fn select_api_keys(
    tx: &db::Transaction<'_>,
    query: &ApiKeyQuery,
) -> Result<Vec<SavedApiKeyRecord>, ErrorInner> {
    static SELECT_API_KEYS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    tx.select(&SELECT_API_KEYS, query).await
}

pub(super) async fn select_api_key_record_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SavedApiKeyRecord, ErrorInner> {
    tx.select_one(ApiKeyPredicate::Id(UuidOperator::Eq(id)), select_api_keys)
        .await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        api_key::{ApiKeyField, ApiKeyPredicate, ApiKeyQuery},
        operator::UuidOperator,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::api_keys::{create::test::insert_test_api_key, index::select_api_keys},
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_api_key(&tx, |_| ()).await.unwrap();

        let query =
            ApiKeyQuery::from_filter(ApiKeyPredicate::Id(UuidOperator::Eq(inserted.record.id)));
        let selected = select_api_keys(&tx, &query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, inserted.record.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<ApiKeyField>(&tx, "api_key_public").await;
    }
}
