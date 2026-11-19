use cellnoor_types::chromium_dataset::TransposedCsvRow;
use phf::phf_ordered_map;

use super::renamed_metrics::*;
use crate::handlers::chromium_datasets::index_detailed::MetricsExtractedCsvRow;

pub(super) fn extract_key_metrics_from_transposed_csv<'a>(
    csv: &'a [TransposedCsvRow],
) -> impl Iterator<Item = MetricsExtractedCsvRow<'a>> {
    csv.into_iter().flat_map(extract_info_from_row)
}

fn extract_info_from_row<'a>(
    TransposedCsvRow {
        library_type,
        group_name: specimen_barcode,
        metric_name: old_name,
        metric_value,
        ..
    }: &'a TransposedCsvRow,
) -> Option<MetricsExtractedCsvRow<'a>> {
    let renamed_metric =
        CELLRANGER_MULTI_METRICS_SUMMARY_RENAMING.get(&(library_type.as_ref(), old_name))?;

    Some(MetricsExtractedCsvRow {
        library_type: *library_type,
        specimen_barcode,
        renamed_metric,
        metric_value,
    })
}

static CELLRANGER_MULTI_METRICS_SUMMARY_RENAMING: phf::OrderedMap<
    (&'static str, &'static str),
    &'static str,
> = phf_ordered_map! {
    // We have to handwrite the library types because phf can't evaluate enum variants I guess
    ("Gene Expression", "Cells per OCM barcode") =>  ESTIMATED_NUMBER_OF_CELLS,
    ("Gene Expression", "Cells per probe barcode") =>  ESTIMATED_NUMBER_OF_CELLS,

    ("Antibody Capture", "Median UMI counts per cell") => AB_MEDIAN_UMI_COUNTS_PER_CELL,


    ("Multiplexing Capture", "Fraction reads in cell-associated barcodes") => CMO_FRACTION_OF_READS_IN_CELLS,

    ("VDJ B", "Cells with productive V-J spanning pair") => VDJ_B_FRACTION_OF_CELLS_WITH_PRODUCTIVE_VJ_SPANNING_PAIR,

    ("VDJ B", "Fraction reads in cells") => VDJ_B_FRACTION_OF_READS_IN_CELLS,
    ("VDJ B", "Reads mapped to any V(D)J gene") => VDJ_B_VDJ_GENE_MAPPING_RATE,
    ("VDJ B", "VDJ cells") => VDJ_B_NUMBER_OF_CELLS,

    ("VDJ T", "Cells with productive V-J spanning pair") => VDJ_T_FRACTION_OF_CELLS_WITH_PRODUCTIVE_VJ_SPANNING_PAIR,
    ("VDJ T", "Fraction reads in cells") => VDJ_T_FRACTION_OF_READS_IN_CELLS,
    ("VDJ T", "Reads mapped to any V(D)J gene") => VDJ_T_VDJ_GENE_MAPPING_RATE,
    ("VDJ T", "VDJ cells") => VDJ_T_NUMBER_OF_CELLS,
};

#[cfg(test)]
mod tests {}
