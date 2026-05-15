use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::{Specimen, creation::NewSpecimen};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        specimens::{
            measurements::create::insert_specimen_measurement, show::select_specimen_by_id,
        },
    },
    state::AppState,
};

pub async fn update_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<Specimen>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_specimen_by_id(&tx, id, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: NewSpecimen,
) -> Result<Specimen, ErrorInner> {
    let (record, measurements) = record.split_for_insertion();

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

    use cellnoor_types::specimen::creation::{
        NewSpecimen,
        block::{BlockFixative, NewBlock},
    };
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

        let (pre_update, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let id = *inserted.record().id;

        let mut inner = pre_update.into_inner();
        inner.readable_id = Uuid::new_v4().to_string().to_nonempty_string();
        inner.measurements = vec![];
        let update = NewSpecimen::Block(NewBlock::Paraffin {
            inner,
            fixative: BlockFixative::FormaldehydeDerivative,
        });

        update_specimen_by_id(&tx, id, update).await.unwrap();
    }
}
