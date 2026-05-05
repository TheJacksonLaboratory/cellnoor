use axum::{Json, extract::State};
use cellnoor_types::{
    institution::{Institution, InstitutionQuery},
    person::{Person, PersonQuery},
};
use serde_qs::web::QsQuery;

use crate::{auth::AuthUser, db, error::Error, state::AppState};

pub async fn index_people(
    State(state): State<AppState>,
    user: AuthUser,
    QsQuery(query): QsQuery<PersonQuery>,
) -> Result<Json<Vec<Person>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_people(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(super) async fn select_people(
    tx: &db::Transaction<'_>,
    query: &PersonQuery,
) -> Result<Vec<Person>, Error> {
    let (sql, params) = query.to_sql_query();
    let query = format!("select person_public from person_public {sql}");

    Ok(tx.query_into_mapped(&query, params).await?)
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        SimpleStringOperator, StringOperator,
        institution::{InstitutionPredicate, InstitutionQuery},
        person::{PersonPredicate, PersonQuery},
    };
    use uuid::Uuid;

    use crate::{handlers::people::index::select_people, state::test_util::db_client_as_admin};

    #[tokio::test(flavor = "multi_thread")]
    async fn default_select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let records = select_people(&tx, &PersonQuery::default()).await.unwrap();

        assert_eq!(records[0].record.id, Uuid::nil());
    }
}
