use axum::{Json, extract::State};
use cellnoor_types::{
    operator::UuidOperator,
    service_account::{ServiceAccount, ServiceAccountPredicate, ServiceAccountQuery},
};
use futures::StreamExt;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder, select_one},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_service_accounts(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ServiceAccountQuery>,
) -> Result<Json<Vec<ServiceAccount>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_service_accounts(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn select_service_accounts(
    tx: &db::Transaction<'_>,
    query: &mut ServiceAccountQuery,
) -> Result<Vec<ServiceAccount>, ErrorInner> {
    static SELECT_SERVICE_ACCOUNTS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    let sql = SELECT_SERVICE_ACCOUNTS.finish_with_query(query);

    let stream = tx.query_stream_into::<ServiceAccount>(sql).await?;
    Ok(stream.collect().await)
}

// Internal helper used to build the response after an insert/update. There is deliberately no
// public `show` handler for service accounts.
pub(super) async fn select_service_account_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<ServiceAccount, ErrorInner> {
    select_one(
        tx,
        ServiceAccountPredicate::Id(UuidOperator::Eq(id)),
        select_service_accounts,
    )
    .await
}

impl AsPredicate for ServiceAccountPredicate {
    fn as_predicate(
        &self,
    ) -> (
        &'static str,
        (&'static str, &(dyn postgres_types::ToSql + Sync)),
    ) {
        let sql = match self {
            Self::Id(u) | Self::OwnedBy(u) => u.as_sql_operator_and_value(),
            Self::Description(s) => s.as_sql_operator_and_value(),
            Self::CreatedAt(t) => t.as_sql_operator_and_value(),
        };

        (self.field_name(), sql)
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        service_account::{ServiceAccountPredicate, ServiceAccountQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::service_accounts::{
            create::test::insert_test_service_account, index::select_service_accounts,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_service_account(&tx, |_| ()).await.unwrap();

        let mut query = ServiceAccountQuery::from_filter(ServiceAccountPredicate::Id(
            UuidOperator::Eq(inserted.id),
        ));
        let selected = select_service_accounts(&tx, &mut query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, inserted.id);
    }
}
