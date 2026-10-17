use std::fs::{self, File};

use axum::extract::multipart::Field;
use camino::{Utf8Path, Utf8PathBuf};
use cellnoor_types::{chromium_dataset::ChromiumDatasetParsedFileData, nonempty::NonemptyString};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::handlers::chromium_datasets::upload_files::{
    UploadFilesError, allowed_content_type::AllowedContentType, write_to_db::parse_file,
};

pub(super) async fn write_html_to_disk(
    form_field: &mut Field<'_>,
    dest: &Utf8Path,
) -> Result<(), UploadFilesError> {
    let mut file = tokio::fs::File::create(dest).await?;

    while let Some(chunk) = form_field.chunk().await? {
        file.write_all(&chunk).await?;
    }

    file.flush().await?;

    Ok(())
}

pub(super) async fn write_csv_or_json_to_disk(
    content_type: AllowedContentType,
    data: &[u8],
    dest: &Utf8Path,
) -> Result<Option<ChromiumDatasetParsedFileData>, UploadFilesError> {
    let parsed_file = parse_file(content_type, data)?;
    tokio::fs::write(&dest, &data).await?;

    Ok(parsed_file)
}

pub(super) fn chromium_dataset_dir_path(
    static_files_dir: &Utf8Path,
    dataset_id: Uuid,
) -> Utf8PathBuf {
    static_files_dir
        .join("chromium-datasets")
        .join(dataset_id.to_string())
}

pub(super) fn project_dir_paths(
    static_files_dir: &Utf8Path,
    project_names: &[NonemptyString],
    dataset_name: &NonemptyString,
) -> Vec<Utf8PathBuf> {
    project_names
        .iter()
        .map(|p| {
            static_files_dir
                .join("projects")
                .join(p.as_ref())
                .join(dataset_name.as_ref())
        })
        .collect()
}

pub(super) fn compress_and_link(
    path: &Utf8Path,
    link_paths: &[Utf8PathBuf],
) -> std::io::Result<()> {
    let compressed_path = compress_from_path(path)?;

    symlink_to_project_dirs(path, link_paths)?;
    symlink_to_project_dirs(
        &compressed_path,
        link_paths.iter().map(|p| p.with_added_extension("zst")),
    )
}

fn compress_from_path(original_path: &Utf8Path) -> std::io::Result<Utf8PathBuf> {
    let compressed_path = original_path.with_added_extension("zst");
    let file = File::create(&compressed_path)?;

    zstd::stream::compress_file_into(original_path, file, zstd::DEFAULT_COMPRESSION_LEVEL)?;

    Ok(compressed_path)
}

fn symlink_to_project_dirs(
    source: &Utf8Path,
    destinations: impl IntoIterator<Item = impl AsRef<Utf8Path>>,
) -> std::io::Result<()> {
    for dest in destinations {
        let dest = dest.as_ref();
        fs::create_dir_all(dest.parent().unwrap())?;

        if dest.exists() {
            continue;
        }

        std::os::unix::fs::symlink(source, dest).map_err(|e| {
            std::io::Error::other(format!("unable to symlink {source} to {dest}: {e}"))
        })?;
    }

    Ok(())
}
