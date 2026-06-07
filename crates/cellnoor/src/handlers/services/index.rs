use axum::{Json, extract::State};
use cellnoor_types::{
    operator::UuidOperator,
    service::{Service, ServicePredicate, ServiceQuery},
};
use futures::StreamExt;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder, select_one},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_services(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ServiceQuery>,
) -> Result<Json<Vec<Service>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_services(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn select_services(
    tx: &db::Transaction<'_>,
    query: &mut ServiceQuery,
) -> Result<Vec<Service>, ErrorInner> {
    static SELECT_SERVICES: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    let sql = SELECT_SERVICES.finish_with_query(query);

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.collect().await)
}

// Even though there's no service-accounts/{id} route, this function is still
// helpful
pub(super) async fn select_service_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Service, ErrorInner> {
    select_one(
        tx,
        ServicePredicate::Id(UuidOperator::Eq(id)),
        select_services,
    )
    .await
}

impl AsPredicate for ServicePredicate {
    fn as_predicate(
        &self,
    ) -> (
        &'static str,
        (&'static str, &(dyn postgres_types::ToSql + Sync)),
    ) {
        let sql = match self {
            Self::Id(u) | Self::OwnedBy(u) => u.as_sql_operator_and_value(),
            Self::Description(s) => s.as_sql_operator_and_value(),
            Self::CanReadAllProjects(b) | Self::CanAdminUsers(b) => b.as_sql_operator_and_value(),
            Self::CreatedAt(t) => t.as_sql_operator_and_value(),
        };

        (self.field_name(), sql)
    }
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

        let mut query =
            ServiceQuery::from_filter(ServicePredicate::Id(UuidOperator::Eq(inserted.id)));
        let selected = select_services(&tx, &mut query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, inserted.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<ServiceField>(&tx, "service").await;
    }
}
