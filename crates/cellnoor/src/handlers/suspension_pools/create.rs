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
