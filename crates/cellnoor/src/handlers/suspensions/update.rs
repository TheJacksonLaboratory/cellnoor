use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension::{SuspensionDetailed, SuspensionUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        suspensions::{
            create::insert_suspension_preparers,
            measurements::create::insert_suspension_measurement, show::select_suspension_by_id,
        },
    },
    state::AppState,
};

pub async fn update_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<SuspensionUpdate>,
) -> Result<Json<SuspensionDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_suspension_by_id(&tx, id, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_suspension_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    SuspensionUpdate {
        record,
        measurements,
        preparers,
    }: &SuspensionUpdate,
) -> Result<SuspensionDetailed, ErrorInner> {
    db::update(tx, "suspension", id, record).await?;

    let preparer_insertions = async {
        if !preparers.is_empty() {
            insert_suspension_preparers(tx, id, preparers).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_suspension_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_suspension_by_id(tx, id).await
}

#[cfg(test)]
mod test {

    use cellnoor_types::suspension::{NewSuspensionRecord, SuspensionContent, SuspensionUpdate};
    use jiff::Timestamp;
    use uuid::Uuid;

    use crate::{
        handlers::suspensions::{
            create::test::insert_test_suspension_and_specimen, update::update_suspension_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();
        let id = *inserted.record.id;

        let mut pre_update = SuspensionUpdate {
            record: NewSuspensionRecord {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                content: SuspensionContent::Nuclei,
                ..insert_input.record
            },
            measurements: vec![],
            preparers: vec![],
        };
        pre_update.record.created_at = Some(Timestamp::now());

        update_suspension_by_id(&tx, id, &pre_update).await.unwrap();
    }
}
