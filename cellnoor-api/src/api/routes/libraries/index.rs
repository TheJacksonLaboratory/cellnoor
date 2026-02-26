use axum::{Json, extract::State};
use cellnoor_models::library::{LibraryFilter, LibraryQuery, LibrarySummary};
use cellnoor_schema::libraries::{id, project_id, readable_id};
use diesel::{SelectableExpression, prelude::*};
use diesel_async::RunQueryDsl;

use crate::{
    api::{
        auth::RemoveUnauthorizedProjects,
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter, like_any},
    state::AppState,
};

pub(super) async fn index_libraries(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<LibraryQuery>,
) -> Result<Json<Vec<LibrarySummary>>, db::Error> {
    select_libraries(q, &db_conn).await.map(Json)
}

pub async fn select_libraries(
    LibraryQuery {
        filter,
        limit,
        offset,
        order_by,
    }: LibraryQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<LibrarySummary>, db::Error> {
    let mut stmt = LibrarySummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for LibraryFilter
where
    id: SelectableExpression<QS>,
    readable_id: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            ids,
            readable_ids,
            project_ids,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(readable_ids) = readable_ids {
            filter = filter.and_condition(like_any(readable_id, readable_ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        filter
    }
}

impl Authorize for LibraryQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
