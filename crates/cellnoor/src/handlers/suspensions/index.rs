use axum::{Json, extract::State};
use cellnoor_types::suspension::{Suspension, SuspensionQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, construct_select_stmt},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_suspensions(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SuspensionQuery>,
) -> Result<Json<Vec<Suspension>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspensions(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_suspensions(
    tx: &db::Transaction<'_>,
    query: &SuspensionQuery,
) -> Result<Vec<Suspension>, ErrorInner> {
    let suspensions = if query.detailed {
        let (sql, params) =
            construct_select_stmt("suspension_detailed", &["suspension_detailed"], None, query);
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Suspension::from_detailed_record).collect().await
    } else {
        // We query through `suspension_to_specimen` rather than `suspension` because
        // the predicate can filter on the parent specimen's fields, which need
        // a `(specimen)` row in scope.
        let (sql, params) =
            construct_select_stmt("suspension_to_specimen", &["suspension"], None, query);
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Suspension::from_record).collect().await
    };

    Ok(suspensions)
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        operator::UuidOperator,
        suspension::{SuspensionPredicateInner, SuspensionQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::suspensions::{
            create::test::insert_test_suspension_and_specimen, index::select_suspensions,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();

        let suspensions = select_suspensions(
            &tx,
            &SuspensionQuery::from_filter(
                SuspensionPredicateInner::Id(UuidOperator::Eq(*inserted.record().id)).into(),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(suspensions.len(), 1);
        assert_eq!(suspensions[0].record(), inserted.record());
    }
}
