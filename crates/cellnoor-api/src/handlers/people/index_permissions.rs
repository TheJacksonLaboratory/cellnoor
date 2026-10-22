use axum::{Json, extract::State};
use cellnoor_types::person::{PersonQuery, PersonWithPermissions};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_people_with_permissions(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<PersonQuery>,
) -> Result<Json<Vec<PersonWithPermissions>>, DbError> {
    state
        .in_transaction(user, async |tx| {
            select_people_with_permissions(tx, &query).await
        })
        .await
}

pub(in super::super) async fn select_people_with_permissions(
    tx: &db::Transaction<'_>,
    query: &PersonQuery,
) -> Result<Vec<PersonWithPermissions>, DbError> {
    static SELECT_PEOPLE_WITH_PERMISSIONS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_with_permissions.sql"));

    tx.select(&SELECT_PEOPLE_WITH_PERMISSIONS, query).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        permission::{Action, Permission, Resource},
        person::{PersonField, PersonPredicate, PersonQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::people::{
            create::insert_test_person_and_institution,
            index_permissions::select_people_with_permissions,
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
        let (_, inserted) = insert_test_person_and_institution(&tx, |p| {
            p.permissions_to_grant = vec![permission];
        })
        .await
        .unwrap();

        let selected = select_people_with_permissions(
            &tx,
            &PersonQuery::from_filter(PersonPredicate::Id(UuidOperator::Eq(inserted.record.id))),
        )
        .await
        .unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].person, inserted.record);
        assert_eq!(selected[0].permissions, [permission]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<PersonField>(&tx, "person_with_permissions").await;
    }
}
