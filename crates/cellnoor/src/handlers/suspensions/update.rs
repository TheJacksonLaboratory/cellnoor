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
            NewSuspensionRecord, SavedSuspensionRecord, Suspension, SuspensionContent,
            SuspensionUpdate,
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

        let (
            insert_input,
            Suspension::Detailed {
                record: SavedSuspensionRecord { id, .. },
                ..
            },
        ) = insert_test_suspension_and_specimen(&tx, identity).await
        else {
            panic!("expected Suspension::Detailed");
        };

        let mut pre_update = SuspensionUpdate {
            record: NewSuspensionRecord {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                content: SuspensionContent::Nuclei,
                ..insert_input.record
            },
            measurements: vec![],
            preparers: vec![],
        };
        pre_update.record.created_at = Some(Timestamp::now());

        let Suspension::Detailed {
            record: post_update_record,
            specimen: post_update_specimen,
            measurements: post_update_measurements,
            preparers: post_update_preparers,
            links: _,
        } = update_suspension_by_id(&tx, *id, &pre_update)
            .await
            .unwrap()
        else {
            panic!("expected Suspension::Detailed");
        };

        let expected_record = SavedSuspensionRecord {
            id,
            readable_id: pre_update.record.readable_id,
            specimen_id: pre_update.record.specimen_id,
            content: pre_update.record.content,
            created_at: pre_update.record.created_at,
            lysis_duration_minutes: pre_update.record.lysis_duration_minutes,
            target_cell_recovery: pre_update.record.target_cell_recovery,
            additional_data: pre_update.record.additional_data,
        };

        assert_eq!(post_update_record, expected_record);
        assert_eq!(
            *post_update_specimen.record().id,
            post_update_record.specimen_id
        );

        // The update did not add new measurements/preparers, so we still have
        // exactly the one each that the insert helper produced.
        assert_eq!(post_update_measurements.len(), 1);
        assert_eq!(post_update_preparers.len(), 1);
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
