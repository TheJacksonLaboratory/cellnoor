use std::cmp::Ordering;

use axum::{Json, extract::State};
use cellnoor_models::chromium_dataset::{
    ChromiumDataset, ChromiumDatasetFilter, ChromiumDatasetOrderBy, ChromiumDatasetQuery,
};
use cellnoor_schema::{
    cdna::dsl::cdna,
    chip_loadings::dsl::chip_loadings,
    chromium_dataset_libraries::dsl::chromium_dataset_libraries,
    chromium_datasets::dsl::*,
    chromium_runs::dsl::chromium_runs,
    gem_pools::dsl::gem_pools,
    libraries::dsl::libraries,
    projects::dsl::projects,
    specimens::{self, table as specimens_table},
    suspension_pools::dsl::suspension_pools,
    suspension_tagging::dsl::suspension_tagging,
    suspensions::{self, table as suspensions_table},
    tenx_assays::{self, table as tenx_assays_table},
    untagged_suspension_pooling::dsl::untagged_suspension_pooling,
};
use diesel::{dsl::AssumeNotNull, prelude::*};
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use jiff_diesel::ToDiesel;

use crate::{
    api::{
        auth::RemoveUnauthorizedProjects,
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter, like_any},
    state::AppState,
};

#[axum::debug_handler]
pub async fn index_chromium_datasets(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<ChromiumDatasetQuery>,
) -> Result<Json<Vec<ChromiumDataset>>, db::Error> {
    select_chromium_datasets(q, &db_conn).await.map(Json)
}

