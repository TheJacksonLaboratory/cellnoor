use axum::{Json, extract::State};
use cellnoor_types::service::{NewService, Service, ServiceField, ServiceSimpleFields};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::{
        permissions::grant_permissions,
        services::{access::add_people::insert_service_accesses, index::select_service_by_id},
        set_is_staff,
    },
    state::AppState,
};

pub async fn create_service(
    State(state): State<AppState>,
    user: AuthUser,
    Json(service): Json<NewService>,
) -> Result<Json<Service>, DbError> {
    state
        .in_transaction(user, async |tx| insert_service(tx, &service).await)
        .await
}

async fn insert_service(
    tx: &db::Transaction<'_>,
    NewService {
        record,
        users,
        permissions_to_grant,
    }: &NewService,
) -> Result<Service, DbError> {
    let id = tx.insert_returning_id(record).await?;
    set_is_staff(tx, id, record.is_staff).await?;
    grant_permissions(tx, id, permissions_to_grant).await?;

    insert_service_accesses(tx, id, users).await?;

    select_service_by_id(tx, id).await
}

// `owned_by` is intentionally omitted: the database fills it from
// `app_user_id()`, and row-level security guarantees it equals the caller.
// `is_staff` is a column of `principal`, not `service` (see `set_is_staff`)
impl Insert for ServiceSimpleFields {
    type Field = ServiceField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            description,
            is_staff: _,
        } = self;

        vec![(ServiceField::Description, description)]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::service::{NewService, Service, ServiceSimpleFields};
    use uuid::Uuid;

    use crate::{
        db::{self, DbError},
        handlers::services::create::insert_service,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_service<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewService, Service), DbError>
    where
        F: FnMut(&mut NewService),
    {
        let mut new = NewService {
            record: ServiceSimpleFields {
                description: Some(Uuid::new_v4().to_string().to_nonempty_string()),
                is_staff: false,
            },
            users: vec![Uuid::nil()],
            permissions_to_grant: Vec::new(),
        };

        modify(&mut new);

        let inserted = insert_service(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_service(&tx, |_| ()).await.unwrap();
    }
}
