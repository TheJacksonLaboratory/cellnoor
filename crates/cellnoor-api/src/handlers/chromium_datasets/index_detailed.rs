use std::collections::{BTreeMap, HashMap};

use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetDetailedLinks, ChromiumDatasetParsedFileData,
        ChromiumDatasetParsedFileRecord, ChromiumDatasetQuery, ChromiumDatasetSpecimen,
        SavedChromiumDatasetRecord,
    },
    file_tree::{FileTreeNode, FileTreeNodeKind},
    id::Id,
    library::SavedLibraryRecord,
    nonempty::NonemptyString,
    suspension_pool::{SavedTaggedSpecimenRecord, TaggedSpecimen},
};
use csvranger::TenxCsvValue;
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
    let chromium_run = row.get("chromium_run");

    let mut raw_files: Vec<_> = row.get("raw_files");
    raw_files.sort_by(|p1: &RawFileRecord, p2| p1.path.cmp(&p2.path));

    let parsed_files: Vec<_> = row.get("parsed_files");
    let mut metrics_by_specimen = extract_key_metrics_from_parsed_files(&parsed_files);

    ChromiumDatasetDetailed {
        links: chromium_dataset_detailed_links(record.id, &raw_files),
        record,
        assay: row.get("assay"),
        libraries: libraries.into_iter().map(library_from_record).collect(),
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .map(|sp| chromium_dataset_specimen_from_tagged_specimen(sp, &mut metrics_by_specimen))
            .collect(),
        chromium_run: chromium_run_from_record(chromium_run),
        files: create_file_tree(&raw_files, parsed_files),
    }
}

static PSEUDO_BARCODE: &str = "specimen";

fn chromium_dataset_specimen_from_tagged_specimen(
    specimen: TaggedSpecimen,
    metrics_by_specimen: &mut HashMap<&str, HashMap<&'static str, TenxCsvValue>>,
) -> ChromiumDatasetSpecimen {
    let Some(tag) = specimen
        .multiplexing_tag
        .as_ref()
        .map(|m| &m.tag_id)
        .map(NonemptyString::as_ref)
        .or(specimen.ocm_barcode_id.as_ref().map(|b| b.as_ref()))
    else {
        return ChromiumDatasetSpecimen {
            specimen,
            key_metrics: metrics_by_specimen
                .remove(PSEUDO_BARCODE)
                .unwrap_or_default(),
        };
    };

    let key_metrics = metrics_by_specimen.remove(tag).unwrap_or_default();

    ChromiumDatasetSpecimen {
        specimen,
        key_metrics,
    }
}

fn extract_key_metrics_from_parsed_files(
    parsed_files: &[ChromiumDatasetParsedFileRecord],
) -> HashMap<&str, HashMap<&'static str, TenxCsvValue>> {
    // We're not typically going to have more than 32 specimens at a time
    let mut specimen_metrics = HashMap::with_capacity(32);

    for p in parsed_files {
        match &p.data.0 {
            ChromiumDatasetParsedFileData::SingleRowCsv { data } => {
                insert_key_metrics_from_multi_row_csv(&mut specimen_metrics, [data]);
            }
            ChromiumDatasetParsedFileData::MultiRowCsv { data } => {
                insert_key_metrics_from_multi_row_csv(&mut specimen_metrics, data);
            }
            // Just skip this for now
            ChromiumDatasetParsedFileData::Json { .. } => {
                continue;
            }
        }
    }

    specimen_metrics
}

static METRICS_RENAMING: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // These keys come from cellranger multi's per_sample_outs/*/metrics_summary.csv
    "Cells per OCM barcode" => "Estimated number of cells",
    "Cells per probe barcode" => "Estimated number of cells",

    // These come from qc_sample_metrics.csv in a gene expression or multiome assay
    "GEX: Median genes per cell" => "Median genes per cell",
    "GEX: Median UMI counts per cell" => "Median UMI counts per cell",
    "GEX: Confidently mapped to transcriptome" => "Transcriptome mapping rate",
    "GEX: Confidently mapped to exonic regions" => "Exonic region mapping rate",
    "GEX: Confidently mapped to intronic regions" => "Intronic region mapping rate",
    "GEX: Confidently mapped to intergenic regions" => "Intergenic region mapping rate",
    "GEX: Confidently mapped reads in cells" => "Fraction of transcriptomic reads in cells",
    "GEX: Number of reads in cells" => "Number of reads in cells"
};

