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

    let library_type_specimen_metrics_map = create_library_type_specimen_metrics_map(&specimens);
    let extracted_metrics = extract_metrics_from_parsed_files(
        &parsed_files,
        record.pipeline_name.as_ref(),
        record.pipeline_command.as_ref(),
    );
    let key_metrics = build_key_metrics(library_type_specimen_metrics_map, extracted_metrics);

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
) -> IndexMap<(LibraryType, &'a str), KeySpecimenMetrics> {
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
    let mut metrics_map = IndexMap::with_capacity(LibraryType::VARIANTS.len() * specimens.len());

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
    mut library_type_specimen_metrics_map: IndexMap<(LibraryType, &'a str), KeySpecimenMetrics>,
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
        if let Some(specimen_metrics) =
            library_type_specimen_metrics_map.get_mut(&(library_type, specimen_barcode))
        {
            specimen_metrics
                .renamed_metrics
                .insert(renamed_metric, metric_value.clone());
        }
    }

    for ((library_type, _), specimen_metrics) in library_type_specimen_metrics_map {
        if specimen_metrics.renamed_metrics.is_empty() {
            continue;
        }

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
        cdna::creation::LibraryType::{self, ChromatinAccessibility, GeneExpression},
        chromium_dataset::{
            ChromiumDatasetParsedFileData, ChromiumDatasetParsedFileRecord, KeySpecimenMetrics,
        },
        file_tree::{FileTreeNode, FileTreeNodeKind},
        id::Id,
    };
    use csvranger::TenxCsvValue;
    use indexmap::IndexMap;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use strum::VariantArray;
    use uuid::Uuid;

    use super::{
        PSEUDO_BARCODE, build_key_metrics, create_file_tree, extract_metrics_from_parsed_files,
        renamed_metrics::*,
    };
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

    const ATAC_SUMMARY: &[u8] = include_bytes!(
        "upload_files/test-data/cellranger-atac_count/\
         8k_mouse_cortex_ATACv2_nextgem_Chromium_Controller_summary.csv"
    );

    const ARC_SUMMARY: &[u8] = include_bytes!(
        "upload_files/test-data/cellranger-arc_count/5k_Mouse_Kidney_MO_gemx_summary.csv"
    );

    fn key_metric_names(
        pipeline: &str,
        pipeline_command: &str,
        file: &[u8],
        barcodes: &[&'static str],
    ) -> IndexMap<LibraryType, Vec<(String, Vec<&'static str>)>> {
        let parsed_files = [parsed_file("metrics.csv", file)];

        let specimen_metrics_map = LibraryType::VARIANTS
            .iter()
            .flat_map(|lt| {
                barcodes.iter().map(|bc| {
                    let specimen_metrics = KeySpecimenMetrics {
                        specimen_id: Uuid::nil(),
                        specimen_readable_id: bc.to_nonempty_string(),
                        specimen_name: bc.to_nonempty_string(),
                        renamed_metrics: IndexMap::new(),
                    };
                    ((*lt, *bc), specimen_metrics)
                })
            })
            .collect();

        let extracted_metrics =
            extract_metrics_from_parsed_files(&parsed_files, pipeline, pipeline_command);

        build_key_metrics(specimen_metrics_map, extracted_metrics)
            .into_iter()
            .filter(|(_, specimens)| !specimens.is_empty())
            .map(|(lt, specimens)| {
                let specimens = specimens
                    .into_iter()
                    .map(|s| {
                        (
                            s.specimen_readable_id.to_string(),
                            s.renamed_metrics.keys().copied().collect(),
                        )
                    })
                    .collect();
                (lt, specimens)
            })
            .collect()
    }

    #[test]
    fn cellranger_atac_count_key_metrics() {
        let expected = IndexMap::from([(
            ChromatinAccessibility,
            vec![(
                PSEUDO_BARCODE.to_owned(),
                vec![
                    ESTIMATED_NUMBER_OF_CELLS,
                    GENOME_MAPPING_RATE,
                    FRACTION_OF_GENOME_IN_PEAKS,
                    FRACTION_OF_HIGH_QUALITY_FRAGMENTS_IN_CELLS,
                    MEAN_READS_PER_CELL,
                    MEDIAN_HIGH_QUALITY_FRAGMENTS_PER_CELL,
                    FRACTION_OF_TRANSPOSITION_EVENTS_IN_PEAKS_IN_CELLS,
                    PERCENT_DUPLICATES,
                    SEQUENCED_READ_PAIRS,
                    TSS_ENRICHMENT_SCORE,
                ],
            )],
        )]);

        assert_eq!(
            key_metric_names("cellranger-atac", "count", ATAC_SUMMARY, &[PSEUDO_BARCODE]),
            expected
        );
    }

    #[test]
    fn cellranger_arc_count_key_metrics() {
        let expected = IndexMap::from([
            (
                GeneExpression,
                vec![(
                    PSEUDO_BARCODE.to_owned(),
                    vec![
                        ESTIMATED_NUMBER_OF_CELLS,
                        FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS,
                        MEAN_READS_PER_CELL,
                        MEDIAN_UMI_COUNTS_PER_CELL,
                        MEDIAN_GENES_PER_CELL,
                        SEQUENCING_SATURATION,
                        EXONIC_REGION_MAPPING_RATE,
                        GENOME_MAPPING_RATE,
                        INTERGENIC_REGION_MAPPING_RATE,
                        INTRONIC_REGION_MAPPING_RATE,
                        TRANSCRIPTOME_MAPPING_RATE,
                        SEQUENCED_READ_PAIRS,
                    ],
                )],
            ),
            (
                ChromatinAccessibility,
                vec![(
                    PSEUDO_BARCODE.to_owned(),
                    vec![
                        GENOME_MAPPING_RATE,
                        FRACTION_OF_GENOME_IN_PEAKS,
                        FRACTION_OF_HIGH_QUALITY_FRAGMENTS_IN_CELLS,
                        FRACTION_OF_TRANSPOSITION_EVENTS_IN_PEAKS_IN_CELLS,
                        MEAN_READS_PER_CELL,
                        MEDIAN_HIGH_QUALITY_FRAGMENTS_PER_CELL,
                        PERCENT_DUPLICATES,
                        SEQUENCED_READ_PAIRS,
                        TSS_ENRICHMENT_SCORE,
                    ],
                )],
            ),
        ]);

        assert_eq!(
            key_metric_names("cellranger-arc", "count", ARC_SUMMARY, &[PSEUDO_BARCODE]),
            expected
        );
    }

    #[test]
    fn cellranger_multi_singleplex_key_metrics() {
        let expected = IndexMap::from([(
            GeneExpression,
            vec![(
                PSEUDO_BARCODE.to_owned(),
                vec![
                    ESTIMATED_NUMBER_OF_CELLS,
                    MEDIAN_GENES_PER_CELL,
                    MEDIAN_UMI_COUNTS_PER_CELL,
                    FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS,
                ],
            )],
        )]);

        assert_eq!(
            key_metric_names(
                "cellranger",
                "multi",
                SINGLEPLEX_QC_SAMPLE_METRICS,
                &[PSEUDO_BARCODE]
            ),
            expected
        );
    }

    #[test]
    fn cellranger_multi_flex_key_metrics() {
        let barcodes = ["A-A03", "A-B03", "A-C03", "A-D03"];

        let expected = IndexMap::from([(
            GeneExpression,
            barcodes
                .map(|bc| (bc.to_owned(), vec![ESTIMATED_NUMBER_OF_CELLS]))
                .to_vec(),
        )]);

        assert_eq!(
            key_metric_names("cellranger", "multi", FLEX_METRICS_SUMMARY, &barcodes),
            expected
        );
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
