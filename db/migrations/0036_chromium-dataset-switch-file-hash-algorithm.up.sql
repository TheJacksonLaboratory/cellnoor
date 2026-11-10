alter table chromium_dataset_raw_file drop rapidhash_v3_hash;

alter table chromium_dataset_raw_file add sha256_hash bytea not null;

alter table chromium_dataset_raw_file add unique (dataset_id, sha256_hash);
