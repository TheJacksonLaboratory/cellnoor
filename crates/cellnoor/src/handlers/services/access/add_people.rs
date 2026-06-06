use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn add_people_to_service(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: service_id }): Path<IdParam>,
    Json(people): Json<Vec<Uuid>>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = insert_service_accesses(&tx, service_id, &people)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super::super) async fn insert_service_accesses(
    tx: &db::Transaction<'_>,
    service_id: Uuid,
    people: &[Uuid],
) -> Result<(), ErrorInner> {
    let accesses: Vec<_> = people
        .iter()
        .map(|&person_id| NewServiceAccess {
            service_id,
            person_id,
        })
        .collect();

    futures::future::try_join_all(
        accesses
            .iter()
            .map(|a| db::insert_into_no_returning(tx, "service_access", a)),
    )
    .await?;

    Ok(())
}

struct NewServiceAccess {
    service_id: Uuid,
    person_id: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewServiceAccess {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            service_id,
            person_id,
        } = self;

        [("service_id", service_id), ("person_id", person_id)]
    }
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            people::create::test::insert_test_person_and_institution,
            services::{
                access::add_people::insert_service_accesses, create::test::insert_test_service,
            },
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn add_people() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, service) = insert_test_service(&tx, |_| ()).await.unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        insert_service_accesses(&tx, *service.id, &[*person.record.id])
            .await
            .unwrap();
    }
}
