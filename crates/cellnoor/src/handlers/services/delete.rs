use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn delete_service(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = delete_service_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn delete_service_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    // A trigger drops the principal, which cascades to who has access to the
    // service and to its API keys
    db::delete_by_id(tx, "service", id).await
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::services::{create::test::insert_test_service, delete::delete_service_by_id},
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_service(&tx, |_| ()).await.unwrap();
        delete_service_by_id(&tx, inserted.id).await.unwrap();
    }
}
