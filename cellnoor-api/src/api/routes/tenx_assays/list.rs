use axum::{extract::State, http::StatusCode};
use cellnoor_models::tenx_assay::{TenxAssay, TenxAssayFilter, TenxAssayQuery};
use cellnoor_schema::tenx_assays::dsl::*;
use diesel::{dsl::AssumeNotNull, prelude::*};
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter, utils::like_any},
    state::AppState,
};

pub async fn list_tenx_assays(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<TenxAssayQuery>,
) -> ApiResponse<Vec<TenxAssay>> {
    Ok((StatusCode::OK, inner_handler(state, user, request).await?))
}

impl db::Operation<Vec<TenxAssay>> for TenxAssayQuery {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Vec<TenxAssay>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = TenxAssay::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for TenxAssayFilter
where
    id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
    AssumeNotNull<library_types>: SelectableExpression<QS>,
    AssumeNotNull<sample_multiplexing>: SelectableExpression<QS>,
    chemistry_version: SelectableExpression<QS>,
    AssumeNotNull<chromium_chip>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            ids,
            names,
            library_types: library_types_query,
            sample_multiplexing: sample_multiplexing_query,
            chemistry_versions,
            chromium_chips,
        } = self;

        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(names) = names {
            filter = filter.and_condition(like_any(name, names));
        }

        if let Some(library_types_query) = library_types_query {
            let mut condition = BoxedFilter::new_false();
            let lib_type_field = library_types.assume_not_null();
            for query in library_types_query {
                condition = condition.or_condition(
                    lib_type_field
                        .contains(query)
                        .and(lib_type_field.is_contained_by(query)),
                );
            }

            filter = filter.and_condition(condition);
        }

        if let Some(sample_multiplexing_query) = sample_multiplexing_query {
            filter = filter.and_condition(
                sample_multiplexing
                    .assume_not_null()
                    .eq_any(sample_multiplexing_query),
            );
        }

        if let Some(chemistry_versions) = chemistry_versions {
            filter = filter.and_condition(like_any(chemistry_version, chemistry_versions));
        }

        if let Some(chromium_chips) = chromium_chips {
            filter =
                filter.and_condition(like_any(chromium_chip.assume_not_null(), chromium_chips));
        }

        filter
    }
}
