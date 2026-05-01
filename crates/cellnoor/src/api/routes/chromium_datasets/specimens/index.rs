use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, specimen::SpecimenSummary};
use cellnoor_schema::{chromium_datasets, libraries, specimens};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection},
    state::AppState,
};

pub async fn index_chromium_dataset_specimens(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<SpecimenSummary>>, db::Error> {
    select_chromium_dataset_specimens(user.projects(), id, &db_conn)
        .await
        .map(Json)
}

// Technically, we don't need `chromium_datasets::id`, and can get away with
// `chromium_dataset_libraries::dataset_id`. However, our join clauses actually
// include `chromium_datasets` because they reuse the query that gets datasets.
// As such, we filter on `chromium_datasets::id` for clarity
fn filter<'a, QS: 'a>(
    dataset_id: Uuid,
    authorized_projects: &'a AuthProjects,
) -> BoxedFilter<'a, QS>
where
    chromium_datasets::id: SelectableExpression<QS>,
    libraries::project_id: SelectableExpression<QS>,
{
    let mut filter = BoxedFilter::new_true();
    filter = filter.and_condition(chromium_datasets::id.eq(dataset_id));

    if let AuthProjects::Some { project_ids } = authorized_projects {
        filter = filter.and_condition(libraries::project_id.eq_any(project_ids.iter()));
    }

    filter
}

pub async fn select_chromium_dataset_specimens(
    authorized_projects: &AuthProjects,
    dataset_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SpecimenSummary>, db::Error> {
    let ordering = specimens::received_at.desc();

    let select_clause = SpecimenSummary::as_select();

    let (unpooled_specimens, tagged_pooled_specimens, untagged_pooled_specimens): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = tokio::try_join!(
        chromium_datasets_to_unpooled_specimens()
            .select(select_clause)
            .filter(filter(dataset_id, authorized_projects))
            .order_by(ordering)
            .load(&mut db_conn),
        chromium_datasets_to_tagged_pooled_specimens()
            .select(select_clause)
            .filter(filter(dataset_id, authorized_projects))
            .order_by(ordering)
            .load(&mut db_conn),
        chromium_datasets_to_untagged_pooled_specimens()
            .select(select_clause)
            .filter(filter(dataset_id, authorized_projects))
            .order_by(ordering)
            .load(&mut db_conn),
    )?;

    Ok(unpooled_specimens
        .into_iter()
        .chain(tagged_pooled_specimens)
        .chain(untagged_pooled_specimens)
        .collect())
}

#[cfg(test)]
mod tests {
    use cellnoor_models::{
        chromium_dataset::{ChromiumDatasetFilter, ChromiumDatasetQuery},
        chromium_run::MAX_SUSPENSIONS_PER_OCM_GEM_POOL,
        tenx_assay::{SampleMultiplexing, TenxAssayFilter},
    };
    use diesel_async::AsyncPgConnection;
    use rstest::rstest;

    use crate::{
        api::{
            auth::AuthProjects,
            routes::chromium_datasets::{
                index::select_chromium_datasets,
                specimens::index::select_chromium_dataset_specimens,
            },
        },
        db::DbConnection,
        test_state::{Database, N_SUSPENSIONS_PER_POOL, database, root_db_conn},
    };

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn chromium_datasets_have_correct_n_specimens(
        #[future] root_db_conn: DbConnection,
        #[future] _database: &'static Database,
    ) {
        assert_eq!(
            n_specimens(SampleMultiplexing::Singleplex, &root_db_conn).await,
            1
        );

        assert_eq!(
            n_specimens(SampleMultiplexing::OnChipMultiplexing, &root_db_conn).await,
            MAX_SUSPENSIONS_PER_OCM_GEM_POOL
        );

        assert_eq!(
            n_specimens(SampleMultiplexing::FlexBarcode, &root_db_conn).await,
            N_SUSPENSIONS_PER_POOL
        );
    }

    async fn n_specimens(
        sample_multiplexing: SampleMultiplexing,
        db_conn: &AsyncPgConnection,
    ) -> usize {
        let q = ChromiumDatasetQuery::builder()
            .filter(
                ChromiumDatasetFilter::builder()
                    .assay(
                        TenxAssayFilter::builder()
                            .sample_multiplexing(vec![sample_multiplexing])
                            .build(),
                    )
                    .build(),
            )
            .build();

        let ds = &select_chromium_datasets(q, db_conn).await.unwrap()[0];

        let specimens = select_chromium_dataset_specimens(&AuthProjects::All, ds.id(), db_conn)
            .await
            .unwrap();

        specimens.len()
    }
}
