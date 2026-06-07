use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    id::Id,
    person::{Person, PersonLinks, PersonPredicate, PersonQuery, SavedPersonRecord},
};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_people(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<PersonQuery>,
) -> Result<Json<Vec<Person>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_people(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn select_people(
    tx: &db::Transaction<'_>,
    query: &mut PersonQuery,
) -> Result<Vec<Person>, ErrorInner> {
    static SELECT_PEOPLE: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    let sql = SELECT_PEOPLE.finish_with_query(query);

    Ok(tx
        .query_stream_into(sql)
        .await
        .map(async |stream| stream.map(person_from_record).collect().await)?
        .await)
}

fn person_links(id: Id) -> PersonLinks {
    let self_ = format!("/people/{id}");

    PersonLinks {
        projects: format!("{self_}/projects"),
        simple: SimpleLinks { self_ },
    }
}

fn person_from_record(record: SavedPersonRecord) -> Person {
    Person {
        links: person_links(record.id),
        record,
    }
}

impl AsPredicate for PersonPredicate {
    fn as_predicate(
        &self,
    ) -> (
        &'static str,
        (&'static str, &(dyn postgres_types::ToSql + Sync)),
    ) {
        let sql = match self {
            Self::Id(u) | Self::InstitutionId(u) => u.as_sql_operator_and_value(),
            Self::Name(s) | Self::Email(s) | Self::Orcid(s) => s.as_sql_operator_and_value(),
            Self::IsStaff(b) | Self::CanManageUsers(b) => b.as_sql_operator_and_value(),
        };

        (self.field_name(), sql)
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
            &mut PersonQuery::from_filter(PersonPredicate::Name(StringOperator::Like(
                inserted.record.name.into(),
            ))),
        )
        .await
        .unwrap();

        assert_eq!(selected_records.len(), 1);
        assert_eq!(*selected_records[0].record.id, *inserted.record.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<PersonField>(&tx, "person_public").await;
    }
}
