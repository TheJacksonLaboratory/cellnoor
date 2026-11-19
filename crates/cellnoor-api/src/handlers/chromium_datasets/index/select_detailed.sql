select
    chromium_dataset,
    reference_package,
    tenx_assay as assay,
    chromium_run,
    array_agg(distinct(
        specimen, multiplexing_tag, ocm_barcode_id
    )::tagged_specimen) as specimens,
    array_agg(distinct cdna) as cdna,
    array_agg(distinct library) as libraries,
    -- TODO: sort these by path
    array(
        select chromium_dataset_raw_file from chromium_dataset_raw_file
        where dataset_id = (chromium_dataset).id
    ) as raw_files,
    -- TODO: sort these by path
    array(
        select chromium_dataset_parsed_file from chromium_dataset_parsed_file
        where dataset_id = (chromium_dataset).id
    ) as parsed_files
from chromium_dataset_to_specimen
/* {where} */
group by chromium_dataset, reference_package, assay, chromium_run
