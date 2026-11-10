use std::{
    fs::{self, File},
    io,
};

use camino::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;
use uuid::Uuid;

pub(super) struct FinalDirectory {
    path: Utf8PathBuf,
}

impl FinalDirectory {
    pub(super) fn new(static_files_dir: &Utf8Path, dataset_id: Uuid) -> Self {
        Self {
            path: static_files_dir
                .join("chromium-datasets")
                .join(dataset_id.to_string()),
        }
    }

    pub(super) fn remove(&self) -> io::Result<()> {
        ignore_not_found(fs::remove_dir_all(&self.path))
    }

    pub(super) fn remove_files_and_their_compressed_copies(
        &self,
        file_paths_relative_to_dataset_directory: &[String],
    ) -> io::Result<()> {
        for file_path_relative_to_dataset_directory in file_paths_relative_to_dataset_directory {
            let file_path = self.path.join(file_path_relative_to_dataset_directory);

            ignore_not_found(fs::remove_file(&file_path))?;
            ignore_not_found(fs::remove_file(compressed_copy_path(&file_path)))?;
        }

        Ok(())
    }

    pub(super) fn remove_empty_subdirectories(&self) -> io::Result<()> {
        ignore_not_found(remove_empty_subdirectories_of(&self.path))
    }
}

pub(super) struct StagingDirectory {
    // The directory is deleted from disk when this is dropped
    temp_directory: TempDir,
}

impl StagingDirectory {
    // The staging directory is a sibling of the dataset directory so that both
    // are on the same filesystem, which `fs::rename` requires
    pub(super) fn create_next_to(dataset_directory: &FinalDirectory) -> io::Result<Self> {
        let all_datasets_directory_path = dataset_directory.path.parent().unwrap();
        fs::create_dir_all(all_datasets_directory_path)?;

        let temp_directory = tempfile::Builder::new()
            .prefix(".staging-")
            .tempdir_in(all_datasets_directory_path)?;

        Ok(Self { temp_directory })
    }

    pub(super) fn path(&self) -> &Utf8Path {
        // The directory was created inside a UTF-8 path, and the random part of
        // its name is ASCII
        Utf8Path::from_path(self.temp_directory.path()).unwrap()
    }

    // The staging directory has the same layout as the final directory, so a
    // file's path relative to either one is the same
    pub(super) async fn create_path_for_file(
        &self,
        file_path_relative_to_dataset_directory: &str,
    ) -> io::Result<Utf8PathBuf> {
        let staged_file_path = self.path().join(file_path_relative_to_dataset_directory);
        tokio::fs::create_dir_all(staged_file_path.parent().unwrap()).await?;

        Ok(staged_file_path)
    }

    pub(super) fn move_files_into(self, dataset_directory: &FinalDirectory) -> io::Result<()> {
        move_files(self.path(), &dataset_directory.path)
    }
}

pub(super) fn write_compressed_copy(file_path: &Utf8Path) -> io::Result<()> {
    let compressed_copy = File::create(compressed_copy_path(file_path))?;

    zstd::stream::compress_file_into(file_path, compressed_copy, zstd::DEFAULT_COMPRESSION_LEVEL)?;

    Ok(())
}

fn compressed_copy_path(file_path: &Utf8Path) -> Utf8PathBuf {
    file_path.with_added_extension("zst")
}

fn move_files(source_directory: &Utf8Path, destination_directory: &Utf8Path) -> io::Result<()> {
    fs::create_dir_all(destination_directory)?;

    for entry in source_directory.read_dir_utf8()? {
        let src = entry?;
        let dest = destination_directory.join(src.file_name());

        if src.file_type()?.is_dir() {
            move_files(src.path(), &dest)?;
        } else {
            fs::rename(src.path(), &dest)?;
        }
    }

    Ok(())
}

fn remove_empty_subdirectories_of(directory: &Utf8Path) -> io::Result<()> {
    for entry in directory.read_dir_utf8()? {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let subdirectory = entry.path();

        // A subdirectory that holds only empty directories is empty once those
        // are removed, so they are removed before checking it
        remove_empty_subdirectories_of(subdirectory)?;

        if directory_is_empty(subdirectory)? {
            fs::remove_dir(subdirectory)?;
        }
    }

    Ok(())
}

fn directory_is_empty(directory_path: &Utf8Path) -> io::Result<bool> {
    Ok(directory_path.read_dir_utf8()?.next().is_none())
}

fn ignore_not_found(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use camino::Utf8Path;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::{FinalDirectory, remove_empty_subdirectories_of};

    #[test]
    fn removing_a_missing_dataset_directory_succeeds() {
        let static_files_dir = tempdir().unwrap();
        let static_files_dir = Utf8Path::from_path(static_files_dir.path()).unwrap();

        FinalDirectory::new(static_files_dir, Uuid::new_v4())
            .remove()
            .unwrap();
    }

    #[test]
    fn only_empty_subdirectories_are_removed() {
        let directory = tempdir().unwrap();
        let directory_path = Utf8Path::from_path(directory.path()).unwrap();

        let nested_empty_directory_path = directory_path.join("empty/also_empty");
        let directory_with_file_path = directory_path.join("has_file");

        fs::create_dir_all(&nested_empty_directory_path).unwrap();
        fs::create_dir_all(&directory_with_file_path).unwrap();
        fs::write(directory_with_file_path.join("file"), "contents").unwrap();

        remove_empty_subdirectories_of(directory_path).unwrap();

        assert!(!directory_path.join("empty").exists());
        assert!(directory_with_file_path.join("file").exists());
    }
}
