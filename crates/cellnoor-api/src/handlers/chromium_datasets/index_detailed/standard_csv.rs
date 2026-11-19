use std::collections::HashMap;

use LibraryType::*;
use cellnoor_types::cdna::creation::LibraryType;
use csvranger::TenxCsvValue;

use super::renamed_metrics::*;
use crate::handlers::chromium_datasets::index_detailed::{MetricsExtractedCsvRow, PSEUDO_BARCODE};

pub(super) fn extract_key_metrics_from_standard_csv<'a>(
    csv: &'a [HashMap<String, TenxCsvValue>],
    pipeline: &str,
    pipeline_command: &str,
) -> Option<impl Iterator<Item = MetricsExtractedCsvRow<'a>>> {
    let metric_renaming = STANDARD_CSV_METRICS_RENAMING.get(&(pipeline, pipeline_command))?;

    let processed = csv
        .into_iter()
        .flat_map(|row| extract_info_from_row(row, metric_renaming));

    Some(processed)
}

fn extract_info_from_row<'a>(
    row: &'a HashMap<String, TenxCsvValue>,
    metric_renaming: &[(&'static str, LibraryType, &'static str)],
) -> impl Iterator<Item = MetricsExtractedCsvRow<'a>> {
    let specimen_barcode = match row.get("Sample barcodes") {
        Some(TenxCsvValue::String(bc)) => bc,
        _ => PSEUDO_BARCODE,
    };

    let old_name_to_parsed_row = |(old_name, library_type, renamed_metric)| {
        row.get(old_name)
            .map(|metric_value| MetricsExtractedCsvRow {
                library_type,
                specimen_barcode,
                renamed_metric,
                metric_value,
            })
    };

    metric_renaming
        .iter()
        .copied()
        .filter_map(old_name_to_parsed_row)
}

// We could easily store this in the database, but it's a bit easier to edit and
// sort when it's in application code
static STANDARD_CSV_METRICS_RENAMING: phf::Map<
    (&'static str, &'static str),
    &'static [(&'static str, LibraryType, &'static str)],
