use axum::{Json, extract::State};
use cellnoor_types::{
    person::PermissionsToGrant,
    service::{NewService, NewServiceRecord, Service, ServiceField},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::{
        people::create::{PermissionSet, permission_to_permission_set},
        services::{access::add_people::insert_service_accesses, index::select_service_by_id},
    },
    state::AppState,
};

pub async fn create_service(
    State(state): State<AppState>,
    user: AuthUser,
    Json(service): Json<NewService>,
) -> Result<Json<Service>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_service(&tx, &service).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_service(
    tx: &db::Transaction<'_>,
    NewService {
        record,
        users,
        permissions_to_grant,
    }: &NewService,
) -> Result<Service, ErrorInner> {
    let id = db::insert_into(tx, "service", record).await?;

    insert_service_accesses(tx, id, users).await?;

    let (_, service) = tokio::try_join!(
        create_db_user(tx, id, permissions_to_grant),
        select_service_by_id(tx, id)
    )?;

    Ok(service)
}

async fn create_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &PermissionsToGrant,
) -> Result<(), ErrorInner> {
    let permission_sets: Vec<_> = permissions
        .iter()
        .map(permission_to_permission_set)
        .collect();

    tx.execute_raw_sql(
        "select create_service_user_with_permissions($1, $2)",
        &[&user_id, &permission_sets],
    )
    .await?;

    Ok(())
}

// `owned_by` is intentionally omitted: the database fills it from
// `current_user::uuid`, and RLS guarantees it equals the caller
impl AsFieldValuePairs<ServiceField, 3> for NewServiceRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, ServiceField, 3> {
        use ServiceField::*;

        let Self {
            description,
            can_read_all_projects,
            can_admin_users,
        } = self;

        [
            (Description, description),
            (CanReadAllProjects, can_read_all_projects),
            (CanAdminUsers, can_admin_users),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        id::NoId,
        person::PermissionsToGrant,
        service::{NewService, NewServiceRecord, Service},
    };
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::services::create::insert_service,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_service<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewService, Service), ErrorInner>
    where
        F: FnMut(&mut NewService),
    {
        let mut new = NewService {
            record: NewServiceRecord {
                description: Some(Uuid::new_v4().to_string().to_nonempty_string()),
                can_read_all_projects: false,
                can_admin_users: false,
            },
            users: vec![Uuid::nil()],
            permissions_to_grant: PermissionsToGrant::default(),
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
