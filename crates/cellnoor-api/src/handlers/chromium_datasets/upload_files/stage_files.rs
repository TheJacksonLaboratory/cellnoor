use std::io;

use axum::extract::{Multipart, multipart::Field};
use camino::{Utf8Path, Utf8PathBuf};
use cellnoor_types::chromium_dataset::ChromiumDatasetParsedFileData;
use tokio::io::AsyncWriteExt;

use crate::handlers::chromium_datasets::{
    dataset_directory::{FinalDirectory, StagingDirectory, write_compressed_copy},
    upload_files::{
        UploadFilesError, allowed_content_type::AllowedContentType, upload_path::UploadPath,
        write_to_db::parse_file,
    },
};

pub(super) struct StagedUpload {
    staging_directory: StagingDirectory,
    final_directory: FinalDirectory,
    files: Vec<StagedFile>,
}

pub(super) struct StagedFile {
    pub(super) path_relative_to_dataset_directory: UploadPath,
    pub(super) parsed_data: Option<ChromiumDatasetParsedFileData>,
}

impl StagedUpload {
    pub(super) async fn from_multipart(
        final_directory: FinalDirectory,
        mut multipart: Multipart,
    ) -> Result<Self, UploadFilesError> {
        let staging_directory = StagingDirectory::create_next_to(&final_directory)?;
        let mut files = Vec::new();

        while let Some(field) = multipart.next_field().await? {
            files.push(stage_file(field, &staging_directory).await?);
        }

        Ok(Self {
            final_directory,
            staging_directory,
            files,
        })
    }

    pub(super) fn files(&self) -> &[StagedFile] {
        &self.files
    }

    pub(super) fn move_files_into_final_directory(
        self,
        overwritten_file_paths: &[String],
    ) -> io::Result<()> {
        let Self {
            staging_directory,
            final_directory,
            files: _,
        } = self;

        // An overwritten file's path can differ from the staged file's path in
        // letter case. On a case-sensitive filesystem, moving the staged file
        // in would then leave the overwritten file in place, so it's removed
        // first
        final_directory.remove_files_and_their_compressed_copies(overwritten_file_paths)?;
        final_directory.remove_empty_subdirectories()?;

        staging_directory.move_files_into(&final_directory)
    }
}

async fn stage_file(
    mut field: Field<'_>,
    staging_directory: &StagingDirectory,
) -> Result<StagedFile, UploadFilesError> {
    let content_type = AllowedContentType::from_multipart_form_field(&field)?;
    let path_relative_to_dataset_directory = UploadPath::new(field.file_name())?;

    let staged_file_path = staging_directory
        .create_path_for_file(path_relative_to_dataset_directory.as_str())
        .await?;

    let parsed_data = match content_type {
        AllowedContentType::Html => {
            write_html_to_disk(&mut field, &staged_file_path).await?;

            None
        }
        AllowedContentType::Csv | AllowedContentType::Json => {
            write_csv_or_json_to_disk(content_type, &field.bytes().await?, &staged_file_path)
                .await?
        }
    };

    write_compressed_copy_on_blocking_thread(staged_file_path).await?;

    Ok(StagedFile {
        path_relative_to_dataset_directory,
        parsed_data,
    })
}

async fn write_html_to_disk(
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

async fn write_csv_or_json_to_disk(
    content_type: AllowedContentType,
    data: &[u8],
    dest: &Utf8Path,
) -> Result<Option<ChromiumDatasetParsedFileData>, UploadFilesError> {
    let parsed_file = parse_file(content_type, data)?;
    tokio::fs::write(&dest, &data).await?;

    Ok(parsed_file)
}

async fn write_compressed_copy_on_blocking_thread(file_path: Utf8PathBuf) -> io::Result<()> {
    tokio::task::spawn_blocking(move || write_compressed_copy(&file_path))
        .await
        .map_err(io::Error::other)?
}

#[cfg(test)]
mod tests {
    use std::fs;

    use camino::{Utf8Path, Utf8PathBuf};
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::StagedUpload;
    use crate::handlers::chromium_datasets::{
        dataset_directory::{FinalDirectory, StagingDirectory, write_compressed_copy},
        upload_files::tests::sorted_entry_names,
    };

    struct TestDirectories {
        staged_upload: StagedUpload,
        dataset_directory_path: Utf8PathBuf,
        staging_directory_path: Utf8PathBuf,
    }

    fn create_test_directories(static_files_dir: &Utf8Path) -> TestDirectories {
        let dataset_id = Uuid::new_v4();
        let dataset_directory = FinalDirectory::new(static_files_dir, dataset_id);
        let staging_directory = StagingDirectory::create_next_to(&dataset_directory).unwrap();

        TestDirectories {
            dataset_directory_path: static_files_dir
                .join("chromium-datasets")
                .join(dataset_id.to_string()),
            staging_directory_path: staging_directory.path().to_owned(),
            staged_upload: StagedUpload {
                final_directory: dataset_directory,
                staging_directory,
                files: Vec::new(),
            },
        }
    }

    fn write_file_and_compressed_copy(file_path: &Utf8Path, contents: &str) {
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(file_path, contents).unwrap();
        write_compressed_copy(file_path).unwrap();
    }

    #[test]
    fn staged_file_replaces_file_whose_path_differs_in_case() {
        let static_files_dir = tempdir().unwrap();
        let static_files_dir = Utf8Path::from_path(static_files_dir.path()).unwrap();

        let TestDirectories {
            staged_upload,
            dataset_directory_path,
            staging_directory_path,
        } = create_test_directories(static_files_dir);

        let overwritten_file_path = "per_sample_outs/SAMPLE1/summary.csv";
        let staged_file_path = "per_sample_outs/sample1/summary.csv";

        write_file_and_compressed_copy(&dataset_directory_path.join(overwritten_file_path), "old");
        write_file_and_compressed_copy(&staging_directory_path.join(staged_file_path), "new");

        staged_upload
            .move_files_into_final_directory(&[overwritten_file_path.to_owned()])
            .unwrap();

        let per_sample_outs_path = dataset_directory_path.join("per_sample_outs");

        assert_eq!(sorted_entry_names(&per_sample_outs_path), ["sample1"]);
        assert_eq!(
            sorted_entry_names(&per_sample_outs_path.join("sample1")),
            ["summary.csv", "summary.csv.zst"]
        );
        assert_eq!(
            fs::read_to_string(dataset_directory_path.join(staged_file_path)).unwrap(),
            "new"
        );
        assert!(!staging_directory_path.exists());
    }

    #[test]
    fn directory_that_holds_other_files_is_kept() {
        let static_files_dir = tempdir().unwrap();
        let static_files_dir = Utf8Path::from_path(static_files_dir.path()).unwrap();

        let TestDirectories {
            staged_upload,
            dataset_directory_path,
            staging_directory_path,
        } = create_test_directories(static_files_dir);

        let overwritten_file_path = "per_sample_outs/sample1/summary.csv";
        let other_file_path = "per_sample_outs/sample1/web_summary.html";

        write_file_and_compressed_copy(&dataset_directory_path.join(overwritten_file_path), "old");
        write_file_and_compressed_copy(&dataset_directory_path.join(other_file_path), "other");
        write_file_and_compressed_copy(&staging_directory_path.join("summary.csv"), "new");

        staged_upload
            .move_files_into_final_directory(&[overwritten_file_path.to_owned()])
            .unwrap();

        assert_eq!(
            sorted_entry_names(&dataset_directory_path.join("per_sample_outs/sample1")),
            ["web_summary.html", "web_summary.html.zst"]
        );
    }
}
