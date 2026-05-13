use axum::{Json, extract::State};
use cellnoor_types::specimen::{Specimen, SpecimenQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_specimens(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SpecimenQuery>,
) -> Result<Json<Vec<Specimen>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_specimens(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_specimens(
    tx: &db::Transaction<'_>,
    query: &SpecimenQuery,
) -> Result<Vec<Specimen>, ErrorInner> {
    let (clause, params) = query.to_sql_query();

    let specimens = if query.detailed {
        let sql = format!("select specimen_detailed from specimen_detailed {clause}");
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Specimen::from_detailed_record).collect().await
    } else {
        let sql = format!("select specimen from specimen {clause}");
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Specimen::from_record).collect().await
    };

    Ok(specimens)
}
