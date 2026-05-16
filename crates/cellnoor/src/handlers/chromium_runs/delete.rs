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
    use cellnoor_types::chromium_run::ChromiumRun;
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::chromium_runs::{
            create::test::insert_test_chromium_run_standard, delete::delete_chromium_run_by_id,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_cascades_gem_pools_and_chip_loadings() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (run, _) = insert_test_chromium_run_standard(&tx, 3).await.unwrap();
        let ChromiumRun::Detailed { record, .. } = &run else {
            panic!("expected detailed");
        };
        let run_id = *record.chromium_run.id;

        delete_chromium_run_by_id(&tx, run_id).await.unwrap();

        let gp_count: i64 = tx
            .query_one(
                "select count(*) from gem_pool where chromium_run_id = $1",
                &[&run_id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(gp_count, 0);

        let cl_count: i64 = tx
            .query_one(
                "select count(*) from chip_loading where gem_pool_id in (select id from gem_pool \
                 where chromium_run_id = $1)",
                &[&run_id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(cl_count, 0);
    }
}
