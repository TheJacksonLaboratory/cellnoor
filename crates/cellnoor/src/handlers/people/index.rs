use axum::{Json, extract::State};
use cellnoor_types::person::{Person, PersonPredicate, PersonQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, BaseSqlStmt},
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

pub async fn select_people(
    tx: &db::Transaction<'_>,
    query: &mut PersonQuery,
) -> Result<Vec<Person>, ErrorInner> {
    let sql = BaseSqlStmt::new(include_str!("index/select.sql")).finish_with_query(query)?;

    Ok(tx
        .query_stream_into(sql)
        .await
        .map(async |stream| stream.map(Person::from_record).collect().await)?
        .await)
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
        };

        (self.field_name(), sql)
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        operator::StringOperator,
        person::{PersonPredicate, PersonQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
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
            &mut PersonQuery::from_filter(
                PersonPredicate::Name(StringOperator::Like(inserted.record.name.into())),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(selected_records.len(), 1);
        assert_eq!(*selected_records[0].record.id, *inserted.record.id);
    }
}
