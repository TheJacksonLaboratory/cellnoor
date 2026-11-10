use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::chromium_dataset::{ChromiumDatasetDetailed, ChromiumDatasetUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{IdParam, chromium_datasets::show::select_chromium_dataset_by_id},
    state::AppState,
};

pub async fn update_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<ChromiumDatasetUpdate>,
) -> Result<Json<ChromiumDatasetDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| {
            update_chromium_dataset_by_id(tx, id, &record).await
        })
        .await
}

async fn update_chromium_dataset_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ChromiumDatasetUpdate,
) -> Result<ChromiumDatasetDetailed, DbError> {
    tx.update(id, update).await?;
    select_chromium_dataset_by_id(tx, id).await
}

#[cfg(test)]
mod tests {
    use cellnoor_types::{chromium_dataset::ChromiumDatasetUpdate, id::NoId};

    use crate::{
        handlers::{
            chromium_datasets::{
                create::insert_test_chromium_dataset, update::update_chromium_dataset_by_id,
            },
            chromium_runs::create::TestChromiumRunKind,
        },
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, ds) = insert_test_chromium_dataset(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        let id = *ds.record.id;

        let update = ChromiumDatasetUpdate {
            id: NoId,
            name: "newname".to_nonempty_string(),
            delivered_at: ds.record.delivered_at,
            additional_data: None,
        };

        update_chromium_dataset_by_id(&tx, id, &update)
            .await
            .unwrap();
    }
}
