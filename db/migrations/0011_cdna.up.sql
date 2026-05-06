create table cdna (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    library_type case_insensitive_text not null,
    prepared_at timestamptz not null,
    gem_pool_id uuid references gem_pool,
    n_amplification_cycles integer not null,
    additional_data jsonb,

    -- a single GEM pool cannot generate more than one cDNA of the same library type
    unique (gem_pool_id, library_type)
);

create table cdna_measurement (
    id uuid primary key default uuidv7(),
    cdna_id uuid references cdna on delete cascade not null,
    measured_by uuid references person not null,
    measured_at timestamptz not null,
    data jsonb not null,

    unique (cdna_id, measured_by, measured_at, data)
);

create trigger measurement_made_after_cdna_prepared before insert or update on cdna_measurement for each row execute function check_timestamp_ordering(
    'measured_at', 'cdna_id', 'cdna', 'prepared_at'
);

create table cdna_preparer (
    cdna_id uuid references cdna on delete cascade not null,
    prepared_by uuid references person not null,
    primary key (cdna_id, prepared_by)
);
