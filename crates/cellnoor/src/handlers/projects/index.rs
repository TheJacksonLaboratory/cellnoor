use axum::{Json, extract::State};
use cellnoor_types::project::{Project, ProjectPredicate, ProjectQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, BaseSqlStmt},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_projects(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ProjectQuery>,
) -> Result<Json<Vec<Project>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_projects(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_projects(
    tx: &db::Transaction<'_>,
    query: &mut ProjectQuery,
) -> Result<Vec<Project>, ErrorInner> {
    let base_stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = BaseSqlStmt::new(base_stmt).finish_with_query(query)?;

    let projects = if query.detailed {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(Project::from_detailed_record).collect().await
    } else {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(Project::from_record).collect().await
    };

    Ok(projects)
}

impl AsPredicate for ProjectPredicate {
    fn as_predicate(
        &self,
    ) -> (
        &'static str,
        (&'static str, &(dyn postgres_types::ToSql + Sync)),
    ) {
        let sql = match self {
            Self::Id(u) => u.as_sql_operator_and_value(),
            Self::Name(s) => s.as_sql_operator_and_value(),
            Self::StartedAt(t) | Self::EndedAt(t) => t.as_sql_operator_and_value(),
        };

        (self.field_name(), sql)
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        operator::SimpleStringOperator,
        project::{ProjectPredicate, ProjectQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::projects::{create::test::insert_test_project, index::select_projects},
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_project(&tx, |_| ()).await.unwrap();

        let mut query = ProjectQuery::from_filter(
            ProjectPredicate::Name(
                SimpleStringOperator::Eq(inserted.record().name.clone().into()).into(),
            ),
            false,
        );
        let selected = select_projects(&tx, &mut query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(*selected[0].record().id, *inserted.record().id);
    }
}
