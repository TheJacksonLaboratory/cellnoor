use axum::{Json, extract::State};
use cellnoor_types::specimen::{Specimen, SpecimenQuery, SpecimenRecord};
use futures::StreamExt;
use serde_qs::web::QsQuery;

use crate::{auth::AuthUser, db, error::Error, state::AppState};

pub async fn index_specimens(
    State(state): State<AppState>,
    user: AuthUser,
    QsQuery(query): QsQuery<SpecimenQuery>,
) -> Result<Json<Vec<Specimen>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_specimens(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

trait RowToStruct {
    fn from_row(&self) -> Self;
}

pub async fn select_specimens(
    tx: &db::Transaction<'_>,
    query: &SpecimenQuery,
) -> Result<Vec<Specimen>, Error> {
    let (clause, params) = query.to_sql_query();

    let specimens = if query.detailed {
        let sql = format!("select specimen_detailed from specimen_detailed {clause}");
        let stream = tx.query_into_stream(&sql, params).await?;
        stream.map(Specimen::from_detailed_record).collect().await
    } else {
        let sql = format!("select specimen from specimen {clause}");
        let stream = tx.query_into_stream(&sql, params).await?;
        stream.map(Specimen::from_record).collect().await
    };

    Ok(specimens)
}
