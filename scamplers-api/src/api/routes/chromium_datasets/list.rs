use axum::{extract::State, http::StatusCode};
use diesel::{dsl::AssumeNotNull, prelude::*};
use jiff::Timestamp;
use jiff_diesel::ToDiesel;
use scamplers_models::chromium_dataset::{
    ChromiumDatasetFilter, ChromiumDatasetQuery, ChromiumDatasetSummary,
};
use scamplers_schema::{chromium_datasets::dsl::*, specimens, tenx_assays};
use serde_qs::axum::QsQuery;
use uuid::Uuid;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{
            ApiResponse, Root,
            chromium_datasets::common::{
                chromium_datasets_to_pooled_specimens, chromium_datasets_to_specimens,
            },
            inner_handler,
        },
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter},
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

        let dataset_ids_derived_from_suspensions: Vec<Uuid> = chromium_datasets_to_specimens()
            .select(id)
            .filter(filter.to_boxed_filter())
            .load(db_conn)?;

        let dataset_ids_derived_from_suspension_pools: Vec<Uuid> =
            chromium_datasets_to_pooled_specimens()
                .select(id)
                .filter(filter.to_boxed_filter())
                .load(db_conn)?;

        let all: Vec<_> = dataset_ids_derived_from_suspensions
            .into_iter()
            .chain(dataset_ids_derived_from_suspension_pools)
            .collect();

        let mut all = chromium_datasets
            .select(ChromiumDatasetSummary::as_select())
            .filter(id.eq_any(&all))
            .limit(limit)
            .offset(offset)
            .into_boxed();
        for ordering in order_by.as_ref() {
            all = all.order_by(ordering);
        }

        Ok(all.load(db_conn)?)
    }
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for ChromiumDatasetFilter
where
    id: SelectableExpression<QS>,
    specimens::id: SelectableExpression<QS>,
    specimens::name: SelectableExpression<QS>,
    specimens::submitted_by: SelectableExpression<QS>,
    specimens::lab_id: SelectableExpression<QS>,
    specimens::received_at: SelectableExpression<QS>,
    specimens::species: SelectableExpression<QS>,
    AssumeNotNull<specimens::host_species>: SelectableExpression<QS>,
    specimens::type_: SelectableExpression<QS>,
    AssumeNotNull<specimens::tissue>: SelectableExpression<QS>,
    AssumeNotNull<specimens::embedded_in>: SelectableExpression<QS>,
    AssumeNotNull<specimens::fixative>: SelectableExpression<QS>,
    specimens::frozen: SelectableExpression<QS>,
    specimens::cryopreserved: SelectableExpression<QS>,
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
