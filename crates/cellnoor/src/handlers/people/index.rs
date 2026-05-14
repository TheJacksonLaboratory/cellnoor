use axum::{Json, extract::State};
use cellnoor_types::person::{Person, PersonQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_people(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<PersonQuery>,
) -> Result<Json<Vec<Person>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_people(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_people(
    tx: &db::Transaction<'_>,
    query: &PersonQuery,
) -> Result<Vec<Person>, ErrorInner> {
    let (sql, params) = query.to_sql_query();
    let query = format!("select person_public from person_public {sql}");

    Ok(tx
        .query_stream_into(&query, params)
        .await
        .map(async |stream| stream.map(Person::from_record).collect().await)?
        .await)
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use cellnoor_types::{
        StringOperator,
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

        let (_, inserted) = insert_test_person_and_institution(&tx, identity).await;

        let selected_records = select_people(
            &tx,
            &PersonQuery::from_filter(
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
