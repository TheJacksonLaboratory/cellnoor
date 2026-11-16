use std::collections::{BTreeMap, HashMap};

use axum::{Json, extract::State};
use cellnoor_types::{
    cdna::creation::LibraryType,
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetDetailedLinks, ChromiumDatasetParsedFileData,
        ChromiumDatasetParsedFileRecord, ChromiumDatasetQuery, ChromiumDatasetSpecimen,
        SavedChromiumDatasetRecord, TransposedCsvRow,
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
        .iter()
        .map(map_detailed_row)
        .collect())
}

fn map_detailed_row(row: &Row) -> ChromiumDatasetDetailed {
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

type KeyMetrics = HashMap<&'static str, TenxCsvValue>;

fn chromium_dataset_specimen_from_tagged_specimen(
    specimen: TaggedSpecimen,
    metrics_by_specimen: &mut HashMap<&str, KeyMetrics>,
) -> ChromiumDatasetSpecimen {
    let Some(tag) = specimen
        .multiplexing_tag
        .as_ref()
        .map(|m| &m.tag_id)
        .map(NonemptyString::as_ref)
        .or(specimen
            .ocm_barcode_id
            .as_ref()
            .map(std::convert::AsRef::as_ref))
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
) -> HashMap<&str, KeyMetrics> {
    // We're not typically going to have more than 32 specimens at a time
    let mut specimen_metrics = HashMap::with_capacity(32);

    for p in parsed_files {
        match &p.data.0 {
            ChromiumDatasetParsedFileData::StandardCsv { data } => {
                insert_key_metrics_from_standard_csv(&mut specimen_metrics, data);
            }
            ChromiumDatasetParsedFileData::TransposedCsv { data } => {
                insert_key_metrics_from_transposed_csv(&mut specimen_metrics, data);
            }
            // We can actually get all the metrics of interest from CSVs, no
            // JSON required
            ChromiumDatasetParsedFileData::Json { .. } => {}
        }
    }

    specimen_metrics
}

const ESTIMATED_NUMBER_OF_CELLS: &str = "Estimated number of cells";

const GEX_MEAN_READS_PER_CELL: &str = "GEX mean reads per cell";
const GEX_MEDIAN_GENES_PER_CELL: &str = "GEX median genes per cell";
const GEX_SEQUENCED_READ_PAIRS: &str = "GEX number of sequenced read pairs";
const GEX_SEQUENCING_SATURATION: &str = "GEX sequencing saturation";
const GEX_INTERGENIC_REGION_MAPPING_RATE: &str = "GEX intergenic region mapping rate";
const GEX_INTRONIC_REGION_MAPPING_RATE: &str = "GEX intronic region mapping rate";
const GEX_EXONIC_REGION_MAPPING_RATE: &str = "GEX exonic region mapping rate";
const GEX_TRANSCRIPTOME_MAPPING_RATE: &str = "GEX transcriptome mapping rate";
const GEX_GENOME_MAPPING_RATE: &str = "GEX genome mapping rate";
const GEX_FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS: &str =
    "GEX fraction of transcriptomic reads in cells";
const GEX_MEDIAN_UMI_COUNTS_PER_CELL: &str = "GEX median UMI counts per cell";
const GEX_NUMBER_OF_READS_IN_CELLS: &str = "GEX number of reads in cells";

const ATAC_MEAN_READS_PER_CELL: &str = "ATAC mean reads per cell";
const ATAC_SEQUENCED_READ_PAIRS: &str = "ATAC number of sequenced read pairs";
const ATAC_GENOME_MAPPING_RATE: &str = "ATAC genome mapping rate";
const ATAC_MEDIAN_HIGH_QUALITY_FRAGMENTS_PER_CELL: &str =
    "ATAC median high-quality fragments per cell";
const ATAC_TSS_ENRICHMENT_SCORE: &str = "ATAC TSS enrichment score";
const ATAC_FRACTION_OF_TRANSPOSITION_EVENTS_IN_PEAKS_IN_CELLS: &str =
    "ATAC fraction of transposition events in peaks in cells";
const ATAC_FRACTION_OF_HIGH_QUALITY_FRAGMENTS_IN_CELLS: &str =
    "ATAC fraction of high-quality fragments in cells";
const ATAC_FRACTION_OF_GENOME_IN_PEAKS: &str = "ATAC fraction of genome in peaks";
const ATAC_PERCENT_DUPLICATES: &str = "ATAC percent duplicates";

const VDJ_B_FRACTION_OF_CELLS_WITH_PRODUCTIVE_VJ_SPANNING_PAIR: &str =
    "VDJ B fraction of cells with productive V-J spanning pair";
const VDJ_B_NUMBER_OF_CELLS: &str = "VDJ B number of cells";
const VDJ_B_FRACTION_OF_READS_IN_CELLS: &str = "VDJ B fraction of reads in cells";
const VDJ_B_VDJ_GENE_MAPPING_RATE: &str = "VDJ B V(D)J gene mapping rate";

const VDJ_T_FRACTION_OF_CELLS_WITH_PRODUCTIVE_VJ_SPANNING_PAIR: &str =
    "VDJ T fraction of cells with productive V-J spanning pair";
const VDJ_T_NUMBER_OF_CELLS: &str = "VDJ T number of cells";
const VDJ_T_FRACTION_OF_READS_IN_CELLS: &str = "VDJ T fraction of reads in cells";
const VDJ_T_VDJ_GENE_MAPPING_RATE: &str = "VDJ T V(D)J gene mapping rate";

const AB_MEDIAN_UMI_COUNTS_PER_CELL: &str = "AB median UMI counts per cell";

const CMO_FRACTION_OF_READS_IN_CELLS: &str = "CMO fraction of reads in cells";

static METRICS_RENAMING: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // cellranger count (metrics_summary.csv)
    "Estimated Number of Cells" => ESTIMATED_NUMBER_OF_CELLS,
    "Mean Reads per Cell" => GEX_MEAN_READS_PER_CELL,
    "Median Genes per Cell" => GEX_MEDIAN_GENES_PER_CELL,
    // "Number of Reads" counts read pairs, the same as cellranger-arc's "GEX Sequenced read pairs":
    // https://www.10xgenomics.com/support/software/cell-ranger/latest/analysis/outputs/cr-3p-outputs-metrics-count
    // https://www.10xgenomics.com/support/software/cell-ranger-arc/latest/analysis/outputs/metrics
    "Number of Reads" => GEX_SEQUENCED_READ_PAIRS,
    "Sequencing Saturation" => GEX_SEQUENCING_SATURATION,
    "Reads Mapped Confidently to Genome" => GEX_GENOME_MAPPING_RATE,
    "Reads Mapped Confidently to Intergenic Regions" => GEX_INTERGENIC_REGION_MAPPING_RATE,
    "Reads Mapped Confidently to Intronic Regions" => GEX_INTRONIC_REGION_MAPPING_RATE,
    "Reads Mapped Confidently to Exonic Regions" => GEX_EXONIC_REGION_MAPPING_RATE,
    "Reads Mapped Confidently to Transcriptome" => GEX_TRANSCRIPTOME_MAPPING_RATE,
    "Fraction Reads in Cells" => GEX_FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS,
    "Median UMI Counts per Cell" => GEX_MEDIAN_UMI_COUNTS_PER_CELL,

    // cellranger multi (per_sample_outs/*/metrics_summary.csv)
    "Cells per OCM barcode" => ESTIMATED_NUMBER_OF_CELLS,
    "Cells per probe barcode" => ESTIMATED_NUMBER_OF_CELLS,

    // cellranger multi (qc_sample_metrics.csv)
    "GEX: Cells" => ESTIMATED_NUMBER_OF_CELLS,
    "GEX: Median genes per cell" => GEX_MEDIAN_GENES_PER_CELL,
    "GEX: Median UMI counts per cell" => GEX_MEDIAN_UMI_COUNTS_PER_CELL,
    "GEX: Confidently mapped to genome" => GEX_GENOME_MAPPING_RATE,
    "GEX: Confidently mapped to transcriptome" => GEX_TRANSCRIPTOME_MAPPING_RATE,
    "GEX: Confidently mapped to exonic regions" => GEX_EXONIC_REGION_MAPPING_RATE,
    "GEX: Confidently mapped to intronic regions" => GEX_INTRONIC_REGION_MAPPING_RATE,
    "GEX: Confidently mapped to intergenic regions" => GEX_INTERGENIC_REGION_MAPPING_RATE,
    "GEX: Confidently mapped reads in cells" => GEX_FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS,
    "GEX: Number of reads in cells" => GEX_NUMBER_OF_READS_IN_CELLS,

    // cellranger-atac count (summary.csv)
    "Estimated number of cells" => ESTIMATED_NUMBER_OF_CELLS,
    "Confidently mapped read pairs" => ATAC_GENOME_MAPPING_RATE,
    "Fraction of genome in peaks" => ATAC_FRACTION_OF_GENOME_IN_PEAKS,
    "Fraction of high-quality fragments in cells" => ATAC_FRACTION_OF_HIGH_QUALITY_FRAGMENTS_IN_CELLS,
    "Mean raw read pairs per cell" => ATAC_MEAN_READS_PER_CELL,
    "Median high-quality fragments per cell" => ATAC_MEDIAN_HIGH_QUALITY_FRAGMENTS_PER_CELL,
    "Fraction of transposition events in peaks in cells" => ATAC_FRACTION_OF_TRANSPOSITION_EVENTS_IN_PEAKS_IN_CELLS,
    "Percent duplicates" => ATAC_PERCENT_DUPLICATES,
    "Sequenced read pairs" => ATAC_SEQUENCED_READ_PAIRS,
    "TSS enrichment score" => ATAC_TSS_ENRICHMENT_SCORE,

    // cellranger-arc count (summary.csv)
    // Note that cellranger-arc count outputs "Estimated number of cells", but that's caught above
    "ATAC Confidently mapped read pairs" => ATAC_GENOME_MAPPING_RATE,
    "ATAC Fraction of genome in peaks" => ATAC_FRACTION_OF_GENOME_IN_PEAKS,
    "ATAC Fraction of high-quality fragments in cells" => ATAC_FRACTION_OF_HIGH_QUALITY_FRAGMENTS_IN_CELLS,
    "ATAC Fraction of transposition events in peaks in cells" => ATAC_FRACTION_OF_TRANSPOSITION_EVENTS_IN_PEAKS_IN_CELLS,
    "ATAC Mean raw read pairs per cell" => ATAC_MEAN_READS_PER_CELL,
    "ATAC Median high-quality fragments per cell" => ATAC_MEDIAN_HIGH_QUALITY_FRAGMENTS_PER_CELL,
    "ATAC Percent duplicates" => ATAC_PERCENT_DUPLICATES,
    "ATAC Sequenced read pairs" => ATAC_SEQUENCED_READ_PAIRS,
    "ATAC TSS enrichment score" => ATAC_TSS_ENRICHMENT_SCORE,
    "GEX Fraction of transcriptomic reads in cells" => GEX_FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS,
    "GEX Mean raw reads per cell" => GEX_MEAN_READS_PER_CELL,
    "GEX Median UMI counts per cell" => GEX_MEDIAN_UMI_COUNTS_PER_CELL,
    "GEX Median genes per cell" => GEX_MEDIAN_GENES_PER_CELL,
    // TODO: verify this claude-slop is correct
    // cellranger-arc defines GEX "Percent duplicates" as the fraction of reads originating from an
    // already-observed UMI, which is how cellranger count defines "Sequencing Saturation":
    // https://www.10xgenomics.com/support/software/cell-ranger-arc/latest/analysis/outputs/metrics
    // https://www.10xgenomics.com/support/software/cell-ranger/latest/analysis/outputs/cr-3p-outputs-metrics-count
    "GEX Percent duplicates" => GEX_SEQUENCING_SATURATION,
    "GEX Reads mapped confidently to exonic regions" => GEX_EXONIC_REGION_MAPPING_RATE,
    "GEX Reads mapped confidently to genome" => GEX_GENOME_MAPPING_RATE,
    "GEX Reads mapped confidently to intergenic regions" => GEX_INTERGENIC_REGION_MAPPING_RATE,
    "GEX Reads mapped confidently to intronic regions" => GEX_INTRONIC_REGION_MAPPING_RATE,
    "GEX Reads mapped confidently to transcriptome" => GEX_TRANSCRIPTOME_MAPPING_RATE,
    "GEX Sequenced read pairs" => GEX_SEQUENCED_READ_PAIRS
};

