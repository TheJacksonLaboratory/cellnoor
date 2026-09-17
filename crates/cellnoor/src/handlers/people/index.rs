use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    person::{Person, PersonLinks, PersonQuery, SavedPersonRecord},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_people(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<PersonQuery>,
) -> Result<Json<Vec<Person>>, Error> {
    state
        .in_transaction(user, async |tx| select_people(tx, &query).await)
        .await
}

pub(in super::super) async fn select_people(
    tx: &db::Transaction<'_>,
    query: &PersonQuery,
) -> Result<Vec<Person>, ErrorInner> {
    static SELECT_PEOPLE: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    Ok(tx
        .select(&SELECT_PEOPLE, query)
        .await?
        .into_iter()
        .map(person_from_record)
        .collect())
}

fn person_links(id: Uuid) -> PersonLinks {
    let simple = SimpleLinks::from_str_and_id("people", id.into());

    PersonLinks {
        projects: format!("{}/projects", simple.self_),
        simple,
    }
}

fn person_from_record(record: SavedPersonRecord) -> Person {
    Person {
        links: person_links(record.id),
        record,
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        operator::StringOperator,
        person::{PersonField, PersonPredicate, PersonQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::people::{
            create::test::insert_test_person_and_institution, index::select_people,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        let selected_records = select_people(
            &tx,
            &PersonQuery::from_filter(PersonPredicate::Name(StringOperator::Like(
                inserted.record.name.into(),
            ))),
        )
        .await
        .unwrap();

        assert_eq!(selected_records.len(), 1);
        assert_eq!(selected_records[0].record.id, inserted.record.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<PersonField>(&tx, "person_public").await;
    }
}
