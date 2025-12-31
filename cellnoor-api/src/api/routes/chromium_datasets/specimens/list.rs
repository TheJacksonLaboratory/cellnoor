use axum::{extract::State, http::StatusCode};
use cellnoor_models::{chromium_dataset::ChromiumDatasetIdSpecimens, specimen::SpecimenSummary};
use cellnoor_schema::{
    cdna, chip_loadings, chromium_dataset_libraries, chromium_datasets, gem_pools, libraries,
    specimens, suspension_pools, suspension_tagging, suspensions,
};
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn list_specimens(
    dataset_id: ChromiumDatasetIdSpecimens,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<SpecimenSummary>> {
    Ok((
        StatusCode::OK,
        inner_handler(state, user, dataset_id).await?,
    ))
}

impl db::Operation<Vec<SpecimenSummary>> for ChromiumDatasetIdSpecimens {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<SpecimenSummary>, db::Error> {
        let filter = chromium_datasets::id.eq(&self);
        let ordering = specimens::received_at;

        let mut specimens = chromium_datasets_to_pooled_specimens()
            .select(SpecimenSummary::as_select())
            .filter(filter)
            .order_by(ordering)
            .load(db_conn)?;

        // If we couldn't find pooled specimens, then we know they weren't pooled
        if specimens.is_empty() {
            specimens = chromium_datasets_to_unpooled_specimens()
                .select(SpecimenSummary::as_select())
                .filter(filter)
                .order_by(ordering)
                .load(db_conn)?;
        }

        Ok(specimens)
    }
}

#[diesel::dsl::auto_type]
fn chromium_datasets_to_unpooled_specimens() -> _ {
    chromium_datasets::table.inner_join(chromium_dataset_libraries::table.inner_join(
        libraries::table.inner_join(cdna::table.inner_join(gem_pools::table.inner_join(
            chip_loadings::table.inner_join(suspensions::table.inner_join(specimens::table)),
        ))),
    ))
}

#[diesel::dsl::auto_type]
fn chromium_datasets_to_pooled_specimens() -> _ {
    chromium_datasets::table.inner_join(
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
        ),
    )
}

#[cfg(test)]
mod tests {
    use cellnoor_models::{
        chromium_dataset::{
            ChromiumDatasetFilter, ChromiumDatasetIdSpecimens, ChromiumDatasetQuery,
        },
        chromium_run::MAX_SUSPENSIONS_PER_OCM_GEM_POOL,
        tenx_assay::{SampleMultiplexing, TenxAssayFilter},
    };
    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;

    use crate::{
        db::Operation,
        test_state::{Database, N_SUSPENSIONS_PER_POOL, database, root_db_conn},
    };

    async fn n_specimens(sample_multiplexing: SampleMultiplexing, db_conn: &Connection) -> usize {
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

        let ds = db_conn
            .interact(move |db_conn| q.execute(db_conn).unwrap())
            .await
            .unwrap()
            .remove(0);

        let query = ChromiumDatasetIdSpecimens(ds.id());

        let specimens = db_conn
            .interact(move |db_conn| query.execute(db_conn).unwrap())
            .await
            .unwrap();

        specimens.len()
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn chromium_datasets_have_correct_n_specimens(
        #[future] root_db_conn: Connection,
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
}
