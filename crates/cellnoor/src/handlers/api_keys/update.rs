use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::api_key::{ApiKeyRecord, ApiKeyUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::{
        IdParam, api_keys::index::select_api_key_record_by_id, people::create::provision_db_user,
    },
    state::AppState,
};

pub async fn update_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(update): Json<ApiKeyUpdate>,
) -> Result<Json<ApiKeyRecord>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_api_key_by_id(&tx, id, &update).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn update_api_key_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ApiKeyUpdate,
) -> Result<ApiKeyRecord, ErrorInner> {
    db::update(tx, "api_key", id, update).await?;

    let (_, record) = tokio::try_join!(
        provision_db_user(
            tx,
            id,
            &update.permissions_to_grant,
            &update.permissions_to_revoke
        ),
        select_api_key_record_by_id(tx, id),
    )?;

    Ok(record)
}

impl AsFieldValuePairs<&'static str, 2> for ApiKeyUpdate {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            description,
            expires_at,
            permissions_to_grant: _,
            permissions_to_revoke: _,
        } = self;

        [("description", description), ("expires_at", expires_at)]
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::api_key::ApiKeyUpdate;

    use crate::{
        auth::AuthUser,
        handlers::api_keys::{create::test::insert_test_api_key, update::update_api_key_by_id},
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, api_key) = insert_test_api_key(&tx, AuthUser::new_as_admin(), |_| ())
            .await
            .unwrap();

        let update = ApiKeyUpdate {
            description: Some("updated".to_nonempty_string()),
            expires_at: None,
            permissions_to_grant: vec![].into(),
            permissions_to_revoke: vec![].into(),
        };

        update_api_key_by_id(&tx, api_key.record.id, &update)
            .await
            .unwrap();
    }
}
