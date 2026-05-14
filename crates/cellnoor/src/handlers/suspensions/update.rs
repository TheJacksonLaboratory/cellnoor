use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension::{NewSuspensionRecord, Suspension, SuspensionUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, ToUpdateClause},
    },
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        suspensions::{
            create::insert_suspension_preparers,
            measurements::create::insert_suspension_measurement, show::select_suspension_by_id,
        },
    },
    state::AppState,
};

pub async fn update_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<SuspensionUpdate>,
) -> Result<Json<Suspension>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_suspension_by_id(&tx, id, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_suspension_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    SuspensionUpdate {
        record,
        measurements,
        preparers,
    }: &SuspensionUpdate,
) -> Result<Suspension, ErrorInner> {
    update_suspension_record(tx, id, &record).await?;

    let preparer_insertions = async {
        if !preparers.is_empty() {
            insert_suspension_preparers(tx, id, preparers.as_ref()).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_suspension_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_suspension_by_id(tx, id).await
}

async fn update_suspension_record(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: &NewSuspensionRecord,
) -> Result<(), ErrorInner> {
    let fields = record.as_field_value_pairs();

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update suspension set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound.into());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        suspension::{
            NewSuspensionRecord, SavedSuspensionRecord, SuspensionContent, SuspensionUpdate,
        },
    };
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
        handlers::suspensions::{
            create::test::insert_test_suspension_and_specimen, update::update_suspension_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let original_suspension = insert_test_suspension_and_specimen(&tx, identity).await;
        let original_record = original_suspension.record();

        let new_readable_id = Uuid::new_v4().to_string().to_nonempty_string();
        let new_created_at = Some(Timestamp::now());

        let new_data = SuspensionUpdate {
            record: NewSuspensionRecord {
                id: NoId {},
                readable_id: new_readable_id.clone(),
                specimen_id: original_record.specimen_id,
                content: SuspensionContent::Nuclei,
                created_at: new_created_at,
                lysis_duration_minutes: None,
                target_cell_recovery: None,
                additional_data: None,
            },
            measurements: vec![],
            preparers: vec![],
        };

        let updated = update_suspension_by_id(&tx, *original_record.id, &new_data)
            .await
            .unwrap();

        assert_eq!(
            updated.record(),
            &SavedSuspensionRecord {
                id: original_record.id,
                readable_id: new_readable_id,
                specimen_id: original_record.specimen_id,
                content: SuspensionContent::Nuclei,
                created_at: new_created_at,
                lysis_duration_minutes: None,
                target_cell_recovery: None,
                additional_data: None,
            }
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let new_data = SuspensionUpdate {
            record: NewSuspensionRecord {
                id: NoId {},
                readable_id: "missing".to_nonempty_string(),
                specimen_id: Uuid::new_v4(),
                content: SuspensionContent::Cells,
                created_at: Some(Timestamp::now()),
                lysis_duration_minutes: None,
                target_cell_recovery: None,
                additional_data: None,
            },
            measurements: vec![],
            preparers: vec![],
        };

        let error = update_suspension_by_id(&tx, Uuid::new_v4(), &new_data)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
