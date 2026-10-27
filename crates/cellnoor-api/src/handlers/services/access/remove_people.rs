use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::IdParam,
    state::AppState,
};

pub async fn remove_people_from_service(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: service_id }): Path<IdParam>,
    crate::extract::JsonExtractor(people): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            delete_service_accesses(tx, service_id, &people).await
        })
        .await
}

pub(in super::super::super) async fn delete_service_accesses(
    tx: &db::Transaction<'_>,
    service_id: Uuid,
    people: &[Uuid],
) -> Result<(), DbError> {
    static DELETE_SERVICE_ACCESSES: &str =
        "delete from service_access where service_id = $1 and person_id = any($2)";

    let n_rows = tx
        .execute(&Sql::new(
            DELETE_SERVICE_ACCESSES,
            vec![&service_id, &people],
        ))
        .await?;

    if n_rows == 0 {
        return Err(DbError::ResourceNotFound);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use cellnoor_types::{operator::UuidOperator, service::ServicePredicate};
    use uuid::Uuid;

    use crate::{
        handlers::services::{
            access::remove_people::delete_service_accesses, create::test::insert_test_service,
            index_detailed::select_services_detailed,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_people() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, service) = insert_test_service(&tx, |s| s.users = vec![Uuid::nil()])
            .await
            .unwrap();

        delete_service_accesses(&tx, service.id, &[Uuid::nil()])
            .await
            .unwrap();

        let selected = tx
            .select_one(
                ServicePredicate::Id(UuidOperator::Eq(service.id)),
                select_services_detailed,
            )
            .await
            .unwrap();

        assert!(selected.members.is_empty());
    }
}
