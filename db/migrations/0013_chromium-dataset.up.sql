create table chromium_dataset (
    id uuid primary key default uuidv7(),
    name case_insensitive_text not null,
    delivered_at timestamptz not null
);

-- We don't actually store the content of the files in the database, just the path, so we can do permissions checks.
-- The actual files are stored on the server and served statically by caddy :)
create table chromium_dataset_raw_file (
    dataset_id uuid references chromium_dataset on delete cascade not null,
    path case_insensitive_text not null,
    primary key (dataset_id, path)
);

-- Some files can be parsed into JSON, so we store those
create table chromium_dataset_parsed_file (
    dataset_id uuid not null,
    path case_insensitive_text not null,
    data jsonb not null,
    primary key (dataset_id, path),
    foreign key (dataset_id, path) references chromium_dataset_raw_file on delete cascade
);

create table chromium_dataset_library (
    dataset_id uuid references chromium_dataset on delete cascade not null,
    library_id uuid references library not null,
    primary key (dataset_id, library_id)
);
