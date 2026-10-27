use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::DbError,
    handlers::{IdParam, suspensions::create::insert_suspension_preparers},
    state::AppState,
};

pub async fn add_suspension_preparers(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: suspension_id }): Path<IdParam>,
    crate::extract::JsonExtractor(preparers): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            insert_suspension_preparers(tx, suspension_id, &preparers).await
        })
        .await
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            people::create::insert_test_person_and_institution,
            suspensions::{
                create::{insert_suspension_preparers, insert_test_suspension_and_specimen},
                show::select_suspension_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn add_preparers() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, suspension) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        let id = *suspension.record.id;

        // Includes an existing preparer, which must not cause a conflict
        let preparers = [suspension.preparers[0], person.record.id];
        insert_suspension_preparers(&tx, id, &preparers)
            .await
            .unwrap();

        let selected = select_suspension_by_id(&tx, id).await.unwrap();

        assert!(preparers.iter().all(|p| selected.preparers.contains(p)));
    }
}
