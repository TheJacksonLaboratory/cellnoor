use axum::{extract::State, http::StatusCode};
use diesel::{dsl::AssumeNotNull, prelude::*};
use scamplers_models::person::{PersonFilter, PersonQuery, PersonSummary};
use scamplers_schema::people::dsl::{email, id, institution_id, microsoft_entra_oid, name, orcid};
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter, utils::like_any},
    state::AppState,
};

pub(super) async fn list_people(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<PersonQuery>,
) -> ApiResponse<Vec<PersonSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for PersonFilter
where
    id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
    institution_id: SelectableExpression<QS>,
    AssumeNotNull<email>: SelectableExpression<QS>,
    AssumeNotNull<orcid>: SelectableExpression<QS>,
    AssumeNotNull<microsoft_entra_oid>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let mut filter = BoxedFilter::new();

        if let Some(ids) = self.ids() {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(names) = self.names() {
            filter = filter.and_condition(like_any(name, names));
        }

        if let Some(emails) = self.emails() {
            filter = filter.and_condition(like_any(email.assume_not_null(), emails));
        }

        if let Some(institution_ids) = self.institution_ids() {
            filter = filter.and_condition(institution_id.eq_any(institution_ids));
        }

        if let Some(orcids) = self.orcids() {
            filter = filter.and_condition(like_any(orcid.assume_not_null(), orcids));
        }

        if let Some(microsoft_entra_oids) = self.microsoft_entra_oids() {
            filter = filter.and_condition(
                microsoft_entra_oid
                    .assume_not_null()
                    .eq_any(microsoft_entra_oids),
            );
        }

        filter
    }
}

impl db::Operation<Vec<PersonSummary>> for PersonQuery {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Vec<PersonSummary>, db::Error> {
        let filter = self.filter();

        let mut stmt = PersonSummary::query()
            .limit(self.limit())
            .offset(self.offset())
            .filter(filter.to_boxed_filter())
            .into_boxed();

        for ordering in self.order_by() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(db_conn)?)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;
    use scamplers_models::person::*;

    use crate::{
        test_state::{Database, database, root_db_conn},
        test_util::test_query,
    };

    fn sort_by_id(i1: &&PersonSummary, i2: &&PersonSummary) -> Ordering {
        i1.id().cmp(&i2.id())
    }

    fn sort_by_name(i1: &&PersonSummary, i2: &&PersonSummary) -> Ordering {
        i1.name().to_lowercase().cmp(&i2.name().to_lowercase())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_person_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        test_query::<PersonQuery, _>()
            .all_records(&database.people)
            .sort_by(sort_by_name)
            .run(root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_person_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        let query = PersonQuery::builder()
            .filter(
                PersonFilter::builder()
                    .names(["%5%", "%h%"].map(str::to_owned))
                    .build(),
            )
            .limit(i64::MAX)
            .order_by(PersonOrderBy::id {
                descending: Some(false),
            })
            .order_by(PersonOrderBy::name {
                descending: Some(true),
            })
            .build();

        test_query()
            .all_records(&database.people)
            .filter(|i| {
                let s = i.name().to_lowercase();
                s.contains("5") | s.contains("h")
            })
            .sort_by(|i1, i2| sort_by_id(i1, i2).then(sort_by_name(i1, i2).reverse()))
            .db_query(query)
            .run(root_db_conn)
            .await;
    }
}
