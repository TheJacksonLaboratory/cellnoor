use std::collections::HashMap;

use axum::{Json, extract::State};
use camino::{Utf8Component, Utf8Path};
use cellnoor_types::{
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetDetailedLinks, ChromiumDatasetParsedFileData,
        ChromiumDatasetParsedFileRecord, ChromiumDatasetQuery, SavedChromiumDatasetRecord,
    },
    file_tree::FileTreeNode,
    id::Id,
    library::SavedLibraryRecord,
    nonempty::NonemptyString,
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    handlers::{
        chromium_datasets::index_compact::chromium_dataset_links,
        libraries::index_compact::library_from_record,
        suspension_pools::index_compact::tagged_specimen_from_record,
    },
    state::AppState,
};

pub async fn index_chromium_datasets_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<ChromiumDatasetQuery>,
) -> Result<Json<Vec<ChromiumDatasetDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| {
            select_chromium_datasets_detailed(tx, &query).await
        })
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_chromium_datasets_detailed(
    tx: &db::Transaction<'_>,
    query: &ChromiumDatasetQuery,
) -> Result<Vec<ChromiumDatasetDetailed>, DbError> {
    static SELECT_DETAILED_CHROMIUM_DATASETS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select_rows(&SELECT_DETAILED_CHROMIUM_DATASETS, query)
        .await?
        .into_iter()
        .map(map_detailed_row)
        .collect())
}

fn map_detailed_row(row: Row) -> ChromiumDatasetDetailed {
    let record: SavedChromiumDatasetRecord = row.get("chromium_dataset");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");
    let libraries: Vec<SavedLibraryRecord> = row.get("libraries");
    let mut raw_file_paths: Vec<_> = row.get("raw_file_paths");
    let parsed_files = row.get("parsed_files");

    let files = create_file_tree(
        record.id,
        record.name.to_string(),
        &mut raw_file_paths,
        parsed_files,
    );

    ChromiumDatasetDetailed {
        links: chromium_dataset_detailed_links(record.id, &raw_file_paths),
        record,
        assay: row.get("assay"),
        libraries: libraries.into_iter().map(library_from_record).collect(),
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        files,
    }
}

fn create_file_tree(
    dataset_id: Id,
    name: String,
    raw_file_paths: &mut [NonemptyString],
    parsed_files: Vec<ChromiumDatasetParsedFileRecord>,
) -> FileTreeNode<ChromiumDatasetParsedFileData> {
    let mut parsed_file_map: HashMap<_, _> =
        parsed_files.iter().map(|f| (&f.path, f.data.0)).collect();

    let mut root = FileTreeNode::Directory {
        name,
        children: Vec::with_capacity(parsed_files.len()),
    };
    let FileTreeNode::Directory { name: _, children } = &mut root else {
        unreachable!()
    };

    raw_file_paths.sort();
    for path in raw_file_paths {
        for component in Utf8Path::new(path).components() {
            let Utf8Component::Normal(component) = component else {
                continue;
            };

            let component = component.to_owned();

            // A simple heuristic to check whether the component is a file
            if component.contains('.') {
                let parsed_contents = parsed_file_map.remove(path);

                children.push(FileTreeNode::File {
                    name: component,
                    raw_file_link: file_path_to_link(dataset_id, path),
                    parsed_contents,
                });
            } else {
                children.push(create_file_tree(
                    dataset_id,
                    component,
                    raw_file_paths,
                    parsed_files,
                ))
            }
        }
    }

    todo!()
}

fn chromium_dataset_detailed_links(
    id: Id,
    raw_file_paths: &[NonemptyString],
) -> ChromiumDatasetDetailedLinks {
    ChromiumDatasetDetailedLinks {
        simple: chromium_dataset_links(id),
        file_dir: dataset_id_to_file_dir(id),
        raw_files: raw_file_paths
            .iter()
            .map(|p| file_path_to_link(id, p))
            .collect(),
    }
}

fn dataset_id_to_file_dir(dataset_id: Id) -> String {
    format!("/files/chromium-datasets/{dataset_id}")
}

fn file_path_to_link(dataset_id: Id, path: &NonemptyString) -> String {
    format!("{}/{path}", dataset_id_to_file_dir(dataset_id))
}
