use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::{SpecimenDetailed, creation::SpecimenUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{
        IdParam,
        specimens::{
            convert_specimen_fields_to_record::convert_specimen_fields_to_record,
            show::select_specimen_by_id,
        },
    },
    state::AppState,
};

pub async fn update_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(update): crate::extract::JsonExtractor<SpecimenUpdate>,
) -> Result<Json<SpecimenDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| update_specimen_by_id(tx, id, update).await)
        .await
}

pub(in super::super) async fn update_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: SpecimenUpdate,
) -> Result<SpecimenDetailed, DbError> {
    tx.update(id, &convert_specimen_fields_to_record(update))
        .await?;

    select_specimen_by_id(tx, id).await
}

#[cfg(test)]
mod test {

    use crate::{
        handlers::specimens::{
            create::insert_test_specimen_and_project, update::update_specimen_by_id,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let id = *inserted.record.id;

        let mut update = insert_input.simple;
        update.readable_id = crate::db::dummy_data::random_name_for("specimen");

        update_specimen_by_id(&tx, id, update).await.unwrap();
    }
}
