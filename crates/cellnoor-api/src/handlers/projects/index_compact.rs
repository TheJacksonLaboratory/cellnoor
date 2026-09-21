use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    project::{ProjectCompact, ProjectQuery, SavedProjectRecord},
};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_projects(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<ProjectQuery>,
) -> Result<Json<Vec<ProjectCompact>>, DbError> {
    state
        .in_transaction(user, async |tx| select_projects_compact(tx, &query).await)
        .await
}

async fn select_projects_compact(
    tx: &db::Transaction<'_>,
    query: &ProjectQuery,
) -> Result<Vec<ProjectCompact>, DbError> {
    static SELECT_COMPACT_PROJECT: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    Ok(tx
        .select(&SELECT_COMPACT_PROJECT, query)
        .await?
        .into_iter()
        .map(project_from_record)
        .collect())
}

pub fn project_from_record(record: SavedProjectRecord) -> ProjectCompact {
    ProjectCompact {
        links: SimpleLinks::from_str_and_id("/projects", record.id.into()),
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

        let query = ProjectQuery::from_filter(ProjectPredicate::Name(
            SimpleStringOperator::Eq(inserted.record().name.clone().into()).into(),
        ));
        let selected = select_projects_compact(&tx, &query).await.unwrap();

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
