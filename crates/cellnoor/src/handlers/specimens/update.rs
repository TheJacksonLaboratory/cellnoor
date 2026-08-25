use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::{SpecimenDetailed, creation::NewSpecimen};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        specimens::{
            measurements::create::insert_specimen_measurement, show::select_specimen_by_id,
            split_new_specimen_for_insertion::split_new_specimen_for_insertion,
        },
    },
    state::AppState,
};

pub async fn update_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<SpecimenDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_specimen_by_id(&tx, id, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: NewSpecimen,
) -> Result<SpecimenDetailed, ErrorInner> {
    let (record, measurements) = split_new_specimen_for_insertion(record);

    db::update(tx, "specimen", id, &record).await?;

    futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_specimen_measurement(tx, id, m)),
    )
    .await?;

    select_specimen_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use crate::{
        handlers::specimens::{
            create::test::insert_test_specimen_and_project, update::update_specimen_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (mut update, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let id = *inserted.record.id;

        update.readable_id = Uuid::new_v4().to_string().to_nonempty_string();
        update.measurements = vec![];

        update_specimen_by_id(&tx, id, update).await.unwrap();
    }
}
