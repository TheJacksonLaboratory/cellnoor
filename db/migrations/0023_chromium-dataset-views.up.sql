create view chromium_dataset_to_specimen as (
    select
        chromium_dataset,
        library.specimen,
        library
    from chromium_dataset
    join chromium_dataset_library as cds_lib on chromium_dataset.id = cds_lib.dataset_id
    join library_to_specimen as library on cds_lib.library_id = (library.library).id
);
