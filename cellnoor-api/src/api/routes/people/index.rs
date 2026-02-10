use axum::{Json, extract::State};
use cellnoor_models::person::{PersonFilter, PersonQuery, PersonSummary};
use cellnoor_schema::people::dsl::{email, id, institution_id, microsoft_entra_oid, name, orcid};
use diesel::{dsl::AssumeNotNull, prelude::*};
use diesel_async::RunQueryDsl;

use crate::{
    api::{
        auth::{self, AuthUser},
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter, like_any},
    state::AppState,
};

pub async fn index_people(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q: query }: AuthJsonQuery<PersonQuery>,
) -> Result<Json<Vec<PersonSummary>>, db::Error> {
    select_people(query, &db_conn).await.map(Json)
}

pub async fn select_people(
    PersonQuery {
        filter,
        limit,
        offset,
        order_by,
    }: PersonQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<PersonSummary>, db::Error> {
    let mut stmt = PersonSummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
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
        let Self {
            ids,
            names,
            emails,
            institution_ids,
            orcids,
            microsoft_entra_oids,
        } = self;

        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(names) = names {
            filter = filter.and_condition(like_any(name, names));
        }

        if let Some(emails) = emails {
            filter = filter.and_condition(like_any(email.assume_not_null(), emails));
        }

        if let Some(institution_ids) = institution_ids {
            filter = filter.and_condition(institution_id.eq_any(institution_ids));
        }

        if let Some(orcids) = orcids {
            filter = filter.and_condition(like_any(orcid.assume_not_null(), orcids));
        }

        if let Some(microsoft_entra_oids) = microsoft_entra_oids {
            filter = filter.and_condition(
                microsoft_entra_oid
                    .assume_not_null()
                    .eq_any(microsoft_entra_oids),
            );
        }

        filter
    }
}

impl Authorize for PersonQuery {
    fn authorize(self, _user: &AuthUser) -> Result<Self, auth::Error> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use cellnoor_models::person::*;
    use rstest::rstest;

    use super::select_people;
    use crate::{
        db::DbConnection,
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
    #[tokio::test(flavor = "multi_thread")]
    async fn default_person_query(
        #[future] root_db_conn: DbConnection,
        #[future] database: &'static Database,
    ) {
        test_query(select_people)
            .all_records(&database.people)
            .sort_by(sort_by_name)
            .run(root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test(flavor = "multi_thread")]
    async fn specific_person_query(
        #[future] root_db_conn: DbConnection,
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

        test_query(select_people)
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
