use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    specimen::{SavedSpecimenRecordDetailed, SpecimenDetailed, SpecimenQuery},
};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    handlers::{
        projects::index_compact::project_from_record,
        specimens::index_compact::specimen_from_record,
    },
    state::AppState,
};

pub async fn index_specimens_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<SpecimenQuery>,
) -> Result<Json<Vec<SpecimenDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| select_specimens_detailed(tx, &query).await)
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_specimens_detailed(
    tx: &db::Transaction<'_>,
    query: &SpecimenQuery,
) -> Result<Vec<SpecimenDetailed>, DbError> {
    static SELECT_DETAILED_SPECIMEN: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select(&SELECT_DETAILED_SPECIMEN, query)
        .await?
        .into_iter()
        .map(specimen_from_detailed_record)
        .collect())
}

fn specimen_from_detailed_record(
    SavedSpecimenRecordDetailed {
        specimen,
        project,
        measurements,
        ancestors,
    }: SavedSpecimenRecordDetailed,
) -> SpecimenDetailed {
    SpecimenDetailed {
        links: SimpleLinks::from_str_and_id("/specimens", specimen.id),
        record: specimen,
        project: project_from_record(project),
        measurements,
        ancestors: ancestors.into_iter().map(specimen_from_record).collect(),
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::specimen::creation::SpecimenOrigin;
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::specimens::create::insert_test_specimen_and_project,
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn ancestors_are_ordered_parent_first() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, root) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let (_, child) = insert_test_specimen_and_project(&tx, |sp| {
            sp.simple.origin = SpecimenOrigin::Derived {
                parent_specimen_id: *root.record.id,
            };
        })
        .await
        .unwrap();
        let (_, grandchild) = insert_test_specimen_and_project(&tx, |sp| {
            sp.simple.origin = SpecimenOrigin::Derived {
                parent_specimen_id: *child.record.id,
            };
        })
        .await
        .unwrap();

        assert!(root.ancestors.is_empty());

        let ancestors: Vec<_> = grandchild.ancestors.into_iter().map(|a| a.record).collect();
        assert_eq!(ancestors, [child.record, root.record]);
    }
}
