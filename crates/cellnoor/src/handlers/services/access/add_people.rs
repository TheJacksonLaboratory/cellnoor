use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::Relation;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
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
    state
        .in_transaction(user, async |tx| {
            insert_service_accesses(tx, service_id, &people).await
        })
        .await
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

    tx.insert_many(&accesses).await
}

struct NewServiceAccess {
    service_id: Uuid,
    person_id: Uuid,
}

impl Relation for NewServiceAccess {
    const NAME: &'static str = "service_access";
}

impl Insert for NewServiceAccess {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            service_id,
            person_id,
        } = self;

        vec![("service_id", service_id), ("person_id", person_id)]
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

        insert_service_accesses(&tx, service.id, &[person.record.id])
            .await
            .unwrap();
    }
}
