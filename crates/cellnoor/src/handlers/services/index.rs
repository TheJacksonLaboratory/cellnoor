use axum::{Json, extract::State};
use cellnoor_types::{
    operator::UuidOperator,
    service::{Service, ServicePredicate, ServiceQuery},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_services(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<ServiceQuery>,
) -> Result<Json<Vec<Service>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_services(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn select_services(
    tx: &db::Transaction<'_>,
    query: &ServiceQuery,
) -> Result<Vec<Service>, ErrorInner> {
    static SELECT_SERVICES: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    tx.select(&SELECT_SERVICES, query).await
}

// Even though there's no service-accounts/{id} route, this function is still
// helpful
pub(super) async fn select_service_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Service, ErrorInner> {
    tx.select_one(ServicePredicate::Id(UuidOperator::Eq(id)), select_services)
        .await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        service::{ServiceField, ServicePredicate, ServiceQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::services::{create::test::insert_test_service, index::select_services},
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_service(&tx, |_| ()).await.unwrap();

        let query = ServiceQuery::from_filter(ServicePredicate::Id(UuidOperator::Eq(inserted.id)));
        let selected = select_services(&tx, &query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, inserted.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<ServiceField>(&tx, "service_public").await;
    }
}
