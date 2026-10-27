use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::{IdParam, preparers::RemovePreparersError, cdna::show::select_cdna_by_id},
    state::AppState,
};

pub async fn remove_cdna_preparers(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: cdna_id }): Path<IdParam>,
    crate::extract::JsonExtractor(preparers): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, RemovePreparersError> {
    state
        .in_transaction(user, async |tx| {
            delete_cdna_preparers(tx, cdna_id, &preparers).await
        })
        .await
}

pub(in super::super::super) async fn delete_cdna_preparers(
    tx: &db::Transaction<'_>,
    cdna_id: Uuid,
    preparers: &[Uuid],
) -> Result<(), RemovePreparersError> {
    static DELETE_CDNA_PREPARERS: &str =
        "delete from cdna_preparer where cdna_id = $1 and prepared_by = any($2)";

    let n = tx
        .execute(&Sql::new(DELETE_CDNA_PREPARERS, vec![&cdna_id, &preparers]))
        .await?;

    if n == 0 {
        return Err(DbError::ResourceNotFound.into());
    }

    if select_cdna_by_id(tx, cdna_id).await?.preparers.is_empty() {
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
            cdna::{
                create::{insert_test_cdna_and_chromium_run, insert_cdna_preparers},
                preparers::remove::delete_cdna_preparers,
                show::select_cdna_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_preparers() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, cdna) = insert_test_cdna_and_chromium_run(&tx, TestChromiumRunKind::Standard, |_| ()).await.unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        let id = *cdna.record.id;

        insert_cdna_preparers(&tx, id, &[person.record.id])
            .await
            .unwrap();
        delete_cdna_preparers(&tx, id, &cdna.preparers)
            .await
            .unwrap();

        let selected = select_cdna_by_id(&tx, id).await.unwrap();

        assert_eq!(selected.preparers, [person.record.id]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cannot_remove_every_preparer() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, cdna) = insert_test_cdna_and_chromium_run(&tx, TestChromiumRunKind::Standard, |_| ()).await.unwrap();

        let error = delete_cdna_preparers(&tx, *cdna.record.id, &cdna.preparers)
            .await
            .unwrap_err();

        std::assert_matches!(error, RemovePreparersError::NoPreparersLeft);
    }
}
