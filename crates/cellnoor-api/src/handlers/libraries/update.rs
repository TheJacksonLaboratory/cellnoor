use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::library::{LibraryDetailed, LibraryUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{IdParam, libraries::show::select_library_by_id},
    state::AppState,
};

pub async fn update_library(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<LibraryUpdate>,
) -> Result<Json<LibraryDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| update_library_by_id(tx, id, &record).await)
        .await
}

async fn update_library_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &LibraryUpdate,
) -> Result<LibraryDetailed, DbError> {
    tx.update(id, update).await?;

    select_library_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{library::LibraryUpdate, positive::PositiveI32};

    use crate::{
        handlers::{
            chromium_runs::create::TestChromiumRunKind,
            libraries::{create::insert_test_library, update::update_library_by_id},
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) =
            insert_test_library(&tx, TestChromiumRunKind::Standard, |_| ())
                .await
                .unwrap();
        let id = *inserted.record.id;

        let pre_update = LibraryUpdate {
            readable_id: crate::db::dummy_data::random_name_for("library"),
            number_of_sample_index_pcr_cycles: PositiveI32::new(12).unwrap(),
            ..insert_input.record
        };

        update_library_by_id(&tx, id, &pre_update).await.unwrap();
    }
}
