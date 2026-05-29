select
    chromium_dataset,
    array_agg(distinct (
        specimen, multiplexing_tag, ocm_barcode_id
    )::tagged_specimen) as specimens,
    array_agg(distinct library) as libraries,
    array(
        select path from chromium_dataset_raw_file
        where dataset_id = (chromium_dataset).id
    ) as raw_file_paths,
    array(
        select chromium_dataset_parsed_file from chromium_dataset_parsed_file
        where dataset_id = (chromium_dataset).id
    ) as data
from chromium_dataset_to_specimen
/* {where} */
group by chromium_dataset
