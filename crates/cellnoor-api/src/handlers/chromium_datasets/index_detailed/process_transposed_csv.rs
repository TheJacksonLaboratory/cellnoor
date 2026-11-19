use cellnoor_types::{cdna::creation::LibraryType, chromium_dataset::TransposedCsvRow};

use super::renamed_metrics::*;
use crate::handlers::chromium_datasets::index_detailed::MetricsExtractedCsvRow;

pub(super) fn extract_key_metrics_from_transposed_csv(
    csv: &[TransposedCsvRow],
) -> impl Iterator<Item = MetricsExtractedCsvRow<'_>> {
    csv.iter().filter_map(extract_info_from_row)
}

fn extract_info_from_row(
    TransposedCsvRow {
        library_type,
        group_name: specimen_barcode,
        metric_name: old_name,
        metric_value,
        ..
    }: &TransposedCsvRow,
) -> Option<MetricsExtractedCsvRow<'_>> {
    rename_cellranger_multi_metric(*library_type, old_name).map(|renamed_metric| {
        MetricsExtractedCsvRow {
            library_type: *library_type,
            specimen_barcode,
            renamed_metric,
            metric_value,
        }
    })
}

fn rename_cellranger_multi_metric(
    library_type: LibraryType,
    metric_name: &str,
) -> Option<&'static str> {
    use LibraryType::{AntibodyCapture, GeneExpression, MultiplexingCapture, VdjB, VdjT};

    let renamed = match (library_type, metric_name) {
        (GeneExpression, "Cells per OCM barcode" | "Cells per probe barcode") => {
            ESTIMATED_NUMBER_OF_CELLS
        }

        (AntibodyCapture, "Median UMI counts per cell") => MEDIAN_UMI_COUNTS_PER_CELL,

        (MultiplexingCapture, "Fraction reads in cell-associated barcodes") => {
            FRACTION_OF_READS_IN_CELLS
        }

        (VdjB | VdjT, "Cells with productive V-J spanning pair") => {
            FRACTION_OF_CELLS_WITH_PRODUCTIVE_VJ_SPANNING_PAIR
        }
        (VdjB | VdjT, "Fraction reads in cells") => FRACTION_OF_READS_IN_CELLS,
        (VdjB | VdjT, "Reads mapped to any V(D)J gene") => VDJ_GENE_MAPPING_RATE,
        (VdjB | VdjT, "VDJ cells") => NUMBER_OF_CELLS,
        _ => return None,
    };

    Some(renamed)
}