fn insert_key_metrics_from_multi_row_csv<'a>(
    renamed_specimen_metrics: &mut HashMap<&'a str, HashMap<&'static str, TenxCsvValue>>,
    parsed_file: impl IntoIterator<Item = &'a HashMap<String, TenxCsvValue>>,
) {
    for csv_row in parsed_file {
        // Depending on the type of file we're parsing, the barcode will be
        // under one of these two keys
        let barcode = csv_row
            .get("Sample barcodes")
            .or_else(|| csv_row.get("Group Name"))
            .and_then(|bc| match bc {
                TenxCsvValue::String(bc) if !bc.is_empty() => Some(bc.as_str()),
                _ => None,
            })
            .unwrap_or("specimen");

        let specimen_metrics = renamed_specimen_metrics
            .entry(barcode)
            .or_insert_with(|| HashMap::with_capacity(32));

        specimen_metrics.extend(extract_renamed_metrics_from_qc_sample_metrics_row(csv_row));
        if let Some((metric_name, metric_value)) =
            extract_renamed_metric_from_multi_metrics_row(csv_row)
        {
            specimen_metrics.insert(metric_name, metric_value);
        }
    }
}

fn extract_renamed_metrics_from_qc_sample_metrics_row(
    csv_row: &HashMap<String, TenxCsvValue>,
) -> Vec<(&'static str, TenxCsvValue)> {
    let mut extracted_metrics = Vec::with_capacity(32);

    // Without this key, we can't even assign the row to a specimen
    if csv_row.get("Sample barcodes").is_none() {
        return extracted_metrics;
    };

    for (metric_name, renamed_metric) in METRICS_RENAMING.into_iter() {
        let Some(metric_value) = csv_row.get(*metric_name) else {
            continue;
        };

        extracted_metrics.push((*renamed_metric, metric_value.to_owned()));
    }

    extracted_metrics
}

fn extract_renamed_metric_from_multi_metrics_row(
    csv_row: &HashMap<String, TenxCsvValue>,
) -> Option<(&'static str, TenxCsvValue)> {
    let TenxCsvValue::String(metric_name) = csv_row.get("Metric Name")? else {
        return None;
    };

    let metric_name = METRICS_RENAMING.get(metric_name)?;

    let metric_value = csv_row.get("Metric Value")?;

    Some((*metric_name, metric_value.to_owned()))
}

#[derive(Debug, Clone, postgres_types::FromSql)]
#[postgres(name = "chromium_dataset_raw_file")]
struct RawFileRecord {
    dataset_id: Id,
    path: NonemptyString,
    rapidhash_v3_hash: i64,
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
        rapidhash_v3_hash,
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
                rapidhash_v3_hash: *rapidhash_v3_hash,
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

    use super::{PSEUDO_BARCODE, create_file_tree, extract_key_metrics_from_parsed_files};
    use crate::{
        handlers::chromium_datasets::{
            index_detailed::RawFileRecord, upload_files::write_to_db::parse_csv,
        },
        state::dev_util::ToNonemptyString,
    };

    const SINGLEPLEX_QC_SAMPLE_METRICS: &[u8] = include_bytes!(
        "upload_files/test-data/\
         SOD1_G93A_mouse_spinal_cord_P112_specimen_1_Multiplex_qc_sample_metrics.csv"
    );

    const FLEX_METRICS_SUMMARY: &[u8] = include_bytes!(
        "upload_files/test-data/\
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
                rapidhash_v3_hash: 0,
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
        .map(|p| RawFileRecord {
            dataset_id,
            path: p.to_nonempty_string(),
            rapidhash_v3_hash: 0,
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

    #[test]
    fn singleplex_key_metrics() {
        let qc_sample_metrics = parsed_file("qc_sample_metrics.csv", SINGLEPLEX_QC_SAMPLE_METRICS);
        // The rows of this file with an empty "Group Name" fall under the same
        // key as qc_sample_metrics.csv
        let metrics_summary = parsed_file(
            "per_sample_outs/sample1/metrics_summary.csv",
            FLEX_METRICS_SUMMARY,
        );

        let expected = HashMap::from([
            ("Median genes per cell", TenxCsvValue::I64(1305)),
            ("Median UMI counts per cell", TenxCsvValue::I64(2244)),
            (
                "Fraction of transcriptomic reads in cells",
                TenxCsvValue::F64(0.6847850295990157),
            ),
        ]);

        // The database returns parsed files in no particular order
        for parsed_files in [
            [qc_sample_metrics.clone(), metrics_summary.clone()],
            [metrics_summary, qc_sample_metrics],
        ] {
            assert_eq!(
                extract_key_metrics_from_parsed_files(&parsed_files)[PSEUDO_BARCODE],
                expected
            );
        }
    }

    #[test]
    fn flex_key_metrics() {
        let parsed_files = [parsed_file(
            "per_sample_outs/sample1/metrics_summary.csv",
            FLEX_METRICS_SUMMARY,
        )];
        let metrics = extract_key_metrics_from_parsed_files(&parsed_files);

        for (probe_barcode, n_cells) in [
            ("A-A03", 7942),
            ("A-B03", 5980),
            ("A-C03", 9271),
            ("A-D03", 10318),
        ] {
            assert_eq!(
                metrics[probe_barcode],
                HashMap::from([("Estimated number of cells", TenxCsvValue::I64(n_cells))])
            );
        }
    }
}
