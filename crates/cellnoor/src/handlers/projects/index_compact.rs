use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    project::{ProjectCompact, ProjectPredicate, ProjectQuery, SavedProjectRecord},
};
use futures::StreamExt;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_projects(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ProjectQuery>,
) -> Result<Json<Vec<ProjectCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_projects_compact(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_projects_compact(
    tx: &db::Transaction<'_>,
    query: &mut ProjectQuery,
) -> Result<Vec<ProjectCompact>, ErrorInner> {
    static SELECT_COMPACT_PROJECT: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    let sql = SELECT_COMPACT_PROJECT.finish_with_query(query);

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(project_from_record).collect().await)
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

pub(super) fn project_simple_links(id: Uuid) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/projects", id.into())
}

pub fn project_from_record(record: SavedProjectRecord) -> ProjectCompact {
    ProjectCompact {
        links: project_simple_links(record.id),
        record,
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        operator::SimpleStringOperator,
        project::{ProjectField, ProjectPredicate, ProjectQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::projects::{
            create::test::insert_test_project, index_compact::select_projects_compact,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_project(&tx, |_| ()).await.unwrap();

        let mut query = ProjectQuery::from_filter(ProjectPredicate::Name(
            SimpleStringOperator::Eq(inserted.record().name.clone().into()).into(),
        ));
        let selected = select_projects_compact(&tx, &mut query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].record.id, inserted.record().id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<ProjectField>(&tx, "project").await;
    }
}