pub async fn select_chromium_datasets(
    ChromiumDatasetQuery {
        filter,
        limit,
        offset,
        order_by,
    }: ChromiumDatasetQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<ChromiumDataset>, db::Error> {
    let mut stmt1 = chromium_datasets_to_unpooled_specimens()
        .select(ChromiumDataset::as_select())
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    let mut stmt2 = chromium_datasets_to_tagged_pooled_specimens()
        .select(ChromiumDataset::as_select())
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    let mut stmt3 = chromium_datasets_to_untagged_pooled_specimens()
        .select(ChromiumDataset::as_select())
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by.as_ref() {
        stmt1 = stmt1.then_order_by(ordering);
        stmt2 = stmt2.then_order_by(ordering);
        stmt3 = stmt3.then_order_by(ordering);
    }

    let (unpooled_datasets, pooled_datasets, untagged_pooled_datasets): (Vec<_>, Vec<_>, Vec<_>) =
        tokio::try_join!(
            stmt1.load(&mut db_conn),
            stmt2.load(&mut db_conn),
            stmt3.load(&mut db_conn)
        )?;

    let mut all_datasets: Vec<_> = unpooled_datasets
        .into_iter()
        .chain(pooled_datasets)
        .chain(untagged_pooled_datasets)
        .collect();

    let sort_fn = |ds1: &ChromiumDataset, ds2: &ChromiumDataset| {
        let mut comparison = Ordering::Equal;

        for ordering in order_by.as_ref() {
            let (mut next_comparison, descending) = match ordering {
                ChromiumDatasetOrderBy::delivered_at { descending } => {
                    (ds1.delivered_at().cmp(&ds2.delivered_at()), descending)
                }
                ChromiumDatasetOrderBy::id { descending } => (ds1.id().cmp(&ds2.id()), descending),
                ChromiumDatasetOrderBy::name { descending } => {
                    (ds1.name().cmp(ds2.name()), descending)
                }
                ChromiumDatasetOrderBy::project_id { descending } => {
                    (ds1.project_id().cmp(&ds2.project_id()), descending)
                }
            };

            if let Some(true) = descending {
                next_comparison = next_comparison.reverse();
            }

            comparison = comparison.then(next_comparison);
        }

        comparison
    };

    all_datasets.sort_by(sort_fn);

    Ok(all_datasets)
}

diesel::alias!(specimens as pooled_tagged_specimens: PooledTaggedSpecimens);
diesel::alias!(suspensions as pooled_tagged_suspensions: PooledTaggedSuspensions);

diesel::alias!(specimens as pooled_untagged_specimens: PooledUntaggedSpecimens);
diesel::alias!(suspensions as pooled_untagged_suspensions: PooledUntaggedSuspensions);

#[must_use]
#[diesel::dsl::auto_type]
pub fn chromium_datasets_to_projects() -> _ {
    chromium_datasets.inner_join(projects)
}

// These 3 functions do not follow DRY because I think pulling out the common
// piece would make the trait bounds would be nightmarish to write
#[must_use]
#[diesel::dsl::auto_type]
pub fn chromium_datasets_to_unpooled_specimens() -> _ {
    chromium_datasets_to_projects()
        .inner_join(
            chromium_dataset_libraries.inner_join(
                libraries.inner_join(
                    cdna.inner_join(
                        gem_pools
                            .inner_join(chromium_runs.inner_join(tenx_assays_table))
                            .inner_join(
                                chip_loadings
                                    .inner_join(suspensions_table.inner_join(specimens_table)),
                            ),
                    ),
                ),
            ),
        )
        .distinct()
}

#[must_use]
#[diesel::dsl::auto_type]
pub fn chromium_datasets_to_tagged_pooled_specimens() -> _ {
    chromium_datasets_to_projects()
        .inner_join(
            chromium_dataset_libraries.inner_join(
                libraries.inner_join(
                    cdna.inner_join(
                        gem_pools
                            .inner_join(chromium_runs.inner_join(tenx_assays_table))
                            .inner_join(
                                chip_loadings.inner_join(
                                    suspension_pools.inner_join(
                                        suspension_tagging.inner_join(
                                            suspensions_table.inner_join(specimens_table),
                                        ),
                                    ),
                                ),
                            ),
                    ),
                ),
            ),
        )
        .distinct()
}

#[must_use]
#[diesel::dsl::auto_type]
pub fn chromium_datasets_to_untagged_pooled_specimens() -> _ {
    chromium_datasets_to_projects()
        .inner_join(
            chromium_dataset_libraries.inner_join(
                libraries.inner_join(
                    cdna.inner_join(
                        gem_pools
                            .inner_join(chromium_runs.inner_join(tenx_assays_table))
                            .inner_join(
                                chip_loadings.inner_join(
                                    suspension_pools.inner_join(
                                        untagged_suspension_pooling.inner_join(
                                            suspensions_table.inner_join(specimens_table),
                                        ),
                                    ),
                                ),
                            ),
                    ),
                ),
            ),
        )
        .distinct()
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for ChromiumDatasetFilter
where
    id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
    AssumeNotNull<specimens::id>: SelectableExpression<QS>,
    AssumeNotNull<specimens::readable_id>: SelectableExpression<QS>,
    AssumeNotNull<specimens::name>: SelectableExpression<QS>,
    AssumeNotNull<specimens::submitted_by>: SelectableExpression<QS>,
    AssumeNotNull<specimens::project_id>: SelectableExpression<QS>,
    AssumeNotNull<specimens::received_at>: SelectableExpression<QS>,
    AssumeNotNull<specimens::species>: SelectableExpression<QS>,
    AssumeNotNull<specimens::host_species>: SelectableExpression<QS>,
    AssumeNotNull<specimens::type_>: SelectableExpression<QS>,
    AssumeNotNull<specimens::tissue>: SelectableExpression<QS>,
    AssumeNotNull<specimens::embedded_in>: SelectableExpression<QS>,
    AssumeNotNull<specimens::fixative>: SelectableExpression<QS>,
    AssumeNotNull<specimens::thermal_preservation_method>: SelectableExpression<QS>,
    AssumeNotNull<specimens::returned_by>: SelectableExpression<QS>,
    AssumeNotNull<specimens::returned_at>: SelectableExpression<QS>,
    AssumeNotNull<specimens::additional_data>: SelectableExpression<QS>,
    tenx_assays::id: SelectableExpression<QS>,
    tenx_assays::name: SelectableExpression<QS>,
    AssumeNotNull<tenx_assays::library_types>: SelectableExpression<QS>,
    AssumeNotNull<tenx_assays::sample_multiplexing>: SelectableExpression<QS>,
    tenx_assays::chemistry_version: SelectableExpression<QS>,
    AssumeNotNull<tenx_assays::chromium_chip>: SelectableExpression<QS>,
    AssumeNotNull<tenx_assays::cmdlines>: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
    delivered_at: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> db::BoxedFilter<'a, QS> {
        let Self {
            ids,
            names,
            specimen,
            assay,
            project_ids,
            delivered_before,
            delivered_after,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(names) = names {
            filter = filter.and_condition(like_any(name, names));
        }

        if let Some(specimen_filter) = specimen {
            filter = filter.and_condition(specimen_filter.to_boxed_filter());
        }

        if let Some(assay_filter) = assay {
            filter = filter.and_condition(assay_filter.to_boxed_filter());
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        if let Some(delivered_before) = delivered_before.map(Timestamp::to_diesel) {
            filter = filter.and_condition(delivered_at.lt(delivered_before));
        }

        if let Some(delivered_after) = delivered_after.map(Timestamp::to_diesel) {
            filter = filter.and_condition(delivered_at.gt(delivered_after));
        }

        filter
    }
}

impl Authorize for ChromiumDatasetQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use cellnoor_models::{
        chromium_dataset::*,
        specimen::{BlockEmbeddingMatrix, Species, SpecimenFilter, SpecimenQuery, SpecimenSummary},
    };
    use jiff::Timestamp;
    use rstest::rstest;

    use super::select_chromium_datasets;
    use crate::{
        api::{
            auth::AuthProjects,
            routes::{
                chromium_datasets::specimens::index::select_chromium_dataset_specimens,
                specimens::index::select_specimens,
            },
        },
        db::DbConnection,
        test_state::{Database, database, root_db_conn},
        test_util::test_query,
    };

    fn sort_by_delivered_at(i1: &&ChromiumDataset, i2: &&ChromiumDataset) -> Ordering {
        i1.delivered_at().cmp(&i2.delivered_at())
    }

    fn sort_by_name(i1: &&ChromiumDataset, i2: &&ChromiumDataset) -> Ordering {
        i1.name().to_lowercase().cmp(&i2.name().to_lowercase())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_chromium_dataset_query(
        #[future] root_db_conn: DbConnection,
        #[future] database: &'static Database,
    ) {
        test_query(select_chromium_datasets)
            .all_records(&database.chromium_datasets)
            .sort_by(|i1, i2| sort_by_delivered_at(i1, i2).reverse())
            .run(&root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_chromium_dataset_query(
        #[future] root_db_conn: DbConnection,
        #[future] database: &'static Database,
    ) {
        let query = ChromiumDatasetQuery::builder()
            .filter(
                ChromiumDatasetFilter::builder()
                    .names(["%s", "%p%"].map(str::to_owned))
                    .build(),
            )
            .limit(i64::MAX)
            .order_by(ChromiumDatasetOrderBy::delivered_at {
                descending: Some(false),
            })
            .order_by(ChromiumDatasetOrderBy::name {
                descending: Some(true),
            })
            .build();

        test_query(select_chromium_datasets)
            .all_records(&database.chromium_datasets)
            .filter(|i| {
                let s = i.name().to_lowercase();
                s.ends_with("s") | s.contains("p")
            })
            .sort_by(|i1, i2| sort_by_delivered_at(i1, i2).then(sort_by_name(i1, i2).reverse()))
            .db_query(query)
            .run(&root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn dataset_filter_respects_specimen_filter(
        #[future] root_db_conn: DbConnection,
        #[future] _database: &'static Database,
    ) {
        let specimen_filter = SpecimenFilter::builder()
            .species([Species::HomoSapiens, Species::MusMusculus])
            .embedded_in([BlockEmbeddingMatrix::CarboxymethylCellulose])
            .received_before(Timestamp::now())
            .build();
        let specimen_query = SpecimenQuery::builder()
            .filter(specimen_filter.clone())
            .limit(i64::MAX)
            .build();
        let dataset_filter = ChromiumDatasetFilter::builder()
            .specimen(specimen_filter)
            .build();
        let dataset_query = ChromiumDatasetQuery::builder()
            .filter(dataset_filter)
            .limit(i64::MAX)
            .build();

        let (specimens, datasets) = tokio::try_join!(
            select_specimens(specimen_query, &root_db_conn),
            select_chromium_datasets(dataset_query, &root_db_conn)
        )
        .unwrap();

        let specimens_from_datasets = datasets.iter().map(ChromiumDataset::id).map(|ds_id| {
            select_chromium_dataset_specimens(&AuthProjects::All, ds_id, &root_db_conn)
        });
        let mut specimens_from_datasets: Vec<_> =
            futures::future::try_join_all(specimens_from_datasets)
                .await
                .unwrap();

        for specimen_set in &mut specimens_from_datasets {
            specimen_set.sort_by_key(SpecimenSummary::id);
            let pre_deduplication_length = specimen_set.len();
            specimen_set.dedup();
            assert_eq!(
                pre_deduplication_length,
                specimen_set.len(),
                "query returned duplicate specimens"
            );
        }

        let specimens_from_datasets: Vec<_> =
            specimens_from_datasets.into_iter().flatten().collect();
        for specimen in &specimens {
            assert!(
                specimens_from_datasets.contains(specimen),
                "chromium dataset query did not respect specimen filter"
            );
        }
    }
}
