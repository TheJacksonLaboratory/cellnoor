use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    chromium_run::{
        ChromiumRunCompact, ChromiumRunLinks, ChromiumRunQuery, SavedChromiumRunRecord,
    },
    id::Id,
};

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_chromium_runs(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRunCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_runs_compact(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_chromium_runs_compact(
    tx: &db::Transaction<'_>,
    query: &ChromiumRunQuery,
) -> Result<Vec<ChromiumRunCompact>, ErrorInner> {
    static SELECT_COMPACT_CHROMIUM_RUNS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    Ok(tx
        .select(&SELECT_COMPACT_CHROMIUM_RUNS, query)
        .await?
        .into_iter()
        .map(chromium_run_from_record)
        .collect())
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
        chromium_run::{ChromiumRunField, ChromiumRunPredicateInner, ChromiumRunQuery},
        operator::UuidOperator,
        specimen::SpecimenField,
        tenx_assay::TenxAssayField,
    };

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
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

        let query = ChromiumRunQuery::from_filter(
            ChromiumRunPredicateInner::Id(UuidOperator::Eq(id)).into(),
        );
        select_chromium_runs_compact(&tx, &query).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let view = "gem_well_to_specimen";

        tokio::join!(
            ensure_fields_are_selectable::<SpecimenField>(&tx, view),
            ensure_fields_are_selectable::<TenxAssayField>(&tx, view),
            ensure_fields_are_selectable::<ChromiumRunField>(&tx, view),
        );
    }
}
