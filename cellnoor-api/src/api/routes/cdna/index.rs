use axum::{Json, extract::State, http::status::StatusCode};
use cellnoor_models::cdna::{CdnaFilter, CdnaQuery, CdnaSummary};
use cellnoor_schema::cdna::{dsl::id, project_id};
use diesel::{SelectableExpression, prelude::*};
use diesel_async::RunQueryDsl;

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
    mut db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<CdnaQuery>,
) -> Result<Json<Vec<CdnaSummary>>, db::Error> {
    select_cdna(q, &mut db_conn).await.map(Json)
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
    project_id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self { ids, project_ids } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
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
