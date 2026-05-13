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
    use cellnoor_types::{
        UuidOperator,
        id::NoId,
        specimen::SpecimenPredicate,
        suspension::{
            NewSuspension, NewSuspensionRecord, SuspensionContent, SuspensionQuery,
            measurement::{NewSuspensionMeasurement, SuspensionMeasurementData, Viability},
        },
    };
    use jiff::Timestamp;
    use positive::PositiveF32;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        handlers::{
            projects::create::{insert_project, test::new_project},
            specimens::create::{insert_specimen, test::new_specimen},
            suspensions::{create::insert_suspension, index::select_suspensions},
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub fn new_suspension(specimen_id: Uuid) -> NewSuspension {
        NewSuspension {
            record: NewSuspensionRecord {
                id: NoId {},
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                specimen_id,
                content: SuspensionContent::Cells,
                created_at: Some(Timestamp::now()),
                lysis_duration_minutes: None,
                target_cell_recovery: None,
                additional_data: None,
            },
            measurements: vec![NewSuspensionMeasurement {
                measured_by: Uuid::nil(),
                measured_at: Timestamp::now(),
                data: Json(SuspensionMeasurementData::Viability {
                    inner: Viability {
                        value: PositiveF32::new(0.5).unwrap(),
                    },
                    post_hybridization: false,
                }),
            }],
            preparers: nonempty::NonemptyVec::new(vec![Uuid::nil()]).unwrap(),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_and_select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let project = insert_project(&tx, &new_project()).await.unwrap();
        let specimen = insert_specimen(&tx, new_specimen(*project.record().id))
            .await
            .unwrap();

        let new = new_suspension(*specimen.record().id);
        let inserted = insert_suspension(&tx, new).await.unwrap();

        let suspensions_from_query = select_suspensions(
            &tx,
            &SuspensionQuery::from_filter(
                SpecimenPredicate::Id(UuidOperator::Eq(*specimen.record().id)).into(),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(suspensions_from_query[0].record(), inserted.record());
    }
}
