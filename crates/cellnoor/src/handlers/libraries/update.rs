use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::library::{LibraryDetailed, LibraryUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        libraries::{
            create::insert_library_preparers, measurements::create::insert_library_measurement,
            show::select_library_by_id,
        },
    },
    state::AppState,
};

pub async fn update_library(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<LibraryUpdate>,
) -> Result<Json<LibraryDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_library_by_id(&tx, id, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_library_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    LibraryUpdate {
        record,
        measurements,
        preparers,
    }: &LibraryUpdate,
) -> Result<LibraryDetailed, ErrorInner> {
    db::update(tx, "library", id, record).await?;

    let preparer_insertions = async {
        if !preparers.is_empty() {
            insert_library_preparers(tx, id, preparers).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_library_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_library_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::library::{LibraryUpdate, NewLibraryRecord};
    use positive::PositiveI32;
    use uuid::Uuid;

    use crate::{
        handlers::libraries::{create::test::insert_test_library, update::update_library_by_id},
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) = insert_test_library(&tx, |_| ()).await.unwrap();
        let id = *inserted.record.id;

        let pre_update = LibraryUpdate {
            record: NewLibraryRecord {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                number_of_sample_index_pcr_cycles: PositiveI32::new(12).unwrap(),
                ..insert_input.record
            },
            measurements: vec![],
            preparers: vec![],
        };

        update_library_by_id(&tx, id, &pre_update).await.unwrap();
    }
}
