use std::fs;

use axum::{
    Json,
    extract::{Path, State},
};
use camino::Utf8Path;
use cellnoor_types::{
    chromium_dataset::{ChromiumDatasetDetailed, ChromiumDatasetUpdate},
    nonempty::NonemptyString,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        chromium_datasets::{
            show::select_chromium_dataset_by_id,
            upload_files::{DatasetWithProjectNames, fetch_dataset_and_project_names},
        },
    },
    state::AppState,
};

pub async fn update_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<ChromiumDatasetUpdate>,
) -> Result<Json<ChromiumDatasetDetailed>, Error> {
    state
        .in_transaction(user, async |tx| {
            update_chromium_dataset_by_id(
                tx,
                &state.public_files_url,
                &state.static_files_dir,
                id,
                &record,
            )
            .await
        })
        .await
}

async fn update_chromium_dataset_by_id(
    tx: &db::Transaction<'_>,
    files_url: &str,
    static_files_dir: &Utf8Path,
    id: Uuid,
    update: &ChromiumDatasetUpdate,
) -> Result<ChromiumDatasetDetailed, ErrorInner> {
    let old_dataset_name = fetch_dataset_name(tx, id).await?;

    tx.update(id, update).await?;
    let updated = select_chromium_dataset_by_id(tx, files_url, id).await?;

    if old_dataset_name != updated.record.name {
        let DatasetWithProjectNames { project_names, .. } =
            fetch_dataset_and_project_names(tx, &id).await?;

        rename_dataset_directories(
            static_files_dir,
            &project_names,
            old_dataset_name.as_ref(),
            updated.record.name.as_ref(),
        )?;
    }

    Ok(updated)
}

/// Rename the dataset's directory under every project it belongs to.
///
/// A dataset's files are symlinked into `projects/{project}/{dataset}` for each
/// of its projects, so there is one directory per project to rename.
fn rename_dataset_directories(
    static_files_dir: &Utf8Path,
    project_names: &[NonemptyString],
    old_dataset_name: &str,
    new_dataset_name: &str,
) -> Result<(), ErrorInner> {
    for project_name in project_names {
        let project_dir = static_files_dir
            .join("projects")
            .join(project_name.as_ref());
        let (old_dir, new_dir) = (
            project_dir.join(old_dataset_name),
            project_dir.join(new_dataset_name),
        );

        if !old_dir.exists() {
            continue;
        }

        fs::rename(&old_dir, &new_dir).map_err(|e| ErrorInner::FileUpload {
            message: format!("failed to rename dataset directory from {old_dir} to {new_dir}: {e}"),
        })?;
    }

    Ok(())
}

async fn fetch_dataset_name(
    tx: &db::Transaction<'_>,

    dataset_id: Uuid,
) -> Result<NonemptyString, ErrorInner> {
    // In theory, we could just write a query that gets only the name, but it
    // might be wise to reuse code we have already written
    let ds = select_chromium_dataset_by_id(tx, "", dataset_id).await?;

    Ok(ds.record.name)
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;
    use cellnoor_types::{chromium_dataset::ChromiumDatasetUpdate, id::NoId};

    use crate::{
        handlers::chromium_datasets::{
            create::test::insert_test_chromium_dataset, update::update_chromium_dataset_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, ds) = insert_test_chromium_dataset(&tx, |_| ()).await.unwrap();
        let id = *ds.record.id;

        let update = ChromiumDatasetUpdate {
            id: NoId,
            name: "newname".to_nonempty_string(),
            delivered_at: ds.record.delivered_at,
        };

        update_chromium_dataset_by_id(&tx, "", &Utf8PathBuf::new(), id, &update)
            .await
            .unwrap();
    }
}
