use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_run::{
        ChromiumRun, ChromiumRunField, ChromiumRunQuery, SavedChromiumRunRecord,
        SavedChromiumRunRecordDetailed, SavedGemPoolWithSpecimensRecord,
    },
    order_by::OrderBy,
    tenx_assay::TenxAssay,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, construct_select_stmt},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_chromium_runs(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRun>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_runs(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_chromium_runs(
    tx: &db::Transaction<'_>,
    query: &mut ChromiumRunQuery,
) -> Result<Vec<ChromiumRun>, ErrorInner> {
    // The first column in the `order by` clause needs to match the `distinct on`
    // clause
    let distinct_on = OrderBy {
        field: ChromiumRunField::Id,
        desc: true,
    };
    query.order_by.push_front(distinct_on);

    let runs = if query.detailed {
        let (sql, params) = construct_detailed_select_stmt(query);
        let stream = tx.query_stream(&sql, params).await?;
        stream
            .map(|row| row.map(map_detailed_row).unwrap())
            .collect()
            .await
    } else {
        let (sql, params) = construct_select_stmt(
            "gem_pool_to_specimen",
            &["distinct on ((chromium_run).id) chromium_run"],
            None,
            query,
        );
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(ChromiumRun::from_record).collect().await
    };

    Ok(runs)
}

fn construct_detailed_select_stmt(
    query: &ChromiumRunQuery,
) -> (String, Vec<&(dyn ToSql + Sync)>) {
    construct_select_stmt(
        "gem_pool_to_specimen",
        &[
            "distinct on ((chromium_run).id) chromium_run",
            "tenx_assay",
            // For each gem_pool belonging to this run, aggregate its tagged
            // specimens via array_agg (inner GROUP BY). The outer array()
            // collects one gem_pool_with_specimens row per gem_pool.
            "array(
                 select (
                     gp.gem_pool,
                     array_agg(
                         (gp.specimen, gp.multiplexing_tag, gp.ocm_barcode_id)::tagged_specimen
                     )
                 )::gem_pool_with_specimens
                 from gem_pool_to_specimen as gp
                 where (gp.chromium_run).id = (chromium_run).id
                 group by gp.gem_pool
             ) as gem_pools",
        ],
        Some("chromium_run, tenx_assay"),
        query,
    )
}

fn map_detailed_row(row: Row) -> ChromiumRun {
    let chromium_run: SavedChromiumRunRecord = row.get("chromium_run");
    let assay: TenxAssay = row.get("tenx_assay");
    let gem_pools: Vec<SavedGemPoolWithSpecimensRecord> = row.get("gem_pools");

    ChromiumRun::from_detailed_record_and_gem_pools(
        SavedChromiumRunRecordDetailed {
            chromium_run,
            assay,
        },
        gem_pools,
    )
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        chromium_run::{ChromiumRun, ChromiumRunQuery},
        operator::SimpleStringOperator,
        specimen::SpecimenPredicate,
        tenx_assay::TenxAssayPredicate,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::chromium_runs::{
            create::test::insert_test_chromium_run_standard, index::select_chromium_runs,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_specimen_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Insert a couple unrelated runs to make sure they aren't returned.
        let _ = insert_test_chromium_run_standard(&tx, 1).await.unwrap();
        let _ = insert_test_chromium_run_standard(&tx, 1).await.unwrap();

        // Insert the target run.
        let (target_run, _) = insert_test_chromium_run_standard(&tx, 2).await.unwrap();
        let ChromiumRun::Detailed {
            record: target_record,
            gem_pools,
            ..
        } = &target_run
        else {
            panic!("expected detailed");
        };
        let target_specimen_name = gem_pools[0].specimens[0].specimen.record().name.clone();

        let mut query = ChromiumRunQuery::from_filter(
            SpecimenPredicate::Name(
                SimpleStringOperator::Eq(target_specimen_name.into()).into(),
            )
            .into(),
            false,
        );

        let selected = select_chromium_runs(&tx, &mut query).await.unwrap();

        assert_eq!(selected.len(), 1);
        let ChromiumRun::Compact { record, .. } = &selected[0] else {
            panic!("expected compact");
        };
        assert_eq!(record.id, target_record.chromium_run.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_tenx_assay_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Insert an unrelated run with its own assay.
        let _ = insert_test_chromium_run_standard(&tx, 1).await.unwrap();

        // Insert the target run with its own (different) assay.
        let (target_run, _) = insert_test_chromium_run_standard(&tx, 1).await.unwrap();
        let ChromiumRun::Detailed {
            record: target_record,
            ..
        } = &target_run
        else {
            panic!("expected detailed");
        };
        let target_assay_name: String = target_record.assay.name.clone().into();

        let mut query = ChromiumRunQuery::from_filter(
            TenxAssayPredicate::Name(SimpleStringOperator::Eq(target_assay_name).into()).into(),
            false,
        );

        let selected = select_chromium_runs(&tx, &mut query).await.unwrap();

        assert_eq!(selected.len(), 1);
        let ChromiumRun::Compact { record, .. } = &selected[0] else {
            panic!("expected compact");
        };
        assert_eq!(record.id, target_record.chromium_run.id);
    }
}
