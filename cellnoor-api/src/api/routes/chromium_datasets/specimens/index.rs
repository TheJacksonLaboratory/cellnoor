use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, specimen::SpecimenSummary};
use cellnoor_schema::{
    cdna, chip_loadings, chromium_dataset_libraries, gem_pools, libraries, specimens,
    suspension_pools, suspension_tagging, suspensions,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_chromium_dataset_specimens(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<SpecimenSummary>>, db::Error> {
    select_chromium_dataset_specimens(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

async fn select_chromium_dataset_specimens(
    authorized_projects: &AuthProjects,
    dataset_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SpecimenSummary>, db::Error> {
    let filter = chromium_dataset_libraries::dataset_id.eq(dataset_id);
    let ordering = specimens::received_at;

    let query = chromium_datasets_to_pooled_specimens()
        .select(SpecimenSummary::as_select())
        .filter(filter)
        .order_by(ordering);

    let mut specimens = match authorized_projects {
        AuthProjects::All => query.load(&mut db_conn).await?,
        AuthProjects::Restricted(projects) => {
            query
                .filter(libraries::project_id.eq_any(projects.iter()))
                .load(&mut db_conn)
                .await?
        }
    };

    // If we couldn't find pooled specimens, then we know they weren't pooled
    if specimens.is_empty() {
        let query = chromium_datasets_to_unpooled_specimens()
            .select(SpecimenSummary::as_select())
            .filter(filter)
            .order_by(ordering);

        specimens = match authorized_projects {
            AuthProjects::All => query.load(&mut db_conn).await?,
            AuthProjects::Restricted(projects) => {
                query
                    .filter(libraries::project_id.eq_any(projects.iter()))
                    .load(&mut db_conn)
                    .await?
            }
        };
    };

    Ok(specimens)
}

#[diesel::dsl::auto_type]
fn chromium_datasets_to_unpooled_specimens() -> _ {
    chromium_dataset_libraries::table.inner_join(libraries::table.inner_join(
        cdna::table.inner_join(gem_pools::table.inner_join(
            chip_loadings::table.inner_join(suspensions::table.inner_join(specimens::table)),
        )),
    ))
}

#[diesel::dsl::auto_type]
fn chromium_datasets_to_pooled_specimens() -> _ {
    chromium_dataset_libraries::table.inner_join(
        libraries::table.inner_join(
            cdna::table.inner_join(
                gem_pools::table.inner_join(
                    chip_loadings::table.inner_join(
                        suspension_pools::table.inner_join(
                            suspension_tagging::table
                                .inner_join(suspensions::table.inner_join(specimens::table)),
                        ),
                    ),
                ),
            ),
        ),
    )
}

// #[cfg(test)]
// mod tests {
//     use cellnoor_models::{
//         chromium_dataset::{ChromiumDatasetFilter, ChromiumDatasetQuery},
//         chromium_run::MAX_SUSPENSIONS_PER_OCM_GEM_POOL,
//         tenx_assay::{SampleMultiplexing, TenxAssayFilter},
//     };

//     use rstest::rstest;

//     use crate::test_state::{Database, N_SUSPENSIONS_PER_POOL, database,
// root_db_conn};

//     async fn n_specimens(sample_multiplexing: SampleMultiplexing, db_conn:
// &Connection) -> usize {         let q = ChromiumDatasetQuery::builder()
//             .filter(
//                 ChromiumDatasetFilter::builder()
//                     .assay(
//                         TenxAssayFilter::builder()
//                             .sample_multiplexing(vec![sample_multiplexing])
//                             .build(),
//                     )
//                     .build(),
//             )
//             .build();

//         let ds = db_conn
//             .interact(move |db_conn| q.execute(&mut db_conn).unwrap())
//             .await
//             .unwrap()
//             .remove(0);

//         let query = ChromiumDatasetIdSpecimens(ds.id());

//         let specimens = db_conn
//             .interact(move |db_conn| query.execute(&mut db_conn).unwrap())
//             .await
//             .unwrap();

//         specimens.len()
//     }

//     #[rstest]
//     #[awt]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn chromium_datasets_have_correct_n_specimens(
//         #[future] root_db_conn: Connection,
//         #[future] _database: &'static Database,
//     ) {
//         assert_eq!(
//             n_specimens(SampleMultiplexing::Singleplex, &root_db_conn).await,
//             1
//         );

//         assert_eq!(
//             n_specimens(SampleMultiplexing::OnChipMultiplexing,
// &root_db_conn).await,             MAX_SUSPENSIONS_PER_OCM_GEM_POOL
//         );

//         assert_eq!(
//             n_specimens(SampleMultiplexing::FlexBarcode,
// &root_db_conn).await,             N_SUSPENSIONS_PER_POOL
//         );
//     }
// }