> = phf::phf_map! {
    ("cellranger", "count") => &[
        ("Estimated Number of Cells", GeneExpression, ESTIMATED_NUMBER_OF_CELLS),
        ("Mean Reads per Cell", GeneExpression, GEX_MEAN_READS_PER_CELL),
        ("Median Genes per Cell", GeneExpression, GEX_MEDIAN_GENES_PER_CELL),
        // "Number of Reads" counts read pairs, the same as cellranger-arc's "GEX Sequenced read pairs":
        // https://www.10xgenomics.com/support/software/cell-ranger/latest/analysis/outputs/cr-3p-outputs-metrics-count
        // https://www.10xgenomics.com/support/software/cell-ranger-arc/latest/analysis/outputs/metrics
        ("Number of Reads", GeneExpression, GEX_SEQUENCED_READ_PAIRS),
        ("Sequencing Saturation", LibraryType::GeneExpression, GEX_SEQUENCING_SATURATION),
        ("Reads Mapped Confidently to Genome", LibraryType::GeneExpression, GEX_GENOME_MAPPING_RATE),
        ("Reads Mapped Confidently to Intergenic Regions", LibraryType::GeneExpression, GEX_INTERGENIC_REGION_MAPPING_RATE),
        ("Reads Mapped Confidently to Intronic Regions", LibraryType::GeneExpression, GEX_INTRONIC_REGION_MAPPING_RATE),
        ("Reads Mapped Confidently to Exonic Regions", LibraryType::GeneExpression, GEX_EXONIC_REGION_MAPPING_RATE),
        ("Reads Mapped Confidently to Transcriptome", LibraryType::GeneExpression, GEX_TRANSCRIPTOME_MAPPING_RATE),
        ("Fraction Reads in Cells", LibraryType::GeneExpression, GEX_FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS),
        ("Median UMI Counts per Cell", LibraryType::GeneExpression, GEX_MEDIAN_UMI_COUNTS_PER_CELL),
    ],

//     ("cellranger", "multi") => phf_ordered_map! {
//         // metrics_summary.csv
//         "Cells per OCM barcode" => (LibraryType::GeneExpression, ESTIMATED_NUMBER_OF_CELLS),
//         "Cells per probe barcode" => (LibraryType::GeneExpression, ESTIMATED_NUMBER_OF_CELLS),
//
//         // qc_sample_metrics.csv
//         "GEX: Cells" => (LibraryType::GeneExpression, ESTIMATED_NUMBER_OF_CELLS),
//         "GEX: Median genes per cell" => (LibraryType::GeneExpression, GEX_MEDIAN_GENES_PER_CELL),
//         "GEX: Median UMI counts per cell" => (LibraryType::GeneExpression, GEX_MEDIAN_UMI_COUNTS_PER_CELL),
//         "GEX: Confidently mapped to genome" => (LibraryType::GeneExpression, GEX_GENOME_MAPPING_RATE),
//         "GEX: Confidently mapped to transcriptome" => (LibraryType::GeneExpression, GEX_TRANSCRIPTOME_MAPPING_RATE),
//         "GEX: Confidently mapped to exonic regions" => (LibraryType::GeneExpression, GEX_EXONIC_REGION_MAPPING_RATE),
//         "GEX: Confidently mapped to intronic regions" => (LibraryType::GeneExpression, GEX_INTRONIC_REGION_MAPPING_RATE),
//         "GEX: Confidently mapped to intergenic regions" => (LibraryType::GeneExpression, GEX_INTERGENIC_REGION_MAPPING_RATE),
//         "GEX: Confidently mapped reads in cells" => (LibraryType::GeneExpression, GEX_FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS),
//         "GEX: Number of reads in cells" => (LibraryType::GeneExpression, GEX_NUMBER_OF_READS_IN_CELLS),
//     },

//     // cellranger-atac count (summary.csv)
//     "Estimated number of cells" => ESTIMATED_NUMBER_OF_CELLS,
//     "Confidently mapped read pairs" => ATAC_GENOME_MAPPING_RATE,
//     "Fraction of genome in peaks" => ATAC_FRACTION_OF_GENOME_IN_PEAKS,
//     "Fraction of high-quality fragments in cells" => ATAC_FRACTION_OF_HIGH_QUALITY_FRAGMENTS_IN_CELLS,
//     "Mean raw read pairs per cell" => ATAC_MEAN_READS_PER_CELL,
//     "Median high-quality fragments per cell" => ATAC_MEDIAN_HIGH_QUALITY_FRAGMENTS_PER_CELL,
//     "Fraction of transposition events in peaks in cells" => ATAC_FRACTION_OF_TRANSPOSITION_EVENTS_IN_PEAKS_IN_CELLS,
//     "Percent duplicates" => ATAC_PERCENT_DUPLICATES,
//     "Sequenced read pairs" => ATAC_SEQUENCED_READ_PAIRS,
//     "TSS enrichment score" => ATAC_TSS_ENRICHMENT_SCORE,
//
//     // cellranger-arc count (summary.csv)
//     // Note that cellranger-arc count outputs "Estimated number of cells", but that's caught above
//     "ATAC Confidently mapped read pairs" => ATAC_GENOME_MAPPING_RATE,
//     "ATAC Fraction of genome in peaks" => ATAC_FRACTION_OF_GENOME_IN_PEAKS,
//     "ATAC Fraction of high-quality fragments in cells" => ATAC_FRACTION_OF_HIGH_QUALITY_FRAGMENTS_IN_CELLS,
//     "ATAC Fraction of transposition events in peaks in cells" => ATAC_FRACTION_OF_TRANSPOSITION_EVENTS_IN_PEAKS_IN_CELLS,
//     "ATAC Mean raw read pairs per cell" => ATAC_MEAN_READS_PER_CELL,
//     "ATAC Median high-quality fragments per cell" => ATAC_MEDIAN_HIGH_QUALITY_FRAGMENTS_PER_CELL,
//     "ATAC Percent duplicates" => ATAC_PERCENT_DUPLICATES,
//     "ATAC Sequenced read pairs" => ATAC_SEQUENCED_READ_PAIRS,
//     "ATAC TSS enrichment score" => ATAC_TSS_ENRICHMENT_SCORE,
//     "GEX Fraction of transcriptomic reads in cells" => GEX_FRACTION_OF_TRANSCRIPTOMIC_READS_IN_CELLS,
//     "GEX Mean raw reads per cell" => GEX_MEAN_READS_PER_CELL,
//     "GEX Median UMI counts per cell" => GEX_MEDIAN_UMI_COUNTS_PER_CELL,
//     "GEX Median genes per cell" => GEX_MEDIAN_GENES_PER_CELL,
//     // TODO: verify this claude-slop is correct
//     // cellranger-arc defines GEX "Percent duplicates" as the fraction of reads originating from an
//     // already-observed UMI, which is how cellranger count defines "Sequencing Saturation":
//     // https://www.10xgenomics.com/support/software/cell-ranger-arc/latest/analysis/outputs/metrics
//     // https://www.10xgenomics.com/support/software/cell-ranger/latest/analysis/outputs/cr-3p-outputs-metrics-count
//     "GEX Percent duplicates" => GEX_SEQUENCING_SATURATION,
//     "GEX Reads mapped confidently to exonic regions" => GEX_EXONIC_REGION_MAPPING_RATE,
//     "GEX Reads mapped confidently to genome" => GEX_GENOME_MAPPING_RATE,
//     "GEX Reads mapped confidently to intergenic regions" => GEX_INTERGENIC_REGION_MAPPING_RATE,
//     "GEX Reads mapped confidently to intronic regions" => GEX_INTRONIC_REGION_MAPPING_RATE,
//     "GEX Reads mapped confidently to transcriptome" => GEX_TRANSCRIPTOME_MAPPING_RATE,
//     "GEX Sequenced read pairs" => GEX_SEQUENCED_READ_PAIRS
};