// cellranger multi (per_sample_outs/*/metrics_summary.csv)
fn rename_multi_metric(library_type: LibraryType, metric_name: &str) -> Option<&'static str> {
    use LibraryType::{AntibodyCapture, GeneExpression, MultiplexingCapture, VdjB, VdjT};

    match (library_type, metric_name) {
        (AntibodyCapture, "Median UMI counts per cell") => Some(AB_MEDIAN_UMI_COUNTS_PER_CELL),

        (GeneExpression, _) => METRICS_RENAMING.get(metric_name).copied(),

        (MultiplexingCapture, "Fraction reads in cell-associated barcodes") => {
            Some(CMO_FRACTION_OF_READS_IN_CELLS)
        }

        (VdjB, "Cells with productive V-J spanning pair") => {
            Some(VDJ_B_FRACTION_OF_CELLS_WITH_PRODUCTIVE_VJ_SPANNING_PAIR)
        }
        (VdjB, "Fraction reads in cells") => Some(VDJ_B_FRACTION_OF_READS_IN_CELLS),
        (VdjB, "Reads mapped to any V(D)J gene") => Some(VDJ_B_VDJ_GENE_MAPPING_RATE),
        (VdjB, "VDJ cells") => Some(VDJ_B_NUMBER_OF_CELLS),

        (VdjT, "Cells with productive V-J spanning pair") => {
            Some(VDJ_T_FRACTION_OF_CELLS_WITH_PRODUCTIVE_VJ_SPANNING_PAIR)
        }
        (VdjT, "Fraction reads in cells") => Some(VDJ_T_FRACTION_OF_READS_IN_CELLS),
        (VdjT, "Reads mapped to any V(D)J gene") => Some(VDJ_T_VDJ_GENE_MAPPING_RATE),
        (VdjT, "VDJ cells") => Some(VDJ_T_NUMBER_OF_CELLS),

        _ => None,
    }
}

