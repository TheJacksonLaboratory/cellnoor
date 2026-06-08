use std::str::FromStr;

use axum::{Json, extract::State};
use cellnoor_types::suspension_pool::{
    MultiplexingTagType, NewSuspensionPool, NewSuspensionPoolCommonFields, NewSuspensionPoolRecord,
    NewTaggedSuspensionPool, SuspensionPoolDetailed, SuspensionPoolField,
};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
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
) -> Result<Json<SuspensionPoolDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_suspension_pool(&tx, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_suspension_pool(
    tx: &db::Transaction<'_>,
    new: &NewSuspensionPool,
) -> Result<SuspensionPoolDetailed, ErrorInner> {
    let multiplexing_tag_type: &str = new.into();
    let multiplexing_tag_type = MultiplexingTagType::from_str(multiplexing_tag_type).ok();

    let (record, measurements, preparer_ids, suspensions) = match new {
        NewSuspensionPool::FlexBarcode(pool)
        | NewSuspensionPool::FlexOligoNucleotideBarcode(pool)
        | NewSuspensionPool::TotalSeqA(pool)
        | NewSuspensionPool::TotalSeqB(pool)
        | NewSuspensionPool::TotalSeqC(pool) => {
            let NewTaggedSuspensionPool {
                common:
                    NewSuspensionPoolCommonFields {
                        record,
                        measurements,
                        preparers,
                    },
                suspensions,
            } = pool;
            (
                record,
                measurements,
                preparers,
                suspensions
                    .iter()
                    .map(|s| (s.suspension_id, Some(&s.tag_id), multiplexing_tag_type))
                    .collect::<Vec<_>>(),
            )
        }
        NewSuspensionPool::Genetic {
            common:
                NewSuspensionPoolCommonFields {
                    record,
                    measurements,
                    preparers,
                },
            suspensions,
        } => (
            record,
            measurements,
            preparers,
            suspensions
                .into_iter()
                .map(|suspension_id| (*suspension_id, None, None))
                .collect(),
        ),
    };

    let id = db::insert_into(tx, "suspension_pool", record).await?;

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

pub(super) async fn insert_suspension_pool_preparers(
    tx: &db::Transaction<'_>,
    pool_id: Uuid,
    preparer_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    let preparers: Vec<_> = preparer_ids
        .iter()
        .map(|&prepared_by| NewSuspensionPoolPreparer {
            pool_id,
            prepared_by,
        })
        .collect();

    futures::future::try_join_all(
        preparers
            .iter()
            .map(|p| db::insert_into_no_returning(tx, "suspension_pool_preparer", p)),
    )
    .await?;

    Ok(())
}

async fn insert_suspension_poolings(
    tx: &db::Transaction<'_>,
    pool_id: Uuid,
    suspensions: &[(Uuid, Option<&NonemptyString>, Option<MultiplexingTagType>)],
) -> Result<(), ErrorInner> {
    let poolings: Vec<_> = suspensions
        .iter()
        .map(|(suspension_id, tag_id, tag_type)| NewSuspensionPooling {
            pool_id,
            suspension_id: *suspension_id,
            tag_id: *tag_id,
            tag_type: *tag_type,
        })
        .collect();

    futures::future::try_join_all(
        poolings
            .iter()
            .map(|p| db::insert_into_no_returning(tx, "suspension_pooling", p)),
    )
    .await?;

    Ok(())
}

struct NewSuspensionPoolPreparer {
    pool_id: Uuid,
    prepared_by: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewSuspensionPoolPreparer {
    fn as_field_value_pairs(&'_ self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            pool_id,
            prepared_by,
        } = self;

        [("pool_id", pool_id), ("prepared_by", prepared_by)]
    }
}

struct NewSuspensionPooling<'a> {
    pool_id: Uuid,
    suspension_id: Uuid,
    tag_id: Option<&'a NonemptyString>,
    tag_type: Option<MultiplexingTagType>,
}

impl AsFieldValuePairs<&'static str, 4> for NewSuspensionPooling<'_> {
    fn as_field_value_pairs(&'_ self) -> FieldValuePairs<'_, &'static str, 4> {
        let Self {
            pool_id,
            suspension_id,
            tag_id,
            tag_type,
        } = self;

        [
            ("pool_id", pool_id),
            ("suspension_id", suspension_id),
            ("tag_id", tag_id),
            ("tag_type", tag_type),
        ]
    }
}

impl AsFieldValuePairs<SuspensionPoolField, 4> for NewSuspensionPoolRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, SuspensionPoolField, 4> {
        use SuspensionPoolField::*;

        let Self {
            id: _,
            readable_id,
            name,
            pooled_at,
            additional_data,
        } = self;

        [
            (ReadableId, readable_id),
            (Name, name),
            (PooledAt, pooled_at),
            (AdditionalData, additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {

    use std::collections::HashSet;

    use cellnoor_types::{
        id::NoId,
        suspension::measurement::Viability,
        suspension_pool::{
            NewSuspensionPool, NewSuspensionPoolCommonFields, NewSuspensionPoolRecord,
            NewTaggedSuspensionPool, SuspensionPoolDetailed, TaggedSuspension,
            measurement::{NewSuspensionPoolMeasurement, SuspensionPoolMeasurementData},
        },
    };
    use jiff::Timestamp;
    use nonempty::{NonemptyBoundedVec, NonemptyVec};
    use positive::PositiveBoundedF32;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            multiplexing_tags::create::tests::insert_test_multiplexing_tag,
            specimens::create::test::insert_test_specimen_and_project,
            suspension_pools::create::insert_suspension_pool,
            suspensions::create::test::insert_test_suspension_and_specimen,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_suspension_pool_and_suspensions<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewSuspensionPool, SuspensionPoolDetailed), ErrorInner>
    where
        F: FnMut(&mut NewSuspensionPool),
    {
        let (_, suspension1) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
        let (_, suspension2) = insert_test_suspension_and_specimen(tx, |_| ()).await?;

        let multiplexing_tag1_id = insert_test_multiplexing_tag(tx).await?;
        let multiplexing_tag2_id = insert_test_multiplexing_tag(tx).await?;

        let suspension1_id = *suspension1.record.id;
        let suspension2_id = *suspension2.record.id;

        let person_id = suspension1.specimen.record.submitted_by;

        let mut new = NewSuspensionPool::FlexBarcode(NewTaggedSuspensionPool {
            common: NewSuspensionPoolCommonFields {
                record: NewSuspensionPoolRecord {
                    id: NoId {},
                    readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                    name: "pool".to_nonempty_string(),
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
                preparers: NonemptyVec::new(vec![person_id]).unwrap(),
            },
            suspensions: NonemptyBoundedVec::new(vec![
                TaggedSuspension {
                    suspension_id: suspension1_id,
                    tag_id: multiplexing_tag1_id.tag_id,
                },
                TaggedSuspension {
                    suspension_id: suspension2_id,
                    tag_id: multiplexing_tag2_id.tag_id,
                },
            ])
            .unwrap(),
        });

        modify(&mut new);

        let inserted = insert_suspension_pool(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Insert a couple unrelated test specimens to make sure the suspension pool
        // query doesn't pick them up

        let mut unrelated_specimen_ids = HashSet::new();

        for _ in 0..4 {
            let id = insert_test_specimen_and_project(&tx, |_| ())
                .await
                .unwrap()
                .1
                .record
                .id;
            unrelated_specimen_ids.insert(*id);
        }

        let (_, inserted) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
            .await
            .unwrap();

        let SuspensionPoolDetailed {
            record,
            links: _,
            specimens,
            measurements,
            preparers,
        } = inserted;

        assert_eq!(specimens.len(), 2);

        let multiplexing_tags = specimens
            .iter()
            .map(|s| s.multiplexing_tag.clone())
            .collect::<Vec<_>>();
        assert_ne!(multiplexing_tags[0], multiplexing_tags[1]);

        assert!(
            unrelated_specimen_ids
                .is_disjoint(&specimens.iter().map(|s| *s.specimen.record.id).collect())
        );

        assert_eq!(measurements.len(), 1);
        assert_eq!(measurements[0].pool_id, *record.id);
        assert_eq!(
            measurements[0].data.0,
            SuspensionPoolMeasurementData::Viability(Viability {
                value: PositiveBoundedF32::new(0.5).unwrap()
            })
        );

        assert_eq!(preparers.len(), 1);
    }
}
