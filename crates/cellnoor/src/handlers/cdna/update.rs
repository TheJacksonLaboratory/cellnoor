use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::cdna::{CdnaDetailed, CdnaUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        cdna::{
            create::insert_cdna_preparers, measurements::create::insert_cdna_measurement,
            show::select_cdna_by_id,
        },
    },
    state::AppState,
};

pub async fn update_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<CdnaUpdate>,
) -> Result<Json<CdnaDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_cdna_by_id(&tx, id, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_cdna_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    CdnaUpdate {
        record,
        measurements,
        preparers,
    }: &CdnaUpdate,
) -> Result<CdnaDetailed, ErrorInner> {
    db::update(tx, "cdna", id, record).await?;

    let preparer_insertions = async {
        if let Some(preparers) = preparers {
            insert_cdna_preparers(tx, id, preparers).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .flatten()
            .map(|m| insert_cdna_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_cdna_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::cdna::{
        CdnaUpdate,
        creation::{CdnaSimpleFields, NewCdna},
    };
    use uuid::Uuid;

    use crate::{
        handlers::cdna::{
            create::test::insert_test_cdna_and_chromium_run, update::update_cdna_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            NewCdna::GeneExpression {
                common: insert_input,
                ..
            },
            inserted,
        ) = insert_test_cdna_and_chromium_run(&tx, |_| ())
            .await
            .unwrap()
        else {
            panic!("we inserted a gene expression cDNA");
        };

        let id = inserted.record.id;

        let pre_update = CdnaUpdate {
            record: CdnaSimpleFields {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                ..insert_input.common.simple
            },
            measurements: None,
            preparers: None,
        };

        update_cdna_by_id(&tx, id, &pre_update).await.unwrap();
    }
}