fn insert_key_metrics_from_standard_csv<'a>(
    renamed_specimen_metrics: &mut HashMap<&'a str, KeyMetrics>,
    parsed_file: &'a [HashMap<String, TenxCsvValue>],
) {
    for csv_row in parsed_file {
        let barcode = match csv_row.get("Sample barcodes") {
            Some(TenxCsvValue::String(bc)) => bc,
            _ => "",
        };

        renamed_specimen_metrics
            .entry(specimen_key(barcode))
            .or_default()
            .extend(extract_renamed_metrics_from_standard_row(csv_row));
    }
}

fn insert_key_metrics_from_transposed_csv<'a>(
    renamed_specimen_metrics: &mut HashMap<&'a str, KeyMetrics>,
    parsed_file: &'a [TransposedCsvRow],
) {
    for csv_row in parsed_file {
        if let Some((metric_name, metric_value)) =
            extract_renamed_metric_from_transposed_row(csv_row)
        {
            renamed_specimen_metrics
                .entry(specimen_key(&csv_row.group_name))
                .or_default()
                .insert(metric_name, metric_value);
        }
    }
}

fn specimen_key(barcode: &str) -> &str {
    // "---" is a special little stupid terminology that cellranger multi uses
    if barcode.is_empty() || barcode == "---" {
        PSEUDO_BARCODE
    } else {
        barcode
    }
}

fn extract_renamed_metrics_from_standard_row(
    csv_row: &HashMap<String, TenxCsvValue>,
) -> impl Iterator<Item = (&'static str, TenxCsvValue)> + '_ {
    METRICS_RENAMING
        .entries()
        .filter_map(|(metric_name, renamed_metric)| {
            Some((*renamed_metric, csv_row.get(*metric_name)?.clone()))
        })
}

fn extract_renamed_metric_from_transposed_row(
    csv_row: &TransposedCsvRow,
) -> Option<(&'static str, TenxCsvValue)> {
    let metric_name = rename_multi_metric(csv_row.library_type, &csv_row.metric_name)?;

    Some((metric_name, csv_row.metric_value.clone()))
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

    use super::{PSEUDO_BARCODE, create_file_tree, extract_key_metrics_from_parsed_files};
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
            ("Estimated number of cells", TenxCsvValue::I64(16410)),
            ("GEX median genes per cell", TenxCsvValue::I64(1305)),
            ("GEX median UMI counts per cell", TenxCsvValue::I64(2244)),
            (
                "GEX fraction of transcriptomic reads in cells",
                TenxCsvValue::F64(0.684_785_029_599_015_7),
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
