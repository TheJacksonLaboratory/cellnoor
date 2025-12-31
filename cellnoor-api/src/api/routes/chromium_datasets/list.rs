use axum::{extract::State, http::StatusCode};
use cellnoor_models::chromium_dataset::{
    ChromiumDatasetFilter, ChromiumDatasetQuery, ChromiumDatasetSummary,
};
use cellnoor_schema::{
    cdna::dsl::cdna,
    chip_loadings::dsl::chip_loadings,
    chromium_dataset_libraries::dsl::chromium_dataset_libraries,
    chromium_datasets::dsl::*,
    chromium_runs::dsl::chromium_runs,
    gem_pools::dsl::gem_pools,
    libraries::dsl::libraries,
    specimens::{self, table as specimens_table},
    suspension_pools::dsl::suspension_pools,
    suspension_tagging::dsl::suspension_tagging,
    suspensions::{self, table as suspensions_table},
    tenx_assays::{self, table as tenx_assays_table},
};
use diesel::{dsl::AssumeNotNull, prelude::*};
use jiff::Timestamp;
use jiff_diesel::ToDiesel;

use crate::{
    api::{
        extract::{auth::AuthenticatedUser, query::QsQuery},
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter, utils::like_any},
    state::AppState,
};

pub async fn list_chromium_datasets(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(query): QsQuery<ChromiumDatasetQuery>,
) -> ApiResponse<Vec<ChromiumDatasetSummary>> {
    Ok((StatusCode::OK, inner_handler(state, user, query).await?))
}

impl db::Operation<Vec<ChromiumDatasetSummary>> for ChromiumDatasetQuery {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<ChromiumDatasetSummary>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = chromium_datasets_to_all_specimens()
            .select(ChromiumDatasetSummary::as_select())
            .limit(limit)
            .offset(offset)
            .filter(filter.to_boxed_filter())
            .into_boxed();

        for ordering in order_by.as_ref() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(db_conn)?)
    }
}

diesel::alias!(specimens as pooled_specimens: PooledSpecimens);
diesel::alias!(suspensions as pooled_suspensions: PooledSuspensions);

#[diesel::dsl::auto_type]
pub(crate) fn chromium_datasets_to_all_specimens() -> _ {
    chromium_datasets
        .inner_join(
            chromium_dataset_libraries.inner_join(
                libraries.inner_join(
                    cdna.inner_join(
                        gem_pools
                            .inner_join(
                                chip_loadings
                                    .left_join(suspensions_table.inner_join(specimens_table))
                                    .left_join(suspension_pools.inner_join(
                                        suspension_tagging.inner_join(
                                            pooled_suspensions.inner_join(pooled_specimens),
                                        ),
                                    )),
                            )
                            .inner_join(chromium_runs.inner_join(tenx_assays_table)),
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
    AssumeNotNull<specimens::name>: SelectableExpression<QS>,
    AssumeNotNull<specimens::submitted_by>: SelectableExpression<QS>,
    AssumeNotNull<specimens::lab_id>: SelectableExpression<QS>,
    AssumeNotNull<specimens::received_at>: SelectableExpression<QS>,
    AssumeNotNull<specimens::species>: SelectableExpression<QS>,
    AssumeNotNull<specimens::host_species>: SelectableExpression<QS>,
    AssumeNotNull<specimens::type_>: SelectableExpression<QS>,
    AssumeNotNull<specimens::tissue>: SelectableExpression<QS>,
    AssumeNotNull<specimens::embedded_in>: SelectableExpression<QS>,
    AssumeNotNull<specimens::fixative>: SelectableExpression<QS>,
    AssumeNotNull<specimens::frozen>: SelectableExpression<QS>,
    AssumeNotNull<specimens::cryopreserved>: SelectableExpression<QS>,
    AssumeNotNull<specimens::returned_by>: SelectableExpression<QS>,
    AssumeNotNull<specimens::returned_at>: SelectableExpression<QS>,
    AssumeNotNull<specimens::additional_data>: SelectableExpression<QS>,
    tenx_assays::id: SelectableExpression<QS>,
    tenx_assays::name: SelectableExpression<QS>,
    AssumeNotNull<tenx_assays::library_types>: SelectableExpression<QS>,
    AssumeNotNull<tenx_assays::sample_multiplexing>: SelectableExpression<QS>,
    tenx_assays::chemistry_version: SelectableExpression<QS>,
    AssumeNotNull<tenx_assays::chromium_chip>: SelectableExpression<QS>,
    lab_id: SelectableExpression<QS>,
    delivered_at: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> db::BoxedFilter<'a, QS> {
        let Self {
            ids,
            names,
            specimen,
            assay,
            lab_ids,
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

        if let Some(lab_ids) = lab_ids {
            filter = filter.and_condition(lab_id.eq_any(lab_ids));
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
