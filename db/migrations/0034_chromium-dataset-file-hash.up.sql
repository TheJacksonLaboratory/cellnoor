alter table chromium_dataset_raw_file add column rapidhash_v3_hash bigint not null;

alter table chromium_dataset_raw_file add unique (dataset_id, rapidhash_v3_hash);
