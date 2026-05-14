use axum::{Json, extract::State};
use cellnoor_types::suspension_pool::{
    NewSuspensionPool, NewSuspensionPoolRecord, SuspensionPool, TaggedSuspension,
};
use postgres_types::ToSql;
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
    handlers::suspension_pools::{
        measurements::create::insert_suspension_pool_measurement,
        show::select_suspension_pool_by_id,
    },
    state::AppState,
};

pub async fn create_suspension_pool(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewSuspensionPool>,
) -> Result<Json<SuspensionPool>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_suspension_pool(&tx, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_suspension_pool(
    tx: &db::Transaction<'_>,
    new: NewSuspensionPool,
) -> Result<SuspensionPool, ErrorInner> {
    let (record, measurements, preparer_ids, suspensions) = match new {
        NewSuspensionPool::ExogenousTag {
            inner,
            measurements,
            preparer_ids,
            suspensions,
        } => (
            inner,
            measurements,
            preparer_ids,
            suspensions
                .into_iter()
                .map(
                    |TaggedSuspension {
                         suspension_id,
                         tag_id,
                     }| (suspension_id, Some(tag_id)),
                )
                .collect::<Vec<_>>(),
        ),
        NewSuspensionPool::Genetic {
            inner,
            measurements,
            preparer_ids,
            suspensions,
        } => (
            inner,
            measurements,
            preparer_ids,
            suspensions
                .into_iter()
                .map(|suspension_id| (suspension_id, None))
                .collect::<Vec<_>>(),
        ),
    };

    let id = insert_suspension_pool_record(tx, &record).await?;

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_suspension_pool_measurement(tx, id, m)),
    );

    tokio::try_join!(
        insert_suspension_pool_preparers(tx, id, preparer_ids.as_ref()),
        insert_suspension_poolings(tx, id, &suspensions),
        measurement_insertions,
    )?;

    select_suspension_pool_by_id(tx, id).await
}

async fn insert_suspension_pool_record(
    tx: &db::Transaction<'_>,
    new_record: &NewSuspensionPoolRecord,
) -> Result<Uuid, ErrorInner> {
    let fields = new_record.as_field_value_pairs();

    let (field_list, placeholders, params) = fields.to_field_list_and_placeholders_and_params();

    let id = tx
        .query_one_into(
            &format!("insert into suspension_pool {field_list} values {placeholders} returning id"),
            &params,
        )
        .await?;

    Ok(id)
}

pub(super) async fn insert_suspension_pool_preparers(
    tx: &db::Transaction<'_>,
    pool_id: Uuid,
    preparer_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    insert_many_to_many(
        &tx,
        JunctionTable::SuspensionPoolPreparer,
        ("pool_id", pool_id),
        ("prepared_by", preparer_ids),
    )
    .await
}

pub(super) async fn insert_suspension_poolings(
    tx: &db::Transaction<'_>,
    pool_id: Uuid,
    suspensions: &[(Uuid, Option<Uuid>)],
) -> Result<(), ErrorInner> {
    if suspensions.is_empty() {
        return Ok(());
    }

    let mut values_clause = String::with_capacity(suspensions.len() * 16);
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(1 + 2 * suspensions.len());
    params.push(&pool_id);

    for (i, (suspension_id, tag_id)) in suspensions.iter().enumerate() {
        if i > 0 {
            values_clause.push(',');
        }

        let suspension_param = params.len() + 1;
        let tag_param = params.len() + 2;

        values_clause.push_str(&format!("($1, ${suspension_param}, ${tag_param})"));

        params.push(suspension_id);
        params.push(tag_id);
    }

    let stmt = format!(
        "insert into suspension_pooling (pool_id, suspension_id, tag_id) values {values_clause} \
         on conflict do nothing"
    );

    tx.execute(&stmt, &params).await?;

    Ok(())
}

impl AsFieldValuePairs<5> for NewSuspensionPoolRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, 5> {
        let Self {
            id: _,
            readable_id,
            name,
            multiplexing_type,
            pooled_at,
            additional_data,
        } = self;

