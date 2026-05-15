use axum::{Json, extract::State};
use cellnoor_types::specimen::{Specimen, SpecimenQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, construct_select_stmt},
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
    let specimens = if query.detailed {
        let (sql, params) =
            construct_select_stmt("specimen_detailed", &["specimen_detailed"], None, query);
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Specimen::from_detailed_record).collect().await
    } else {
        let (sql, params) = construct_select_stmt("specimen", &["specimen"], None, query);
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(Specimen::from_record).collect().await
    };

    Ok(specimens)
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        operator::UuidOperator,
        specimen::{SpecimenPredicate, SpecimenQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::specimens::{
            create::test::insert_test_specimen_and_project, index::select_specimens,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();

        let specimens = select_specimens(
            &tx,
            &SpecimenQuery::from_filter(
                SpecimenPredicate::Id(UuidOperator::Eq(*inserted.record().id)),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(specimens.len(), 1);
        assert_eq!(specimens[0].record(), inserted.record());
    }
}
