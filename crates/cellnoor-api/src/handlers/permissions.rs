use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    Relation,
    permission::{Action, Permission, Resource},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert, Sql},
    handlers::IdParam,
    state::AppState,
};

pub async fn grant_permissions_to_principal(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(permissions): crate::extract::JsonExtractor<Vec<Permission>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            grant_permissions(tx, id, &permissions).await
        })
        .await
}

pub async fn revoke_permissions_from_principal(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(permissions): crate::extract::JsonExtractor<Vec<Permission>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            revoke_permissions(tx, id, &permissions).await
        })
        .await
}

struct PermissionRow {
    principal_id: Uuid,
    permission: Permission,
}

impl Relation for PermissionRow {
    const NAME: &'static str = "permission";
}

impl Insert for PermissionRow {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            principal_id,
            permission,
        } = self;

        vec![
            ("principal_id", principal_id),
            ("action", &permission.action),
            ("resource", &permission.resource),
        ]
    }
}

fn permission_rows(principal_id: Uuid, permissions: &[Permission]) -> Vec<PermissionRow> {
    permissions
        .iter()
        .map(|&permission| PermissionRow {
            principal_id,
            permission,
        })
        .collect()
}

pub(crate) async fn grant_permissions(
    tx: &db::Transaction<'_>,
    principal_id: Uuid,
    permissions: &[Permission],
) -> Result<(), DbError> {
    // This is protected by row-level security. A user cannot give a permission
    // they don't already have
    tx.insert_many_on_conflict_do_nothing(&permission_rows(principal_id, permissions))
        .await
}

pub(crate) async fn revoke_permissions(
    tx: &db::Transaction<'_>,
    principal_id: Uuid,
    permissions: &[Permission],
) -> Result<(), DbError> {
    static REVOKE_PERMISSIONS: &str = "delete from permission where principal_id = $1 and \
                                       (action, resource) in (select * from unnest($2::text[], \
                                       $3::text[]))";

    // A delete matches a set of rows rather than adding them, so the
    // permissions go down as two parallel arrays
    let (actions, resources): (Vec<Action>, Vec<Resource>) =
        permissions.iter().map(|p| (p.action, p.resource)).unzip();

    let n = tx
        .execute(&Sql::new(
            REVOKE_PERMISSIONS,
            vec![&principal_id, &actions, &resources],
        ))
        .await?;

    // Row-level security silently skips the rows the user may not delete
    if n < permissions.len() as u64 {
        return Err(DbError::ResourceNotFound);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        permission::{Action, Permission, Resource},
        person::PersonPredicate,
    };

    use crate::{
        handlers::{
            people::{
                create::insert_test_person_and_institution,
                index_permissions::select_people_with_permissions,
            },
            permissions::revoke_permissions,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn revoke_only_removes_given_pairs() {
        use Action::{Create, Update};
        use Resource::{Institution, Person};

        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let permission = |resource, action| Permission { resource, action };

        let (_, person) = insert_test_person_and_institution(&tx, |p| {
            p.permissions_to_grant = vec![
                permission(Person, Create),
                permission(Person, Update),
                permission(Institution, Create),
                permission(Institution, Update),
            ];
        })
        .await
        .unwrap();

        revoke_permissions(
            &tx,
            person.record.id,
            &[permission(Person, Update), permission(Institution, Create)],
        )
        .await
        .unwrap();

        let remaining = tx
            .select_one(
                PersonPredicate::Id(UuidOperator::Eq(person.record.id)),
                select_people_with_permissions,
            )
            .await
            .unwrap()
            .permissions;

        assert_eq!(remaining.len(), 2);
        assert!(remaining.contains(&permission(Person, Create)));
        assert!(remaining.contains(&permission(Institution, Update)));
    }
}
