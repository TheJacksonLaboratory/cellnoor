use axum::{Json, extract::State};
use cellnoor_types::suspension::{
    NewSuspension, NewSuspensionRecord, SuspensionDetailed, SuspensionField,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::suspensions::{
        measurements::create::insert_suspension_measurement, show::select_suspension_by_id,
    },
    state::AppState,
};

pub async fn create_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewSuspension>,
) -> Result<Json<SuspensionDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_suspension(&tx, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_suspension(
    tx: &db::Transaction<'_>,
    NewSuspension {
        record,
        measurements,
        preparers,
    }: NewSuspension,
) -> Result<SuspensionDetailed, ErrorInner> {
    let id = db::insert_into(tx, "suspension", &record).await?;

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_suspension_measurement(tx, id, m)),
    );

    tokio::try_join!(
        insert_suspension_preparers(tx, id, preparers.as_ref()),
        measurement_insertions
    )?;

    select_suspension_by_id(tx, id).await
}

pub(super) async fn insert_suspension_preparers(
    tx: &db::Transaction<'_>,
    suspension_id: Uuid,
    preparer_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    let preparers: Vec<_> = preparer_ids
        .iter()
        .map(|&prepared_by| NewSuspensionPreparer {
            suspension_id,
            prepared_by,
        })
        .collect();

    futures::future::try_join_all(
        preparers
            .iter()
            .map(|p| db::insert_into_no_returning(tx, "suspension_preparer", p)),
    )
    .await?;

    Ok(())
}

struct NewSuspensionPreparer {
    suspension_id: Uuid,
    prepared_by: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewSuspensionPreparer {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            suspension_id,
            prepared_by,
        } = self;

        [
            ("suspension_id", suspension_id),
            ("prepared_by", prepared_by),
        ]
    }
}

impl AsFieldValuePairs<SuspensionField, 7> for NewSuspensionRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, SuspensionField, 7> {
        use SuspensionField::*;

        let Self {
            id: _,
            readable_id,
            specimen_id,
            content,
            created_at,
            lysis_duration_minutes,
            target_cell_recovery,
            additional_data,
        } = self;

        [
            (ReadableId, readable_id),
            (SpecimenId, specimen_id),
            (Content, content),
            (CreatedAt, created_at),
            (LysisDurationMinutes, lysis_duration_minutes),
            (TargetCellRecovery, target_cell_recovery),
            (AdditionalData, additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        id::NoId,
        suspension::{
            NewSuspension, NewSuspensionRecord, SuspensionContent, SuspensionDetailed,
            measurement::{
                CellViability, NewSuspensionMeasurement, SuspensionMeasurementData,
                SuspensionMeasurementQuantity,
            },
        },
    };
    use jiff::Timestamp;
    use positive::PositiveBoundedF32;
    use postgres_types::Json;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            specimens::create::test::insert_test_specimen_and_project,
            suspensions::create::insert_suspension,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_suspension_and_specimen<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewSuspension, SuspensionDetailed), ErrorInner>
    where
        F: FnMut(&mut NewSuspension),
    {
        let (_, specimen) = insert_test_specimen_and_project(tx, |_| ()).await?;
        let specimen_record = &specimen.record;
        let specimen_id = *specimen_record.id;
        let person_id = specimen_record.submitted_by;

        let mut new = NewSuspension {
            record: NewSuspensionRecord {
                id: NoId {},
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                specimen_id,
                content: SuspensionContent::Cells,
                created_at: None,
                lysis_duration_minutes: None,
                target_cell_recovery: None,
                additional_data: None,
            },
            measurements: vec![NewSuspensionMeasurement {
                measured_by: person_id,
                measured_at: Timestamp::now(),
                data: Json(SuspensionMeasurementData {
                    quantity: SuspensionMeasurementQuantity::Viability(CellViability {
                        value: PositiveBoundedF32::new(0.5).unwrap(),
                    }),
                    post_hybridization: false,
                }),
            }],
            preparers: nonempty::NonemptyVec::new(vec![person_id]).unwrap(),
        };

        modify(&mut new);

        let inserted = insert_suspension(tx, new.clone()).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();
    }
}
