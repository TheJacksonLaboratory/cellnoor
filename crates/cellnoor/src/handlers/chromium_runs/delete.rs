use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::path::IdParam,
    state::AppState,
};

pub async fn delete_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = delete_chromium_run_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn delete_chromium_run_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<(), ErrorInner> {
    db::delete_by_id(tx, "chromium_run", id).await
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::chromium_runs::{
            create::test::insert_test_standard_chromium_run, delete::delete_chromium_run_by_id,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_standard_chromium_run(&tx, |_| ())
            .await
            .unwrap();

        delete_chromium_run_by_id(&tx, *run.record().id)
            .await
            .unwrap();
    }
}
