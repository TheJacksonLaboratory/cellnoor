use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::cdna::{Cdna, CdnaUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        cdna::{
            create::insert_cdna_preparers,
            measurements::create::insert_cdna_measurement, show::select_cdna_by_id,
        },
        path::IdParam,
    },
    state::AppState,
};

pub async fn update_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<CdnaUpdate>,
) -> Result<Json<Cdna>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_cdna_by_id(&tx, id, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_cdna_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    CdnaUpdate {
        record,
        measurements,
        preparers,
    }: &CdnaUpdate,
) -> Result<Cdna, ErrorInner> {
    db::update(tx, "cdna", id, record).await?;

    let preparer_insertions = async {
        if !preparers.is_empty() {
            insert_cdna_preparers(tx, id, preparers).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_cdna_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_cdna_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::cdna::{CdnaUpdate, NewCdnaRecord};
    use positive::PositiveI32;
    use uuid::Uuid;

    use crate::{
        handlers::cdna::{create::test::insert_test_cdna, update::update_cdna_by_id},
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) = insert_test_cdna(&tx, |_| ()).await.unwrap();
        let id = *inserted.record().id;

        let pre_update = CdnaUpdate {
            record: NewCdnaRecord {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                n_amplification_cycles: PositiveI32::new(15).unwrap(),
                ..insert_input.record
            },
            measurements: vec![],
            preparers: vec![],
        };

        update_cdna_by_id(&tx, id, &pre_update).await.unwrap();
    }
}
