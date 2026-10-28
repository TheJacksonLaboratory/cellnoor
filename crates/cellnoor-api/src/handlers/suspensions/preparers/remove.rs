use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::{
        IdParam, preparers::RemovePreparersError, suspensions::show::select_suspension_by_id,
    },
    state::AppState,
};

pub async fn remove_suspension_preparers(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: suspension_id }): Path<IdParam>,
    crate::extract::JsonExtractor(preparers): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, RemovePreparersError> {
    state
        .in_transaction(user, async |tx| {
            delete_suspension_preparers(tx, suspension_id, &preparers).await
        })
        .await
}

pub(in super::super::super) async fn delete_suspension_preparers(
    tx: &db::Transaction<'_>,
    suspension_id: Uuid,
    preparers: &[Uuid],
) -> Result<(), RemovePreparersError> {
    static DELETE_SUSPENSION_PREPARERS: &str =
        "delete from suspension_preparer where suspension_id = $1 and prepared_by = any($2)";

    let n = tx
        .execute(&Sql::new(
            DELETE_SUSPENSION_PREPARERS,
            vec![&suspension_id, &preparers],
        ))
        .await?;

    if n == 0 {
        return Err(DbError::ResourceNotFound.into());
    }

    if select_suspension_by_id(tx, suspension_id)
        .await?
        .preparers
        .is_empty()
    {
        return Err(RemovePreparersError::NoPreparersLeft);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            people::create::insert_test_person_and_institution,
            preparers::RemovePreparersError,
            suspensions::{
                create::{insert_suspension_preparers, insert_test_suspension_and_specimen},
                preparers::remove::delete_suspension_preparers,
                show::select_suspension_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_preparers() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, suspension) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        let id = *suspension.record.id;

        insert_suspension_preparers(&tx, id, &[person.record.id])
            .await
            .unwrap();
        delete_suspension_preparers(&tx, id, &suspension.preparers)
            .await
            .unwrap();

        let selected = select_suspension_by_id(&tx, id).await.unwrap();

        assert_eq!(selected.preparers, [person.record.id]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cannot_remove_every_preparer() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, suspension) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();

        let error = delete_suspension_preparers(&tx, *suspension.record.id, &suspension.preparers)
            .await
            .unwrap_err();

        std::assert_matches!(error, RemovePreparersError::NoPreparersLeft);
    }
}
