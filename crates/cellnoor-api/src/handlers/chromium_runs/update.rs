use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::chromium_run::{ChromiumRunDetailed, ChromiumRunUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{IdParam, chromium_runs::show::select_chromium_run_by_id},
    state::AppState,
};

pub async fn update_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<ChromiumRunUpdate>,
) -> Result<Json<ChromiumRunDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| {
            update_chromium_run_by_id(tx, id, &record).await
        })
        .await
}

async fn update_chromium_run_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ChromiumRunUpdate,
) -> Result<ChromiumRunDetailed, DbError> {
    tx.update(id, update).await?;
    select_chromium_run_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{chromium_run::ChromiumRunUpdate, id::NoId};

    use crate::{
        handlers::chromium_runs::{
            TestChromiumRunKind, create::insert_test_chromium_run,
            update::update_chromium_run_by_id,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let run = insert_test_chromium_run(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        let record = &run.record;
        let id = *record.id;

        let update = ChromiumRunUpdate {
            id: NoId,
            readable_id: crate::db::dummy_data::random_name_for("chromium_run"),
            assay_id: record.assay_id,
            run_at: record.run_at,
            run_by: record.run_by,
            succeeded: !record.succeeded,
            additional_data: record.additional_data.clone(),
        };

        update_chromium_run_by_id(&tx, id, &update).await.unwrap();
    }
}