        [
            ("readable_id", readable_id),
            ("name", name),
            ("multiplexing_type", multiplexing_type),
            ("pooled_at", pooled_at),
            ("additional_data", additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        suspension::measurement::Viability,
        suspension_pool::{
            NewSuspensionPool, NewSuspensionPoolRecord, SavedSuspensionPoolRecord, SuspensionPool,
            measurement::{NewSuspensionPoolMeasurement, SuspensionPoolMeasurementData},
        },
    };
    use jiff::Timestamp;
    use nonempty::NonemptyVec;
    use positive::PositiveBoundedF32;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        handlers::{
            people::create::test::insert_test_person_and_institution,
            suspension_pools::create::insert_suspension_pool,
            suspensions::create::test::insert_test_suspension_and_specimen,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_pool_and_suspension<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> (NewSuspensionPool, SuspensionPool)
    where
        F: Fn(NewSuspensionPool) -> NewSuspensionPool,
    {
        let (_, suspension) = insert_test_suspension_and_specimen(tx, identity).await;
        let (_, person) = insert_test_person_and_institution(tx, identity).await;
        let suspension_id = *suspension.record().id;
        let person_id = *person.record.id;

        let mut new = NewSuspensionPool::Genetic {
            inner: NewSuspensionPoolRecord {
                id: NoId {},
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                name: "pool".to_nonempty_string(),
                multiplexing_type: "genetic".to_owned(),
                pooled_at: Timestamp::now(),
                additional_data: None,
            },
            measurements: vec![NewSuspensionPoolMeasurement {
                measured_by: person_id,
                measured_at: Timestamp::now(),
                data: Json(SuspensionPoolMeasurementData::Viability(Viability {
                    value: PositiveBoundedF32::new(0.5).unwrap(),
                })),
            }],
            preparer_ids: NonemptyVec::new(vec![person_id]).unwrap(),
            suspensions: NonemptyVec::new(vec![suspension_id]).unwrap(),
        };

        new = modify(new);

        let inserted = insert_suspension_pool(tx, new.clone()).await.unwrap();
        (new, inserted)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (input, inserted) = insert_test_pool_and_suspension(&tx, identity).await;

        let SuspensionPool::Detailed {
            record: output_record,
            specimens: output_specimens,
            measurements: output_measurements,
            preparers: output_preparers,
            links: _,
        } = inserted
        else {
            panic!("expected SuspensionPool::Detailed");
        };

        let NewSuspensionPool::Genetic {
            inner: input_record,
            measurements: input_measurements,
            preparer_ids: input_preparer_ids,
            suspensions: input_suspensions,
        } = input
        else {
            panic!("helper only uses Genetic");
        };

        let expected_record = SavedSuspensionPoolRecord {
            id: output_record.id,
            readable_id: input_record.readable_id,
            name: input_record.name,
            multiplexing_type: input_record.multiplexing_type,
            pooled_at: input_record.pooled_at,
            additional_data: input_record.additional_data,
        };

        assert_eq!(output_record, expected_record);

        // Genetic pool: one tagged_specimen per suspension, all with tag = None.
        assert_eq!(output_specimens.len(), input_suspensions.as_ref().len());
        for tagged in &output_specimens {
            assert_eq!(tagged.tag, None);
        }

        // Measurements: ids are auto-generated; everything else should match.
        assert_eq!(output_measurements.len(), input_measurements.len());
        for (out, inp) in output_measurements.iter().zip(input_measurements.iter()) {
            assert_eq!(out.pool_id, *output_record.id);
            assert_eq!(out.measured_by, inp.measured_by);
            assert_eq!(out.measured_at, inp.measured_at);
            assert_eq!(out.data, inp.data);
        }

        assert_eq!(
            output_preparers,
            input_preparer_ids.into_iter().collect::<Vec<_>>()
        );
    }
}
