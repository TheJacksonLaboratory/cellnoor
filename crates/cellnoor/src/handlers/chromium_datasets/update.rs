use std::fs;

use axum::{
    Json,
    extract::{Path, State},
};
use camino::Utf8Path;
use cellnoor_types::chromium_dataset::{ChromiumDatasetDetailed, ChromiumDatasetUpdate};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, chromium_datasets::show::select_chromium_dataset_by_id},
    state::AppState,
};

pub async fn update_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<ChromiumDatasetUpdate>,
) -> Result<Json<ChromiumDatasetDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_chromium_dataset_by_id(
        &tx,
        state.public_files_url(),
        state.static_files_dir(),
        id,
        &record,
    )
    .await
    .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_chromium_dataset_by_id(
    tx: &db::Transaction<'_>,
    files_url: &str,
    static_files_dir: &Utf8Path,
    id: Uuid,
    update: &ChromiumDatasetUpdate,
) -> Result<ChromiumDatasetDetailed, ErrorInner> {
    let old_dataset_name = fetch_dataset_name(tx, id).await?;

    db::update(tx, "chromium_dataset", id, update).await?;
    let updated = select_chromium_dataset_by_id(tx, files_url, id).await?;

    if old_dataset_name != updated.record.name {
        rename_dataset_directory(
            static_files_dir,
            old_dataset_name.as_ref(),
            updated.record.name.as_ref(),
        )?;
    }

    Ok(updated)
}

fn rename_dataset_directory(
    static_files_dir: &Utf8Path,
    old_dataset_name: &str,
    new_dataset_name: &str,
) -> Result<(), ErrorInner> {
    let old_dataset_dir = static_files_dir.join(format!("**/{old_dataset_name}"));

    // Dataset names are unique, so we know there's only one
    let paths = glob::glob(old_dataset_dir.as_str())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let Some([old_path]) = paths.as_array() else {
        return Ok(());
    };

    let new_dataset_dir = old_path.parent().unwrap().join(new_dataset_name);

    fs::rename(old_path, &new_dataset_dir).map_err(|e| ErrorInner::FileUpload {
        message: format!(
            "failed to rename dataset directory from {old_dataset_dir} to {}: {}",
            new_dataset_dir.to_str().unwrap(),
            e
        ),
    })?;

    Ok(())
}

async fn fetch_dataset_name(
    tx: &db::Transaction<'_>,

    dataset_id: Uuid,
) -> Result<NonemptyString, ErrorInner> {
    // In theory, we could just write a query that gets only the name, but it might
    // be wise to reuse code we have already written
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
            id: NoId {},
            name: "newname".to_nonempty_string(),
            delivered_at: ds.record.delivered_at,
        };

        update_chromium_dataset_by_id(&tx, "", &Utf8PathBuf::new(), id, &update)
            .await
            .unwrap();
    }
}
