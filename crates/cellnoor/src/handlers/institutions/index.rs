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
        .query_stream_into(&query, params)
        .await
        .map(async |stream| stream.map(Institution::from_record).collect().await)?
        .await)
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use cellnoor_types::{
        SimpleStringOperator, UuidOperator,
        institution::{InstitutionPredicate, InstitutionQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::institutions::{
            create::test::insert_test_institution, index::select_institutions,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let inserted = insert_test_institution(&tx, identity).await;

        let by_id = select_institutions(
            &tx,
            &InstitutionQuery::from_filter(
                InstitutionPredicate::Id(UuidOperator::Eq(*inserted.record.id)),
                false,
            ),
        )
        .await
        .unwrap();
        assert_eq!(by_id.len(), 1);
        assert_eq!(by_id[0].record.id, inserted.record.id);

        let none = select_institutions(
            &tx,
            &InstitutionQuery::from_filter(
                InstitutionPredicate::Name(SimpleStringOperator::In(vec!["".to_owned()]).into()),
                false,
            ),
        )
        .await
        .unwrap();
        assert_eq!(none.len(), 0);
    }
}
