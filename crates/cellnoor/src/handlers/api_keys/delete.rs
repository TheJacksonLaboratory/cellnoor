use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::person::{PermissionsToGrant, PermissionsToRevoke};
use deadpool_postgres::tokio_postgres::error::{DbError, SqlState};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn delete_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = delete_api_key_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn delete_api_key_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    db::delete_by_id(tx, "api_key", id).await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        auth::AuthUser,
        handlers::api_keys::{create::test::insert_test_api_key, delete::delete_api_key_by_id},
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, api_key) = insert_test_api_key(&tx, AuthUser::new_as_admin(), |_| ())
            .await
            .unwrap();

        delete_api_key_by_id(&tx, api_key.record.id).await.unwrap();
    }
}
