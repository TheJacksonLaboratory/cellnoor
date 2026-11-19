use std::{
    collections::{BTreeMap, HashMap},
    slice::Iter,
};

use axum::{Json, extract::State};
use cellnoor_types::{
    cdna::{SavedCdnaRecord, creation::LibraryType},
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetDetailedLinks, ChromiumDatasetParsedFileData,
        ChromiumDatasetParsedFileRecord, ChromiumDatasetQuery, KeySpecimenMetrics,
        SavedChromiumDatasetRecord,
    },
    chromium_run::creation::ocm::OcmBarcodeId,
    file_tree::{FileTreeNode, FileTreeNodeKind},
    id::Id,
    library::SavedLibraryRecord,
    nonempty::NonemptyString,
    specimen::SavedSpecimenRecord,
    suspension_pool::SavedTaggedSpecimenRecord,
};
use csvranger::TenxCsvValue;
use deadpool_postgres::tokio_postgres::Row;
use indexmap::IndexMap;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    handlers::{
        cdna::index_compact::cdna_from_record,
        chromium_datasets::{
            index_compact::chromium_dataset_links,
            index_detailed::{
                process_transposed_csv::extract_key_metrics_from_transposed_csv,
                standard_csv::extract_key_metrics_from_standard_csv,
            },
        },
        chromium_runs::index_compact::chromium_run_from_record,
        libraries::index_compact::library_from_record,
        suspension_pools::index_compact::tagged_specimen_from_record,
    },
    state::AppState,
};

mod process_transposed_csv;
mod renamed_metrics;
mod standard_csv;

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
        .iter()
        .map(map_detailed_row)
        .collect())
}

fn map_detailed_row(row: &Row) -> ChromiumDatasetDetailed {
    let record: SavedChromiumDatasetRecord = row.get("chromium_dataset");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");
    let libraries: Vec<SavedLibraryRecord> = row.get("libraries");
    let cdna: Vec<SavedCdnaRecord> = row.get("cdna");
    let chromium_run = row.get("chromium_run");

    let mut raw_files: Vec<_> = row.get("raw_files");
    raw_files.sort_by(|p1: &RawFileRecord, p2| p1.path.cmp(&p2.path));

    let parsed_files: Vec<_> = row.get("parsed_files");

    let mut library_type_specimen_metrics_map =
        create_library_type_specimen_metrics_map(&specimens);
    let extracted_metrics = extract_metrics_from_parsed_files(
        &parsed_files,
        record.pipeline_name.as_ref(),
        record.pipeline_command.as_ref(),
    );
    let key_metrics = build_key_metrics(&mut library_type_specimen_metrics_map, extracted_metrics);

    ChromiumDatasetDetailed {
        links: chromium_dataset_detailed_links(record.id, &raw_files),
        record,
        reference_package: row.get("reference_package"),
        assay: row.get("assay"),
        libraries: libraries.into_iter().map(library_from_record).collect(),
        cdna: cdna.into_iter().map(cdna_from_record).collect(),
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        chromium_run: chromium_run_from_record(chromium_run),
        key_metrics,
        files: create_file_tree(&raw_files, parsed_files),
    }
}

static PSEUDO_BARCODE: &str = "specimen";

fn create_library_type_specimen_metrics_map<'a>(
    specimens: &'a [SavedTaggedSpecimenRecord],
) -> HashMap<(LibraryType, &'a str), KeySpecimenMetrics> {
    let extract_tag = |SavedTaggedSpecimenRecord {
                           specimen:
                               SavedSpecimenRecord {
                                   id,
                                   name,
                                   readable_id,
                                   ..
                               },
                           multiplexing_tag,
                           ocm_barcode_id,
                       }: &'a SavedTaggedSpecimenRecord| {
        (
            multiplexing_tag
                .as_ref()
                .map(|m| &m.tag_id)
                .map(NonemptyString::as_ref)
                .or(ocm_barcode_id.as_ref().map(OcmBarcodeId::as_ref))
                .unwrap_or(PSEUDO_BARCODE),
            KeySpecimenMetrics {
                specimen_id: **id,
                specimen_readable_id: readable_id.clone(),
                specimen_name: name.clone(),
                renamed_metrics: IndexMap::with_capacity(8),
            },
        )
    };

    let barcode_specimen_map = specimens.iter().map(extract_tag);
    let mut metrics_map = HashMap::with_capacity(LibraryType::VARIANTS.len() * specimens.len());

    for lt in LibraryType::VARIANTS {
        for (bc, sp) in barcode_specimen_map.clone() {
            metrics_map.insert((*lt, bc), sp);
        }
    }

    metrics_map
}

struct MetricsExtractedCsvRow<'a> {
    library_type: LibraryType,
    specimen_barcode: &'a str,
    renamed_metric: &'static str,
    metric_value: &'a TenxCsvValue,
}

fn extract_metrics_from_parsed_files<'a>(
    parsed_files: &'a [ChromiumDatasetParsedFileRecord],
    pipeline: &str,
    pipeline_command: &str,
) -> impl Iterator<Item = MetricsExtractedCsvRow<'a>> {
    use ChromiumDatasetParsedFileData::*;

    let extract = |ChromiumDatasetParsedFileRecord {
                       data: postgres_types::Json(d),
                       ..
                   }: &'a ChromiumDatasetParsedFileRecord| {
        match d {
            StandardCsv { data, .. } => {
                extract_key_metrics_from_standard_csv(data, pipeline, pipeline_command)
                    .map(either::Left)
            }
            TransposedCsv { data } => {
                Some(either::Right(extract_key_metrics_from_transposed_csv(data)))
            }
            // We get everything we need from the CSVs
            Json { .. } => None,
        }
    };

    parsed_files.iter().flat_map(extract).flatten()
}

