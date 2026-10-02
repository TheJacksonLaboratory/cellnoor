use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::cdna::{CdnaDetailed, CdnaUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{
        IdParam,
        cdna::{
            create::insert_cdna_preparers, measurements::create::insert_cdna_measurements,
            show::select_cdna_by_id,
        },
    },
    state::AppState,
};

pub async fn update_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<CdnaUpdate>,
) -> Result<Json<CdnaDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| update_cdna_by_id(tx, id, &record).await)
        .await
}

async fn update_cdna_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    CdnaUpdate {
        record,
        measurements,
        preparers,
    }: &CdnaUpdate,
) -> Result<CdnaDetailed, DbError> {
    tx.update(id, record).await?;

    tokio::try_join!(
        insert_cdna_preparers(tx, id, preparers.as_deref().unwrap_or_default()),
        insert_cdna_measurements(tx, id, measurements.as_deref().unwrap_or_default())
    )?;

    select_cdna_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::cdna::{CdnaSimpleFields, CdnaUpdate};

    use crate::{
        handlers::{
            cdna::{create::insert_test_cdna_and_chromium_run, update::update_cdna_by_id},
            chromium_runs::create::TestChromiumRunKind,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) =
            insert_test_cdna_and_chromium_run(&tx, TestChromiumRunKind::Standard, |_| ())
                .await
                .unwrap();

        let id = *inserted.record.id;

        let update = CdnaUpdate {
            record: CdnaSimpleFields {
                readable_id: crate::db::dummy_data::random_name_for("cdna"),
                ..insert_input.simple
            },
            measurements: None,
            preparers: None,
        };

        update_cdna_by_id(&tx, id, &update).await.unwrap();
    }
}
