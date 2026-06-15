use axum::{Json, extract::State};
use cellnoor_types::chromium_dataset::{
    ChromiumDatasetDetailed, ChromiumDatasetField, NewChromiumDataset, NewChromiumDatasetRecord,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs, SqlBuilder},
    error::{Error, ErrorInner},
    handlers::chromium_datasets::show::select_chromium_dataset_by_id,
    state::AppState,
};

pub async fn create_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewChromiumDataset>,
) -> Result<Json<ChromiumDatasetDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_chromium_dataset(&tx, state.public_files_url(), record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_chromium_dataset(
    tx: &db::Transaction<'_>,
    raw_files_url: &str,
    NewChromiumDataset {
        record,
        library_ids,
    }: NewChromiumDataset,
) -> Result<ChromiumDatasetDetailed, ErrorInner> {
    validate_libraries_have_same_gem_well(tx, library_ids.as_ref()).await?;

    let id = db::insert_into(tx, "chromium_dataset", &record).await?;

    insert_chromium_dataset_libraries(tx, id, library_ids.as_ref()).await?;

    select_chromium_dataset_by_id(tx, raw_files_url, id).await
}

pub async fn validate_libraries_have_same_gem_well(
    tx: &db::Transaction<'_>,
    library_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    static SELECT_N_GEM_WELLS_AND_LIBRARY_TYPES: SqlBuilder =
        SqlBuilder::new(include_str!("create/select_n_gem_wells_and_lib_types.sql"));

    let sql = SELECT_N_GEM_WELLS_AND_LIBRARY_TYPES.finish_with_params(vec![&library_ids]);

    let (n_gem_wells, n_library_types): (i64, i64) = tx
        .query_one(&sql)
        .await
        .map(|row| (row.get("n_gem_wells"), row.get("n_library_types")))?;

    if n_gem_wells != 1 {
        return Err(ErrorInner::DataConstraint {
            resource: Some("chromium_dataset".to_owned()),
            field: Some("library_ids".to_owned()),
            message: "all libraries in a Chromium dataset must come from the same GEM well"
                .to_owned(),
            detail: None,
        });
    }

    if library_ids.len() as i64 != n_library_types {
        return Err(ErrorInner::DataConstraint {
            resource: Some("chromium_dataset".to_owned()),
            field: Some("library_ids".to_owned()),
            message:
                "cannot have multiple instances of the same library type in one Chromium dataset"
                    .to_owned(),
            detail: None,
        });
    }

    Ok(())
}

async fn insert_chromium_dataset_libraries(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    library_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    let rows: Vec<_> = library_ids
        .iter()
        .map(|&library_id| NewChromiumDatasetLibrary {
            dataset_id,
            library_id,
        })
        .collect();

    futures::future::try_join_all(
        rows.iter()
            .map(|r| db::insert_into_no_returning(tx, "chromium_dataset_library", r)),
    )
    .await?;

    Ok(())
}

struct NewChromiumDatasetLibrary {
    dataset_id: Uuid,
    library_id: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewChromiumDatasetLibrary {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            dataset_id,
            library_id,
        } = self;

        [("dataset_id", dataset_id), ("library_id", library_id)]
    }
}

impl AsFieldValuePairs<ChromiumDatasetField, 2> for NewChromiumDatasetRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, ChromiumDatasetField, 2> {
        use ChromiumDatasetField::*;

        let Self {
            id: _,
            name,
            delivered_at,
        } = self;

        [(Name, name), (DeliveredAt, delivered_at)]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        chromium_dataset::{ChromiumDatasetDetailed, NewChromiumDataset, NewChromiumDatasetRecord},
        id::NoId,
    };
    use jiff::Timestamp;
    use nonempty::NonemptyVec;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            chromium_datasets::create::insert_chromium_dataset,
            libraries::create::test::insert_test_library,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_chromium_dataset<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewChromiumDataset, ChromiumDatasetDetailed), ErrorInner>
    where
        F: FnMut(&mut NewChromiumDataset),
    {
        let (_, library) = insert_test_library(tx, |_| ()).await?;

        let mut new = NewChromiumDataset {
            record: NewChromiumDatasetRecord {
                id: NoId {},
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                delivered_at: Timestamp::now(),
            },
            library_ids: NonemptyVec::new(vec![*library.record.id]).unwrap(),
        };

        modify(&mut new);

        let inserted = insert_chromium_dataset(tx, "files", new.clone()).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_chromium_dataset(&tx, |_| ()).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn libraries_from_different_gem_wells_fail() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, library1) = insert_test_library(&tx, |_| ()).await.unwrap();
        let (_, library2) = insert_test_library(&tx, |_| ()).await.unwrap();

        let new = NewChromiumDataset {
            record: NewChromiumDatasetRecord {
                id: NoId {},
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                delivered_at: Timestamp::now(),
            },
            library_ids: NonemptyVec::new(vec![*library1.record.id, *library2.record.id]).unwrap(),
        };

        let err = insert_chromium_dataset(&tx, "files", new)
            .await
            .unwrap_err();

        assert_eq!(
            err,
            ErrorInner::DataConstraint {
                resource: Some("chromium_dataset".to_owned()),
                field: Some("library_ids".to_owned()),
                message: "all libraries in a Chromium dataset must come from the same GEM well"
                    .to_owned(),
                detail: None,
            }
        );
    }
}
