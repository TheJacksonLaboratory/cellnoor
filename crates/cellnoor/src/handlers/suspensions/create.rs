use axum::{Json, extract::State};
use cellnoor_types::suspension::{NewSuspension, NewSuspensionRecord, Suspension};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{
            AsFieldValuePairs, FieldValuePairs, JunctionTable, ToFieldListPlaceholdersParams,
            insert_many_to_many,
        },
    },
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
) -> Result<Json<Suspension>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_suspension(&tx, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_suspension(
    tx: &db::Transaction<'_>,
    NewSuspension {
        record,
        measurements,
        preparers,
    }: NewSuspension,
) -> Result<Suspension, ErrorInner> {
    let id = insert_suspension_record(tx, &record).await?;

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

async fn insert_suspension_record(
    tx: &db::Transaction<'_>,
    new_record: &NewSuspensionRecord,
) -> Result<Uuid, ErrorInner> {
    let fields = new_record.as_field_value_pairs();

    let (field_list, placeholders, params) = fields.to_field_list_and_placeholders_and_params();

    let id = tx
        .query_one_into(
            &format!("insert into suspension {field_list} values {placeholders} returning id"),
            &params,
        )
        .await?;

    Ok(id)
}

pub(super) async fn insert_suspension_preparers(
    tx: &db::Transaction<'_>,
    suspension_id: Uuid,
    preparer_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    insert_many_to_many(
        &tx,
        JunctionTable::SuspensionPreparer,
        ("suspension_id", suspension_id),
        ("prepared_by", preparer_ids),
    )
    .await
}

impl AsFieldValuePairs<7> for NewSuspensionRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, 7> {
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
            ("readable_id", readable_id),
            ("specimen_id", specimen_id),
            ("content", content),
            ("created_at", created_at),
            ("lysis_duration_minutes", lysis_duration_minutes),
            ("target_cell_recovery", target_cell_recovery),
            ("additional_data", additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        specimen::SavedSpecimenRecord,
        suspension::{
            NewSuspension, NewSuspensionRecord, SavedSuspensionRecord, Suspension,
            SuspensionContent,
            measurement::{NewSuspensionMeasurement, SuspensionMeasurementData, Viability},
        },
    };
    use jiff::Timestamp;
    use positive::PositiveBoundedF32;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        handlers::{
            people::create::test::insert_test_person_and_institution,
            projects::show::select_project_by_id,
            specimens::{
                create::test::insert_test_specimen_and_project, show::select_specimen_by_id,
            },
            suspensions::create::insert_suspension,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_suspension_and_specimen<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> (NewSuspension, Suspension)
    where
        F: Fn(NewSuspension) -> NewSuspension,
    {
        let (_, specimen) = insert_test_specimen_and_project(tx, identity).await;
        let SavedSpecimenRecord {
            id: specimen_id,
            submitted_by: person_id,
            ..
        } = specimen.record();

        let mut new = NewSuspension {
            record: NewSuspensionRecord {
                id: NoId {},
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                specimen_id: **specimen_id,
                content: SuspensionContent::Cells,
                created_at: None,
                lysis_duration_minutes: None,
                target_cell_recovery: None,
                additional_data: None,
            },
            measurements: vec![NewSuspensionMeasurement {
                measured_by: *person_id,
                measured_at: Timestamp::now(),
                data: Json(SuspensionMeasurementData::Viability {
                    inner: Viability {
                        value: PositiveBoundedF32::new(0.5).unwrap(),
                    },
                    post_hybridization: false,
                }),
            }],
            preparers: nonempty::NonemptyVec::new(vec![*person_id]).unwrap(),
        };

        new = modify(new);

        let inserted = insert_suspension(tx, new.clone()).await.unwrap();
        (new, inserted)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            NewSuspension {
                record: input_record,
                measurements: input_measurements,
                preparers: input_preparers,
            },
            Suspension::Detailed {
                record: output_record,
                specimen: output_specimen,
                measurements: output_measurements,
                preparers: output_preparers,
                links: _,
            },
        ) = insert_test_suspension_and_specimen(&tx, identity).await
        else {
            panic!("expected Suspension::Detailed");
        };

        let expected_record = SavedSuspensionRecord {
            id: output_record.id,
            readable_id: input_record.readable_id,
            specimen_id: input_record.specimen_id,
            content: input_record.content,
            created_at: input_record.created_at,
            lysis_duration_minutes: input_record.lysis_duration_minutes,
            target_cell_recovery: input_record.target_cell_recovery,
            additional_data: input_record.additional_data,
        };

        assert_eq!(output_record, expected_record);

        // specimen field: verify the join via id
        assert_eq!(
            output_specimen.record(),
            select_specimen_by_id(&tx, input_record.specimen_id)
                .await
                .unwrap()
                .record()
        );

        // measurements: ids are auto-generated; everything else should match
        assert_eq!(output_measurements.len(), input_measurements.len());
        for (out, inp) in output_measurements.iter().zip(input_measurements.iter()) {
            assert_eq!(out.suspension_id, *output_record.id);
            assert_eq!(out.measured_by, inp.measured_by);
            assert_eq!(out.measured_at, inp.measured_at);
            assert_eq!(out.data, inp.data);
        }

        assert_eq!(
            output_preparers,
            input_preparers.into_iter().collect::<Vec<_>>()
        );
    }
}
