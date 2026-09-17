use axum::{Json, extract::State};
use cellnoor_types::{
    Relation,
    suspension::{NewSuspension, NewSuspensionRecord, SuspensionDetailed, SuspensionField},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::suspensions::{
        measurements::create::insert_suspension_measurements, show::select_suspension_by_id,
    },
    state::AppState,
};

pub async fn create_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewSuspension>,
) -> Result<Json<SuspensionDetailed>, Error> {
    state
        .in_transaction(user, async |tx| insert_suspension(tx, record).await)
        .await
}

async fn insert_suspension(
    tx: &db::Transaction<'_>,
    NewSuspension {
        record,
        measurements,
        preparers,
    }: NewSuspension,
) -> Result<SuspensionDetailed, ErrorInner> {
    let id = tx.insert_returning_id(&record).await?;

    let measurement_insertions = insert_suspension_measurements(tx, id, &measurements);

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

    tx.insert_many(&preparers).await
}

struct NewSuspensionPreparer {
    suspension_id: Uuid,
    prepared_by: Uuid,
}

impl Relation for NewSuspensionPreparer {
    const NAME: &'static str = "suspension_preparer";
}

impl Insert for NewSuspensionPreparer {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            suspension_id,
            prepared_by,
        } = self;

        vec![
            ("suspension_id", suspension_id),
            ("prepared_by", prepared_by),
        ]
    }
}

impl Insert for NewSuspensionRecord {
    type Field = SuspensionField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use SuspensionField::*;

        let Self {
            id: _,
            readable_id,
            specimen_id,
            // Populated by a database trigger
            specimen_received_at: _,
            content,
            created_at,
            lysis_duration_minutes,
            target_cell_recovery,
            additional_data,
        } = self;

        vec![
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
        positive::PositiveBoundedF32,
        suspension::{
            NewSuspension, NewSuspensionRecord, SuspensionContent, SuspensionDetailed,
            measurement::{
                CellViability, NewSuspensionMeasurement, SuspensionMeasurementData,
                SuspensionMeasurementQuantity,
            },
        },
    };
    use jiff::Timestamp;
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
                id: NoId,
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                specimen_id,
                specimen_received_at: specimen.record.received_at,
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
            preparers: cellnoor_types::nonempty::NonemptyVec::new(vec![person_id]).unwrap(),
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
