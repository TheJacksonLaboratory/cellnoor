use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension::{SuspensionDetailed, SuspensionUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{
        IdParam,
        suspensions::{
            create::insert_suspension_preparers,
            measurements::create::insert_suspension_measurements, show::select_suspension_by_id,
        },
    },
    state::AppState,
};

pub async fn update_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<SuspensionUpdate>,
) -> Result<Json<SuspensionDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| {
            update_suspension_by_id(tx, id, &record).await
        })
        .await
}

async fn update_suspension_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    SuspensionUpdate {
        record,
        measurements,
        preparers,
    }: &SuspensionUpdate,
) -> Result<SuspensionDetailed, DbError> {
    tx.update(id, record).await?;

    let preparer_insertions = async {
        if let Some(preparers) = preparers {
            insert_suspension_preparers(tx, id, preparers).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions =
        insert_suspension_measurements(tx, id, measurements.as_deref().unwrap_or_default());

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_suspension_by_id(tx, id).await
}

#[cfg(test)]
mod test {

    use cellnoor_types::suspension::{NewSuspensionRecord, SuspensionContent, SuspensionUpdate};
    use uuid::Uuid;

    use crate::{
        handlers::suspensions::{
            create::test::insert_test_suspension_and_specimen, update::update_suspension_by_id,
        },
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();
        let id = *inserted.record.id;

        let pre_update = SuspensionUpdate {
            record: NewSuspensionRecord {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                content: SuspensionContent::Nuclei,
                ..insert_input.record
            },
            measurements: None,
            preparers: None,
        };

        update_suspension_by_id(&tx, id, &pre_update).await.unwrap();
    }
}
