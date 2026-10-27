use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::{IdParam, preparers::RemovePreparersError, libraries::show::select_library_by_id},
    state::AppState,
};

pub async fn remove_library_preparers(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: library_id }): Path<IdParam>,
    crate::extract::JsonExtractor(preparers): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, RemovePreparersError> {
    state
        .in_transaction(user, async |tx| {
            delete_library_preparers(tx, library_id, &preparers).await
        })
        .await
}

pub(in super::super::super) async fn delete_library_preparers(
    tx: &db::Transaction<'_>,
    library_id: Uuid,
    preparers: &[Uuid],
) -> Result<(), RemovePreparersError> {
    static DELETE_LIBRARY_PREPARERS: &str =
        "delete from library_preparer where library_id = $1 and prepared_by = any($2)";

    let n = tx
        .execute(&Sql::new(DELETE_LIBRARY_PREPARERS, vec![&library_id, &preparers]))
        .await?;

    if n == 0 {
        return Err(DbError::ResourceNotFound.into());
    }

    if select_library_by_id(tx, library_id).await?.preparers.is_empty() {
        return Err(RemovePreparersError::NoPreparersLeft);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            chromium_runs::create::TestChromiumRunKind,
            people::create::insert_test_person_and_institution,
            preparers::RemovePreparersError,
            libraries::{
                create::{insert_test_library, insert_library_preparers},
                preparers::remove::delete_library_preparers,
                show::select_library_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_preparers() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, library) = insert_test_library(&tx, TestChromiumRunKind::Standard, |_| ()).await.unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        let id = *library.record.id;

        insert_library_preparers(&tx, id, &[person.record.id])
            .await
            .unwrap();
        delete_library_preparers(&tx, id, &library.preparers)
            .await
            .unwrap();

        let selected = select_library_by_id(&tx, id).await.unwrap();

        assert_eq!(selected.preparers, [person.record.id]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cannot_remove_every_preparer() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, library) = insert_test_library(&tx, TestChromiumRunKind::Standard, |_| ()).await.unwrap();

        let error = delete_library_preparers(&tx, *library.record.id, &library.preparers)
            .await
            .unwrap_err();

        std::assert_matches!(error, RemovePreparersError::NoPreparersLeft);
    }
}
