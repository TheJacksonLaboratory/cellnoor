use axum::{Json, extract::State};
use cellnoor_types::service::{ServiceDetailed, ServiceQuery};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_services_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<ServiceQuery>,
) -> Result<Json<Vec<ServiceDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| select_services_detailed(tx, &query).await)
        .await
}

pub(in super::super) async fn select_services_detailed(
    tx: &db::Transaction<'_>,
    query: &ServiceQuery,
) -> Result<Vec<ServiceDetailed>, DbError> {
    static SELECT_SERVICES_DETAILED: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    tx.select(&SELECT_SERVICES_DETAILED, query).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        permission::{Action, Permission, Resource},
        service::{ServiceField, ServicePredicate, ServiceQuery},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::services::{
            create::test::insert_test_service, index_detailed::select_services_detailed,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let permission = Permission {
            resource: Resource::Specimen,
            action: Action::Create,
        };
        let (_, inserted) = insert_test_service(&tx, |s| {
            s.users = vec![Uuid::nil()];
            s.permissions_to_grant = vec![permission];
        })
        .await
        .unwrap();

        let query = ServiceQuery::from_filter(ServicePredicate::Id(UuidOperator::Eq(inserted.id)));
        let selected = select_services_detailed(&tx, &query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].service, inserted);
        assert_eq!(selected[0].permissions, [permission]);
        assert_eq!(selected[0].members, [Uuid::nil()]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<ServiceField>(&tx, "service_detailed").await;
    }
}
