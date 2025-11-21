use axum::{extract::State, http::StatusCode};
use diesel::{dsl::AssumeNotNull, prelude::*, sql_types::Text};
use scamplers_models::person::{
    self, OrdinalColumns, Person, PersonId, PersonSummary, PersonSummaryWithParents,
};
use scamplers_schema::people;
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter, utils::like_any},
    query,
    state::AppState,
};

pub(super) async fn list_people(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<person::Query>,
) -> ApiResponse<Vec<PersonSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

diesel::define_sql_function! {fn get_user_roles(user_id: Text) -> Array<Text>}

impl db::Operation<Person> for PersonId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Person, db::Error> {
        let info = PersonSummaryWithParents::query()
            .filter(people::id.eq(&self))
            .first(db_conn)?;

        let roles = diesel::select(get_user_roles(self.to_id_string())).get_result(db_conn)?;

        Ok(Person::new(info, roles))
    }
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for person::Filter
where
    people::id: SelectableExpression<QS>,
    people::name: SelectableExpression<QS>,
    AssumeNotNull<people::email>: SelectableExpression<QS>,
    AssumeNotNull<people::orcid>: SelectableExpression<QS>,
    AssumeNotNull<people::microsoft_entra_oid>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let mut filter = BoxedFilter::new();

        if let Some(ids) = self.ids() {
            filter = filter.and_condition(people::id.eq_any(ids));
        }

        if let Some(names) = self.names() {
            filter = filter.and_condition(like_any(people::name, names));
        }

        if let Some(emails) = self.emails() {
            filter = filter.and_condition(like_any(people::email.assume_not_null(), emails));
        }

        if let Some(orcids) = self.orcids() {
            filter = filter.and_condition(like_any(people::orcid.assume_not_null(), orcids));
        }

        if let Some(microsoft_entra_oids) = self.microsoft_entra_oids() {
            filter = filter.and_condition(
                people::microsoft_entra_oid
                    .assume_not_null()
                    .eq_any(microsoft_entra_oids),
            );
        }

        filter
    }
}

impl db::Operation<Vec<PersonSummary>> for person::Query {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Vec<PersonSummary>, db::Error> {
        let stmt = query!(PersonSummary::query(self).order_by(
            OrdinalColumns::Id = people::id,
            OrdinalColumns::Name = people::name,
            OrdinalColumns::Email = people::email
        ));

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
        test_query::<Query, _>()
            .all_data(&database.people)
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
        let query = Query::builder()
            .filter(
                Filter::builder()
                    .names(["%5%", "%h%"].map(str::to_owned))
                    .build(),
            )
            .order_by_descending(OrdinalColumns::Name)
            .build();

        test_query()
            .all_data(&database.people)
            .filter(|i| {
                let s = i.name().to_lowercase();
                s.contains("5") | s.contains("h")
            })
            .sort_by(|i1, i2| sort_by_name(i1, i2).reverse())
            .db_query(query)
            .run(root_db_conn)
            .await;
    }
}
