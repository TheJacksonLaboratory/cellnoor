use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, InstitutionQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_institutions(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<InstitutionQuery>,
) -> Result<Json<Vec<Institution>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_institutions(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_institutions(
    tx: &db::Transaction<'_>,
    query: &InstitutionQuery,
) -> Result<Vec<Institution>, ErrorInner> {
    let (sql, params) = query.to_sql_query();
    let query = format!("select institution from institution {sql}");

    Ok(tx
        .query_into_stream(&query, params)
        .await
        .map(async |stream| stream.map(Institution::from_record).collect().await)?
        .await)
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        SimpleStringOperator, StringOperator,
        institution::{InstitutionPredicate, InstitutionQuery},
    };
    use uuid::Uuid;

    use crate::{
        handlers::institutions::index::select_institutions, state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn default_select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let institutions = select_institutions(&tx, &InstitutionQuery::default())
            .await
            .unwrap();

        assert_eq!(institutions[0].record.id, Uuid::nil());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn filtered_select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let query = InstitutionQuery::from_filter(
            InstitutionPredicate::Name(StringOperator::Like("Jackson%".to_owned())),
            false,
        );

        let institutions = select_institutions(&tx, &query).await.unwrap();

        assert_eq!(institutions.len(), 1);
        assert_eq!(institutions[0].record.id, Uuid::nil());

        let query = InstitutionQuery::from_filter(
            InstitutionPredicate::Name(SimpleStringOperator::In(vec!["".to_owned()]).into()),
            false,
        );

        assert_eq!(select_institutions(&tx, &query).await.unwrap().len(), 0);
    }
}
