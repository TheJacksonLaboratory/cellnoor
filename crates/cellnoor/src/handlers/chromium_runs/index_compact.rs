use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    chromium_run::{
        ChromiumRunCompact, ChromiumRunLinks, ChromiumRunPredicate, ChromiumRunPredicateInner,
        ChromiumRunQuery, SavedChromiumRunRecord,
    },
    id::Id,
};
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_chromium_runs(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRunCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_runs_compact(&tx, &mut query)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_chromium_runs_compact(
    tx: &db::Transaction<'_>,
    query: &mut ChromiumRunQuery,
) -> Result<Vec<ChromiumRunCompact>, ErrorInner> {
    static SELECT_COMPACT_CHROMIUM_RUNS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    let sql = SELECT_COMPACT_CHROMIUM_RUNS.finish_with_query(query);

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(chromium_run_from_record).collect().await)
}

impl AsPredicate for ChromiumRunPredicate {
    fn as_predicate(&self) -> (&'static str, (&'static str, &(dyn ToSql + Sync))) {
        let operator_and_value = match self {
            Self::Specimen(p) => return p.as_predicate(),
            Self::TenxAssay(p) => return p.as_predicate(),
            Self::ChromiumRun(field) => match field {
                ChromiumRunPredicateInner::Id(u)
                | ChromiumRunPredicateInner::AssayId(u)
                | ChromiumRunPredicateInner::RunBy(u) => u.as_sql_operator_and_value(),
                ChromiumRunPredicateInner::ReadableId(s) => s.as_sql_operator_and_value(),
                ChromiumRunPredicateInner::RunAt(t) => t.as_sql_operator_and_value(),
                ChromiumRunPredicateInner::Succeeded(b) => b.as_sql_operator_and_value(),
                ChromiumRunPredicateInner::AdditionalData(j) => j.as_sql_operator_and_value(),
            },
        };

        (self.field_name(), operator_and_value)
    }
}

pub(super) fn chromium_run_links(id: Id) -> ChromiumRunLinks {
    ChromiumRunLinks {
        simple: SimpleLinks::from_str_and_id("/chromium-runs", id),
        suspensions: format!("/chromium-runs/{id}/suspensions"),
        suspension_pools: format!("/chromium-runs/{id}/suspension-pools"),
    }
}

pub fn chromium_run_from_record(record: SavedChromiumRunRecord) -> ChromiumRunCompact {
    ChromiumRunCompact {
        links: chromium_run_links(record.id),
        record,
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        chromium_run::{ChromiumRunPredicateInner, ChromiumRunQuery},
        operator::UuidOperator,
    };

    use crate::{
        handlers::chromium_runs::{
            create::test::insert_test_standard_chromium_run,
            index_compact::select_chromium_runs_compact,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_compact() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_standard_chromium_run(&tx, |_| ())
            .await
            .unwrap();
        let id = *run.record.id;

        let mut query = ChromiumRunQuery::from_filter(
            ChromiumRunPredicateInner::Id(UuidOperator::Eq(id)).into(),
        );
        select_chromium_runs_compact(&tx, &mut query).await.unwrap();
    }
}
