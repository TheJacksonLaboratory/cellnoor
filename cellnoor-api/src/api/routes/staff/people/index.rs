use axum::{Json, extract::State};
use cellnoor_models::person::{PersonFilterStaff, PersonQueryStaff, PersonSummaryStaff};
use cellnoor_schema::people::{self, email, id, institution_id, microsoft_entra_oid, name, orcid};
use diesel::{dsl::AssumeNotNull, prelude::*};
use diesel_async::RunQueryDsl;

use crate::{
    api::{
        auth,
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub(super) async fn index_people_staff_view(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<PersonQueryStaff>,
) -> Result<Json<Vec<PersonSummaryStaff>>, db::Error> {
    select_people_staff_view(q, &db_conn).await.map(Json)
}

pub async fn select_people_staff_view(
    PersonQueryStaff {
        filter,
        limit,
        offset,
        order_by,
    }: PersonQueryStaff,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<PersonSummaryStaff>, db::Error> {
    let mut stmt = PersonSummaryStaff::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for PersonFilterStaff
where
    id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
    institution_id: SelectableExpression<QS>,
    AssumeNotNull<email>: SelectableExpression<QS>,
    AssumeNotNull<orcid>: SelectableExpression<QS>,
    AssumeNotNull<microsoft_entra_oid>: SelectableExpression<QS>,
    people::is_admin: SelectableExpression<QS>,
    people::is_biology_staff: SelectableExpression<QS>,
    people::is_computational_staff: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            inner,
            microsoft_entra_oids,
            is_admin,
            is_biology_staff,
            is_computational_staff,
        } = self;

        let mut filter = BoxedFilter::new_true();

        if let Some(inner) = inner {
            filter = filter.and_condition(inner.to_boxed_filter());
        }

        if let Some(microsoft_entra_oids) = microsoft_entra_oids {
            filter = filter.and_condition(
                microsoft_entra_oid
                    .assume_not_null()
                    .eq_any(microsoft_entra_oids),
            );
        }

        if let Some(is_admin) = *is_admin {
            filter = filter.and_condition(people::is_admin.eq(is_admin));
        }

        if let Some(is_biology_staff) = *is_biology_staff {
            filter = filter.and_condition(people::is_biology_staff.eq(is_biology_staff));
        }

        if let Some(is_computational_staff) = *is_computational_staff {
            filter =
                filter.and_condition(people::is_computational_staff.eq(is_computational_staff));
        }

        filter
    }
}

impl Authorize for PersonQueryStaff {
    fn authorize(self, user: &crate::api::auth::AuthUser) -> Result<Self, auth::Error> {
        // This is technically redundant because we've applied middleware to the whole
        // router, but it doesn't hurt and may prevent future errors
        if !user.is_staff() {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(self)
    }
}
