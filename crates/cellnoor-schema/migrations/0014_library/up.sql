create table libraries (
    id uuid primary key default uuidv7(),
    links jsonb generated always as (
        construct_links('libraries', id, '{"measurements", "sequencing-runs", "chromium-datasets"}')
    ) stored not null,
    readable_id case_insensitive_text unique not null,
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    single_index_set_name case_insensitive_text references single_index_sets on delete restrict on update restrict,
    dual_index_set_name case_insensitive_text references dual_index_sets on delete restrict on update restrict,
    number_of_sample_index_pcr_cycles integer not null,
    target_reads_per_cell bigint not null,
    prepared_at timestamptz not null,
    additional_data jsonb,
    constraint has_index check ((single_index_set_name is null) != (dual_index_set_name is null))
);

create table library_measurements (
    id uuid primary key default uuidv7(),
    library_id uuid references libraries on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamptz not null,
    data jsonb not null
);

create table library_preparers (
    library_id uuid references libraries on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (library_id, prepared_by)
);

create table sequencing_submissions (
    library_id uuid references libraries on delete restrict on update restrict not null,
    sequencing_run_id uuid references sequencing_runs on delete restrict on update restrict not null,
    submitted_at timestamptz not null,
    primary key (library_id, sequencing_run_id)
);
