use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::DbError,
    handlers::{IdParam, cdna::create::insert_cdna_preparers},
    state::AppState,
};

pub async fn add_cdna_preparers(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: cdna_id }): Path<IdParam>,
    crate::extract::JsonExtractor(preparers): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            insert_cdna_preparers(tx, cdna_id, &preparers).await
        })
        .await
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            cdna::{
                create::{insert_cdna_preparers, insert_test_cdna_and_chromium_run},
                show::select_cdna_by_id,
            },
            chromium_runs::create::TestChromiumRunKind,
            people::create::insert_test_person_and_institution,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn add_preparers() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, cdna) =
            insert_test_cdna_and_chromium_run(&tx, TestChromiumRunKind::Standard, |_| ())
                .await
                .unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        let id = *cdna.record.id;

        // Includes an existing preparer, which must not cause a conflict
        let preparers = [cdna.preparers[0], person.record.id];
        insert_cdna_preparers(&tx, id, &preparers).await.unwrap();

        let selected = select_cdna_by_id(&tx, id).await.unwrap();

        assert!(preparers.iter().all(|p| selected.preparers.contains(p)));
    }
}
