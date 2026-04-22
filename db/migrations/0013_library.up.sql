create table library (
    id uuid primary key default uuidv7(),
    links simple_links generated always as (row('/libraries/' || id)) stored not null,
    readable_id case_insensitive_text unique not null,
    cdna_id uuid references cdna not null,
    single_index_set_name case_insensitive_text references single_index_set,
    dual_index_set_name case_insensitive_text references dual_index_set,
    number_of_sample_index_pcr_cycles integer not null,
    target_reads_per_cell bigint,
    prepared_at timestamptz not null,
    additional_data jsonb,
    constraint has_index check ((single_index_set_name is null) != (dual_index_set_name is null))
);

create table library_measurement (
    id uuid primary key default uuidv7(),
    library_id uuid references library on delete cascade not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (library_id, measured_by, measured_at, data)
);

create table library_preparer (
    library_id uuid references library on delete cascade not null,
    prepared_by uuid references person not null,
    primary key (library_id, prepared_by)
);
