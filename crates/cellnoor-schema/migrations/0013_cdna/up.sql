create table cdna (
    id uuid primary key default uuidv7(),
    readable_id case_insensitive_text unique not null,
    links jsonb generated always as (construct_links('cdna', id, '{"measurements", "libraries"}')) stored not null,
    library_type case_insensitive_text not null,
    prepared_at timestamptz not null,
    gem_pool_id uuid references gem_pools on delete restrict on update restrict,
    n_amplification_cycles integer not null,
    additional_data jsonb,

    -- a single GEM pool cannot generate more than one cDNA of the same library type
    unique (gem_pool_id, library_type)
);

create table cdna_measurements (
    id uuid primary key default uuidv7(),
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamptz not null,
    data jsonb not null
);

create table cdna_preparers (
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (cdna_id, prepared_by)
);
