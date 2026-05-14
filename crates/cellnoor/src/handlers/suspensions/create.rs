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
            NewSuspension, NewSuspensionRecord, Suspension, SuspensionContent,
            measurement::{NewSuspensionMeasurement, SuspensionMeasurementData, Viability},
        },
    };
    use jiff::Timestamp;
    use positive::PositiveBoundedF32;
    use postgres_types::Json;
    use uuid::Uuid;

    use crate::{
        db,
        handlers::{
            people::create::test::insert_test_person_and_institution,
            specimens::create::test::insert_test_specimen_and_project,
            suspensions::create::insert_suspension,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_suspension_and_specimen<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> Suspension
    where
        F: Fn(NewSuspension) -> NewSuspension,
    {
        let specimen = insert_test_specimen_and_project(tx, identity).await;
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
                created_at: Some(Timestamp::now()),
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

        insert_suspension(tx, new).await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_suspension_and_specimen(&tx, identity).await;
    }
}
