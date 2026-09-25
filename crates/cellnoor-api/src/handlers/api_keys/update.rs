use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::api_key::{ApiKeyUpdate, SavedApiKeyRecord};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::{IdParam, api_keys::index::select_api_key_record_by_id},
    state::AppState,
};

pub async fn update_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(update): crate::extract::JsonExtractor<ApiKeyUpdate>,
) -> Result<Json<SavedApiKeyRecord>, DbError> {
    state
        .in_transaction(user, async |tx| update_api_key_by_id(tx, id, &update).await)
        .await
}

pub(in super::super) async fn update_api_key_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ApiKeyUpdate,
) -> Result<SavedApiKeyRecord, DbError> {
    tx.update(id, update).await?;

    select_api_key_record_by_id(tx, id).await
}

impl Insert for ApiKeyUpdate {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            description,
            expires_at,
        } = self;

        vec![("description", description), ("expires_at", expires_at)]
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::api_key::ApiKeyUpdate;

    use crate::{
        handlers::api_keys::{create::test::insert_test_api_key, update::update_api_key_by_id},
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, api_key) = insert_test_api_key(&tx, |_| ()).await.unwrap();

        let update = ApiKeyUpdate {
            description: Some("updated".to_nonempty_string()),
            expires_at: None,
        };

        update_api_key_by_id(&tx, api_key.record.id, &update)
            .await
            .unwrap();
    }
}
