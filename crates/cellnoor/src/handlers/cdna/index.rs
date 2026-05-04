use axum::{Json, extract::State};
use cellnoor_models::cdna::{CdnaFilter, CdnaQuery, CdnaSummary};
use cellnoor_schema::cdna::{
    self, gem_pool_id, id, library_type, n_amplification_cycles, prepared_at, project_id,
    readable_id,
};
use diesel::{SelectableExpression, dsl::AssumeNotNull, prelude::*};
use diesel_async::RunQueryDsl;
use jiff_diesel::ToDiesel;

use crate::{
    api::{
        auth::RemoveUnauthorizedProjects,
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub(super) async fn index_cdna(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<CdnaQuery>,
) -> Result<Json<Vec<CdnaSummary>>, db::Error> {
    select_cdna(q, &db_conn).await.map(Json)
}

pub async fn select_cdna(
    CdnaQuery {
        filter,
        limit,
        offset,
        order_by,
    }: CdnaQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<CdnaSummary>, db::Error> {
    let mut stmt = CdnaSummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for CdnaFilter
where
    id: SelectableExpression<QS>,
    readable_id: SelectableExpression<QS>,
    AssumeNotNull<gem_pool_id>: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
    library_type: SelectableExpression<QS>,
    prepared_at: SelectableExpression<QS>,
    n_amplification_cycles: SelectableExpression<QS>,
    AssumeNotNull<cdna::additional_data>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            ids,
            readable_ids,
            gem_pool_ids,
            project_ids,
            library_types,
            prepared_before,
            prepared_after,
            n_amplification_cycles_less_than,
            n_amplification_cycles_more_than,
            additional_data,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(readable_ids) = readable_ids {
            filter = filter.and_condition(readable_id.eq_any(readable_ids));
        }

        if let Some(gem_pool_ids) = gem_pool_ids {
            filter = filter.and_condition(gem_pool_id.assume_not_null().eq_any(gem_pool_ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        if let Some(library_types) = library_types {
            filter = filter.and_condition(library_type.eq_any(library_types));
        }

        if let Some(prepared_before) = prepared_before.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(prepared_at.lt(prepared_before));
        }

        if let Some(prepared_after) = prepared_after.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(prepared_at.gt(prepared_after));
        }

        if let Some(n_amplification_cycles_less_than) = n_amplification_cycles_less_than {
            filter =
                filter.and_condition(n_amplification_cycles.lt(*n_amplification_cycles_less_than));
        }

        if let Some(n_amplification_cycles_more_than) = n_amplification_cycles_more_than {
            filter =
                filter.and_condition(n_amplification_cycles.gt(*n_amplification_cycles_more_than));
        }

        if let Some(additional_data) = additional_data {
            filter = filter.and_condition(
                cdna::additional_data
                    .assume_not_null()
                    .contains(additional_data),
            );
        }

        filter
    }
}

impl Authorize for CdnaQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
