use axum::{Json, extract::State};
use cellnoor_types::suspension::{Suspension, SuspensionQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, SqlTemplate},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_suspensions(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SuspensionQuery>,
) -> Result<Json<Vec<Suspension>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspensions(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_suspensions(
    tx: &db::Transaction<'_>,
    query: &mut SuspensionQuery,
) -> Result<Vec<Suspension>, ErrorInner> {
    let stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = SqlTemplate::new(stmt).finish_with_query(query)?;

    let suspensions = if query.detailed {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(Suspension::from_detailed_record).collect().await
    } else {
        let stream = tx.query_stream_into(sql).await?;
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
            &mut SuspensionQuery::from_filter(
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