fn build_key_metrics<'a>(
    library_type_specimen_metrics_map: &mut HashMap<(LibraryType, &'a str), KeySpecimenMetrics>,
    extracted_metrics: impl Iterator<Item = MetricsExtractedCsvRow<'a>>,
) -> IndexMap<LibraryType, Vec<KeySpecimenMetrics>> {
    // We already know this has the order we want because GeneExpression is the
    // first LibraryType
    let mut key_metrics: IndexMap<LibraryType, Vec<KeySpecimenMetrics>> = LibraryType::VARIANTS
        .iter()
        .copied()
        .map(|lt| (lt, Vec::with_capacity(8)))
        .collect();

    for MetricsExtractedCsvRow {
        library_type,
        specimen_barcode,
        renamed_metric,
        metric_value,
    } in extracted_metrics
    {
        let Some(mut specimen_metrics) =
            library_type_specimen_metrics_map.remove(&(library_type, specimen_barcode))
        else {
            continue;
        };

        specimen_metrics
            .renamed_metrics
            .insert(renamed_metric, metric_value.clone());

        let lib_type_key_metrics = key_metrics
            .entry(library_type)
            .or_insert_with(|| Vec::with_capacity(16));

        lib_type_key_metrics.push(specimen_metrics)
    }

    key_metrics
}

#[derive(Debug, Clone, postgres_types::FromSql)]
#[postgres(name = "chromium_dataset_raw_file")]
struct RawFileRecord {
    dataset_id: Id,
    path: NonemptyString,
    sha256_hash: Vec<u8>,
}

fn create_file_tree(
    raw_files: &[RawFileRecord],
    parsed_files: Vec<ChromiumDatasetParsedFileRecord>,
) -> Vec<FileTreeNode<ChromiumDatasetParsedFileData>> {
    let mut parsed_files: HashMap<_, _> = parsed_files
        .into_iter()
        .map(|f| (f.path, f.data.0))
        .collect();

    let mut root = Directory::default();

    for RawFileRecord {
        dataset_id,
        path,
        sha256_hash,
    } in raw_files
    {
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
                raw_file_link: file_path_to_link(*dataset_id, path),
                parsed_contents: parsed_files.remove(path),
                sha256_hash: sha256_hash.clone(),
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
    raw_files: &[RawFileRecord],
) -> ChromiumDatasetDetailedLinks {
    ChromiumDatasetDetailedLinks {
        simple: chromium_dataset_links(id),
        file_dir: dataset_id_to_file_dir(id),
        raw_files: raw_files
            .iter()
            .map(|p| file_path_to_link(id, &p.path))
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
    use std::collections::HashMap;

    use cellnoor_types::{
        chromium_dataset::{ChromiumDatasetParsedFileData, ChromiumDatasetParsedFileRecord},
        file_tree::{FileTreeNode, FileTreeNodeKind},
        id::Id,
    };
    use csvranger::TenxCsvValue;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::{PSEUDO_BARCODE, create_file_tree};
    use crate::{
        handlers::chromium_datasets::{
            index_detailed::RawFileRecord, upload_files::write_to_db::parse_csv,
        },
        state::dev_util::ToNonemptyString,
    };

    const SINGLEPLEX_QC_SAMPLE_METRICS: &[u8] = include_bytes!(
        "upload_files/test-data/cellranger_multi/gex/\
         SOD1_G93A_mouse_spinal_cord_P112_specimen_1_Multiplex_qc_sample_metrics.csv"
    );

    const FLEX_METRICS_SUMMARY: &[u8] = include_bytes!(
        "upload_files/test-data/cellranger_multi/gex/per_sample_outs/sample/\
         4plex_rat_liver_kidney_lgintest_testis_CF_RatLiver_A-A03_metrics_summary.csv"
    );

    fn parsed_file(path: &str, data: &[u8]) -> ChromiumDatasetParsedFileRecord {
        ChromiumDatasetParsedFileRecord {
            dataset_id: Uuid::nil(),
            path: path.to_nonempty_string(),
            data: Json(parse_csv(data).unwrap()),
        }
    }

    #[test]
    fn file_tree() {
        let dataset_id = Id { id: Uuid::nil() };

        let parsed_file_data = ChromiumDatasetParsedFileData::StandardCsv {
            columns: vec!["field".to_owned()],
            data: vec![
                [("field".to_owned(), TenxCsvValue::I64(0))]
                    .into_iter()
                    .collect(),
            ],
        };

        let file = |full_path: &str, parsed_contents| FileTreeNode {
            path: full_path.to_owned(),
            name: full_path.split('/').next_back().map(str::to_owned).unwrap(),
            kind: FileTreeNodeKind::File {
                raw_file_link: format!("/files/chromium-datasets/{dataset_id}/{full_path}"),
                parsed_contents,
                sha256_hash: vec![],
            },
        };
        let dir = |full_path: &str, children| FileTreeNode {
            path: full_path.to_owned(),
            name: full_path
                .rsplit_once('/')
                .map_or(full_path, |(_, p)| p)
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
        .map(|p| RawFileRecord {
            dataset_id,
            path: p.to_nonempty_string(),
            sha256_hash: vec![],
        });

        assert_eq!(
            create_file_tree(
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
