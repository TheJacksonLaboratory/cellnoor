select exists(
    select 1 from chromium_dataset_raw_file
    where dataset_id = $1 and path = $2
);
