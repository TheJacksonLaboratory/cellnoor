use std::collections::{BTreeMap, HashMap};

use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetDetailedLinks, ChromiumDatasetParsedFileData,
        ChromiumDatasetParsedFileRecord, ChromiumDatasetQuery, SavedChromiumDatasetRecord,
    },
    chromium_run::SavedChromiumRunRecord,
    file_tree::{FileTreeNode, FileTreeNodeKind},
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
        chromium_runs::index_compact::chromium_run_from_record,
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
    let chromium_runs: Vec<SavedChromiumRunRecord> = row.get("chromium_runs");

    let mut raw_file_paths: Vec<_> = row.get("raw_file_paths");
    raw_file_paths.sort();

    let files = create_file_tree(record.id, &raw_file_paths, row.get("parsed_files"));

    ChromiumDatasetDetailed {
        links: chromium_dataset_detailed_links(record.id, &raw_file_paths),
        record,
        assay: row.get("assay"),
        libraries: libraries.into_iter().map(library_from_record).collect(),
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        chromium_runs: chromium_runs
            .into_iter()
            .map(chromium_run_from_record)
            .collect(),
        files,
    }
}

fn create_file_tree(
    dataset_id: Id,
    raw_file_paths: &[NonemptyString],
    parsed_files: Vec<ChromiumDatasetParsedFileRecord>,
) -> Vec<FileTreeNode<ChromiumDatasetParsedFileData>> {
    let mut parsed_files: HashMap<_, _> = parsed_files
        .into_iter()
        .map(|f| (f.path, f.data.0))
        .collect();

    let mut root = Directory::default();

    for path in raw_file_paths {
        let path_str: &str = path.as_ref();
        let mut name_start = 0;

        let mut directory = &mut root;
        // Walk down the file tree until we get to the innermost directory
        for (slash_idx, _) in path_str.match_indices('/') {
            directory = directory
                .subdirectories
                .entry(path_str[name_start..slash_idx].to_owned())
                .or_insert_with(|| Directory::new(path_str[..slash_idx].to_owned()));
            name_start = slash_idx + 1;
        }

        directory.files.push(FileTreeNode {
            path: path.to_string(),
            name: path_str[name_start..].to_owned(),
            kind: FileTreeNodeKind::File {
                raw_file_link: file_path_to_link(dataset_id, path),
                parsed_contents: parsed_files.remove(path),
            },
        });
    }

    root.into_nodes()
}

#[derive(Default)]
struct Directory {
    path: String,
    // BTreeMap to maintain order
    subdirectories: BTreeMap<String, Directory>,
    files: Vec<FileTreeNode<ChromiumDatasetParsedFileData>>,
}

impl Directory {
    fn new(path: String) -> Self {
        Self {
            path,
            subdirectories: BTreeMap::new(),
            files: Vec::with_capacity(4),
        }
    }

    fn into_nodes(self) -> Vec<FileTreeNode<ChromiumDatasetParsedFileData>> {
        let Self {
            path: _,
            subdirectories,
            mut files,
        } = self;

        files.sort_by(|a, b| a.name.cmp(&b.name));

        subdirectories
            .into_iter()
            .map(|(name, mut directory)| FileTreeNode {
                path: std::mem::take(&mut directory.path),
                name,
                kind: FileTreeNodeKind::Directory {
                    children: directory.into_nodes(),
                },
            })
            .chain(files)
            .collect()
    }
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

#[cfg(test)]
mod tests {
    use cellnoor_types::{
        chromium_dataset::{ChromiumDatasetParsedFileData, ChromiumDatasetParsedFileRecord},
        file_tree::{FileTreeNode, FileTreeNodeKind},
        id::Id,
    };
    use csvranger::TenxCsvValue;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::create_file_tree;
    use crate::state::dev_util::ToNonemptyString;

    #[test]
    fn file_tree() {
        let dataset_id = Id { id: Uuid::nil() };

        let parsed_file_data = ChromiumDatasetParsedFileData::SingleRowCsv {
            data: [("field".to_owned(), TenxCsvValue::I64(0))]
                .into_iter()
                .collect(),
        };

        let file = |full_path: &str, parsed_contents| FileTreeNode {
            path: full_path.to_owned(),
            name: full_path.split('/').next_back().map(str::to_owned).unwrap(),
            kind: FileTreeNodeKind::File {
                raw_file_link: format!("/files/chromium-datasets/{dataset_id}/{full_path}"),
                parsed_contents,
            },
        };
        let dir = |full_path: &str, children| FileTreeNode {
            path: full_path.to_owned(),
            name: full_path
                .rsplit_once('/')
                .map(|(_, p)| p)
                .unwrap_or(full_path)
                .to_owned(),
            kind: FileTreeNodeKind::Directory { children },
        };

        let expected = [
            dir(
                "per_sample_outs",
                vec![
                    dir(
                        "per_sample_outs/sample1",
                        vec![file(
                            "per_sample_outs/sample1/metrics_summary.csv",
                            Some(parsed_file_data.clone()),
                        )],
                    ),
                    dir(
                        "per_sample_outs/sample2",
                        vec![
                            file(
                                "per_sample_outs/sample2/metrics_summary.csv",
                                Some(parsed_file_data.clone()),
                            ),
                            file("per_sample_outs/sample2/web_summary.html", None),
                        ],
                    ),
                ],
            ),
            file("qc_report.html", None),
        ];

        // These paths are intentionally misordered so that we get check that
        // the output is correctly sorted
        let paths = [
            "qc_report.html",
            "per_sample_outs/sample2/web_summary.html",
            "per_sample_outs/sample2/metrics_summary.csv",
            "per_sample_outs/sample1/metrics_summary.csv",
        ]
        .map(|p| p.to_nonempty_string());

        assert_eq!(
            create_file_tree(
                dataset_id,
                &paths,
                vec![
                    ChromiumDatasetParsedFileRecord {
                        dataset_id: *dataset_id,
                        path: "per_sample_outs/sample1/metrics_summary.csv".to_nonempty_string(),
                        data: Json(parsed_file_data.clone())
                    },
                    ChromiumDatasetParsedFileRecord {
                        dataset_id: *dataset_id,
                        path: "per_sample_outs/sample2/metrics_summary.csv".to_nonempty_string(),
                        data: Json(parsed_file_data)
                    }
                ]
            ),
            expected
        );
    }
}
