use axum::{Json, extract::State};
use cellnoor_types::specimen::{Specimen, SpecimenPredicate, SpecimenQuery};
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, BaseSqlStmt},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_specimens(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SpecimenQuery>,
) -> Result<Json<Vec<Specimen>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_specimens(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_specimens(
    tx: &db::Transaction<'_>,
    query: &mut SpecimenQuery,
) -> Result<Vec<Specimen>, ErrorInner> {
    let stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = BaseSqlStmt::new(stmt).finish_with_query(query)?;

    let specimens = if query.detailed {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(Specimen::from_detailed_record).collect().await
    } else {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(Specimen::from_record).collect().await
    };

    Ok(specimens)
}

impl AsPredicate for SpecimenPredicate {
    fn as_predicate(&self) -> (&'static str, (&'static str, &(dyn ToSql + Sync))) {
        let sql = match self {
            Self::Id(u) | Self::SubmittedBy(u) | Self::ProjectId(u) | Self::ReturnedBy(u) => {
                u.as_sql_operator_and_value()
            }
            Self::ReadableId(s) | Self::Name(s) | Self::Tissue(s) => s.as_sql_operator_and_value(),
            Self::ReceivedAt(t) | Self::ReturnedAt(t) => t.as_sql_operator_and_value(),
            Self::Species(sp) | Self::HostSpecies(sp) => sp.as_sql_operator_and_value(),
            Self::Type(ty) => ty.as_sql_operator_and_value(),
            Self::EmbeddedIn(e) => e.as_sql_operator_and_value(),
            Self::Fixative(f) => f.as_sql_operator_and_value(),
            Self::ThermalPreservationMethod(tp) => tp.as_sql_operator_and_value(),
            Self::AdditionalData(d) => d.as_sql_operator_and_value(),
        };

        (self.field_name(), sql)
    }
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
            &mut SpecimenQuery::from_filter(
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
