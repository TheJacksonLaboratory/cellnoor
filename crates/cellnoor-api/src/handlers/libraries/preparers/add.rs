use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::DbError,
    handlers::{IdParam, libraries::create::insert_library_preparers},
    state::AppState,
};

pub async fn add_library_preparers(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: library_id }): Path<IdParam>,
    crate::extract::JsonExtractor(preparers): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            insert_library_preparers(tx, library_id, &preparers).await
        })
        .await
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            chromium_runs::create::TestChromiumRunKind,
            libraries::{
                create::{insert_library_preparers, insert_test_library},
                show::select_library_by_id,
            },
            people::create::insert_test_person_and_institution,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn add_preparers() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, library) = insert_test_library(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        let id = *library.record.id;

        // Includes an existing preparer, which must not cause a conflict
        let preparers = [library.preparers[0], person.record.id];
        insert_library_preparers(&tx, id, &preparers).await.unwrap();

        let selected = select_library_by_id(&tx, id).await.unwrap();

        assert!(preparers.iter().all(|p| selected.preparers.contains(p)));
    }
}
