use axum::{Json, extract::State};
use cellnoor_types::suspension::{Suspension, SuspensionQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db,
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
    let (clause, params) = query.to_sql_query();

    let suspensions = if query.detailed {
        let sql = format!("select suspension_detailed from suspension_detailed {clause}");
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Suspension::from_detailed_record).collect().await
    } else {
        // We query through `suspension_to_specimen` rather than `suspension` because
        // the predicate can filter on the parent specimen's fields, which need
        // a `(specimen)` row in scope.
        let sql = format!("select suspension from suspension_to_specimen {clause}");
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Suspension::from_record).collect().await
    };

    Ok(suspensions)
}
