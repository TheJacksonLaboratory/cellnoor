use axum::{Json, extract::State};
use cellnoor_types::{
    order_by::OrderBy,
    suspension_pool::{
        SavedSuspensionPoolRecord, SavedTaggedSpecimenRecord, SuspensionPool, SuspensionPoolField,
        SuspensionPoolLinks, SuspensionPoolQuery, TaggedSpecimen,
    },
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, SqlTemplate},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_suspension_pools(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SuspensionPoolQuery>,
) -> Result<Json<Vec<SuspensionPool>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspension_pools(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_suspension_pools(
    tx: &db::Transaction<'_>,
    query: &mut SuspensionPoolQuery,
) -> Result<Vec<SuspensionPool>, ErrorInner> {
    // The first column in the `order by` clause needs to match the `distinct on`
    // clause
    let distinct_on = OrderBy {
        field: SuspensionPoolField::Id,
        desc: true,
    };
    query.order_by.push_front(distinct_on);

    let stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = SqlTemplate::new(stmt).finish_with_query(query)?;

    let pools = if query.detailed {
        let stream = tx.query_stream(sql).await?;
        stream
            .map(|row| row.map(map_detailed_row).unwrap())
            .collect()
            .await
    } else {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(SuspensionPool::from_record).collect().await
    };

    Ok(pools)
}

fn map_detailed_row(row: Row) -> SuspensionPool {
    let record: SavedSuspensionPoolRecord = row.get("suspension_pool");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");

    SuspensionPool::Detailed {
        links: SuspensionPoolLinks::from_id(record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(TaggedSpecimen::from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::SimpleStringOperator,
        specimen::SpecimenPredicate,
        suspension_pool::{SuspensionPool, SuspensionPoolQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::suspension_pools::{
            create::test::insert_test_suspension_pool_and_suspensions,
            index::select_suspension_pools,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_specimen_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            _,
            SuspensionPool::Detailed {
                record: inserted_record,
                specimens,
                ..
            },
        ) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
            .await
            .unwrap()
        else {
            unreachable!("expected detailed suspension pool")
        };

        let mut query = SuspensionPoolQuery::from_filter(
            SpecimenPredicate::Name(
                SimpleStringOperator::Eq(specimens[0].specimen.record().name.clone().into()).into(),
            )
            .into(),
            false,
        );

        let selected_suspension_pools = select_suspension_pools(&tx, &mut query).await.unwrap();

        assert_eq!(selected_suspension_pools.len(), 1);
        assert_eq!(selected_suspension_pools[0].record(), &inserted_record);
    }
}
