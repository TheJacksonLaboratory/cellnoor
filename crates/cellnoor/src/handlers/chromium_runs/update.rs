use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::chromium_run::{ChromiumRun, ChromiumRunUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{chromium_runs::show::select_chromium_run_by_id, path::IdParam},
    state::AppState,
};

pub async fn update_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<ChromiumRunUpdate>,
) -> Result<Json<ChromiumRun>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_chromium_run_by_id(&tx, id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_chromium_run_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ChromiumRunUpdate,
) -> Result<ChromiumRun, ErrorInner> {
    db::update(tx, "chromium_run", id, update).await?;
    select_chromium_run_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{chromium_run::ChromiumRunUpdate, id::NoId};
    use uuid::Uuid;

    use crate::{
        handlers::chromium_runs::{
            create::test::insert_test_standard_chromium_run, update::update_chromium_run_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_standard_chromium_run(&tx, |_| ()).await.unwrap();
        let record = run.record();
        let id = *record.id;

        let update = ChromiumRunUpdate {
            id: NoId {},
            readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
            assay_id: record.assay_id,
            run_at: record.run_at,
            run_by: record.run_by,
            succeeded: !record.succeeded,
            additional_data: record.additional_data.clone(),
        };

        update_chromium_run_by_id(&tx, id, &update).await.unwrap();
    }
}
